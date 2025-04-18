// Defining IOURINGINLINE upfront prevents liburing.h to define it as "static inline"
// and makes all functions visible to bindgen.
#define IOURINGINLINE
#include "lib/src/include/liburing.h"
