#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use std::io::Error;
    use std::mem;

    use crate::*;

    const QUEUE_DEPTH: u32 = 4;

    #[test]
    fn test_io_uring_queue_init() {
        let mut ring = unsafe {
            let mut s = mem::MaybeUninit::<io_uring>::uninit();
            let ret = io_uring_queue_init(QUEUE_DEPTH, s.as_mut_ptr(), 0);
            if ret < 0 {
                panic!("io_uring_queue_init: {:?}", Error::from_raw_os_error(ret));
            }
            s.assume_init()
        };

        loop {
            let sqe = unsafe { io_uring_get_sqe(&mut ring) };
            if sqe == std::ptr::null_mut() {
                break;
            }
            unsafe { io_uring_prep_nop(sqe) };
        }
        let ret = unsafe { io_uring_submit(&mut ring) };
        if ret < 0 {
            panic!("io_uring_submit: {:?}", Error::from_raw_os_error(ret));
        }

        let mut cqe: *mut io_uring_cqe = unsafe { std::mem::zeroed() };
        // let mut done = 0;
        let pending = ret;
        for _ in 0..pending {
            let ret = unsafe { io_uring_wait_cqe(&mut ring, &mut cqe) };
            if ret < 0 {
                panic!("io_uring_wait_cqe: {:?}", Error::from_raw_os_error(ret));
            }
            // done += 1;
            if unsafe { (*cqe).res } < 0 {
                eprintln!("(*cqe).res = {}", unsafe { (*cqe).res });
            }
            unsafe { io_uring_cqe_seen(&mut ring, cqe) };
        }

        // println!("Submitted={}, completed={}", pending, done);
        unsafe { io_uring_queue_exit(&mut ring) };
    }
}
