use std::fs::File;
use std::io::Error;
use std::mem;
use std::os::unix::io::AsRawFd;

use liburing::*;

const QUEUE_DEPTH: u32 = 4;

#[test]
fn test_io_uring_fsync() {
    let mut ring = unsafe {
        let mut s = mem::MaybeUninit::<io_uring>::uninit();
        let ret = io_uring_queue_init(QUEUE_DEPTH, s.as_mut_ptr(), 0);
        if ret < 0 {
            panic!("io_uring_queue_init: {:?}", Error::from_raw_os_error(-ret));
        }
        s.assume_init()
    };

    let file = File::open("/proc/self/exe").unwrap();

    // 3 linked operations: NOP, NOP -> FSYNC+DRAIN
    let pending = unsafe {
        let sqe = io_uring_get_sqe(&mut ring);
        if sqe == std::ptr::null_mut() {
            panic!("free sqe missing");
        }
        io_uring_prep_nop(sqe);

        let sqe = io_uring_get_sqe(&mut ring);
        if sqe == std::ptr::null_mut() {
            panic!("free sqe missing");
        }
        io_uring_prep_nop(sqe);
        io_uring_sqe_set_flags(sqe, io_uring_sqe_flags_bit_IOSQE_IO_LINK_BIT);

        let sqe = io_uring_get_sqe(&mut ring);
        if sqe == std::ptr::null_mut() {
            panic!("free sqe missing");
        }
        io_uring_prep_nop(sqe);
        io_uring_prep_fsync(sqe, file.as_raw_fd(), IORING_FSYNC_DATASYNC);
        io_uring_sqe_set_flags(sqe, io_uring_sqe_flags_bit_IOSQE_IO_DRAIN_BIT);
        (*sqe).user_data = 1;

        let ret = io_uring_submit(&mut ring);
        if ret < 0 {
            panic!("io_uring_submit: {:?}", Error::from_raw_os_error(-ret));
        }

        ret
    };

    let mut cqe: *mut io_uring_cqe = unsafe { std::mem::zeroed() };
    for _ in 0..pending {
        let ret = unsafe { io_uring_wait_cqe(&mut ring, &mut cqe) };
        if ret < 0 {
            panic!("io_uring_wait_cqe: {:?}", Error::from_raw_os_error(-ret));
        }
        unsafe {
            if (*cqe).res == -22 /* -EINVAL (22) */ && (*cqe).user_data == 1 {
                panic!(
                    "kernel doesn't support IOSQE_IO_DRAIN (*cqe).res = {}, (*cqe).user = {}",
                    (*cqe).res,
                    (*cqe).user_data
                );
            }
            if (*cqe).res < 0 {
                panic!(
                    "operation error: {:?}, (*cqe).user = {}",
                    Error::from_raw_os_error(-(*cqe).res),
                    (*cqe).user_data
                );
            }
            if (*cqe).res != 0 {
                panic!(
                    "(*cqe).res = {}, (*cqe).user = {}",
                    (*cqe).res,
                    (*cqe).user_data
                );
            }
            io_uring_cqe_seen(&mut ring, cqe);
        }
    }

    // println!("Submitted={}, completed={}", pending, done);
    unsafe { io_uring_queue_exit(&mut ring) };
}
