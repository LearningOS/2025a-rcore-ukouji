//! Process management syscalls
use crate::task::{change_program_brk, exit_current_and_run_next, mmap_current, munmap_current, suspend_current_and_run_next};

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

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    -1
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
        || _len % 0x1000 != 0
        || _prot & 0x7 == 0 || _prot & !0x7 != 0 {
        trace!("reject to mmap start va:{:x}, len:{:x}, prot:{:x}", _start, _len, _prot);
        return -1;
    }

    mmap_current(_start, _len, _prot)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    if _start == 0 || _start & 0xfff != 0 || _len % 0x1000 != 0 {
        trace!("reject to munmap start va:{:x}, len:{:x}", _start, _len);
        return -1;
    }

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
