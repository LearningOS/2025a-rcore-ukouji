//! Process management syscalls
use crate::task::{change_program_brk, exit_current_and_run_next, mmap_current, munmap_current, suspend_current_and_run_next};
use crate::timer::get_time_us;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// Copy something to userspace
/// Procedure:
///     1. mapping the user virtual address into kernel's address space
///     2. performing the copy operation
///     3. unmapping the address
/// NOTE: we assume the src/dst are of same size!
unsafe fn copy_to_user(dst_uva: *mut u8, src_kva: *const u8, size: usize) -> () {
    dst_uva.copy_from(src_kva, size)
}

// unsafe fn copy_from_user(dst_kva: *mut u8, src_uva: *mut u8, size: usize) -> () {
//     dst_kva.copy_from(src_uva, size)
// }

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    debug!("kernel: sys_get_time");
    let us = get_time_us();
    let tv = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    unsafe {
        copy_to_user(_ts as *mut u8, &tv as *const TimeVal as *const u8, _tz)
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    -1
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _prot: usize) -> isize {
    if _start == 0 || _start & 0xfff != 0
        || _prot & 0x7 == 0 || _prot & !0x7 != 0 {
        trace!("reject to mmap start va:{:x}, len:{:x}, prot:{:x}", _start, _len, _prot);
        return -1;
    }

    info!("Requesting to map addr={:x} with len={:x}", _start, _len);
    mmap_current(_start, _len, _prot)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    if _start == 0 || _start & 0xfff != 0 {
        trace!("reject to munmap start va:{:x}, len:{:x}", _start, _len);
        return -1;
    }
    info!("Requesting to unmap addr={:x} with len={:x}", _start, _len);
    munmap_current(_start, _len)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
