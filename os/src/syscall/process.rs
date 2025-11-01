//! Process management syscalls
use crate::task;
use crate::timer::get_time_us;
use core::mem::size_of;
use core::cmp::min;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    task::exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    task::suspend_current_and_run_next();
    0
}

/// Copy something to userspace
/// Procedure:
///     1. finding the PA of given userspace VA
///     2. mapping the user PA into kernel's address space as Identical mapping
///     3. performing the copy operation
///     4. unmapping the address
/// NOTE: we assume the src/dst are of same size!
/// @return: number of bytes copied
unsafe fn copy_to_user(dst_uva: *mut u8, src_kva: *const u8, size: usize) -> usize {
    let dst_uva = dst_uva as usize;
    let src_kva = src_kva as usize;
    let mut s = 0;
    while s < size {
        // Copy contents in a 4K page
        let nbytes = min(0x1000 - ((dst_uva + s) & 0xfff), size - s);
        let dst = task::virt_to_phys(dst_uva + s);
        if dst == 0 {
            return s;
        }
        debug!("DST UVA:0x{:x} KVA=0x{:x}, len=0x{:x}", dst_uva + s ,dst, nbytes);
        if task::mmap_current_identical(dst, nbytes, true) != 0 {
            // give up copy as mmap is failing
            return s;
        }
        (dst as *mut u8).copy_from((src_kva + s) as *const u8, nbytes);
        task::munmap_current(dst, nbytes);
        s += nbytes;
    }
    s
}

#[allow(unused)]
/// Copy something from userspace
/// Procedure:
///     1. finding the PA of given userspace VA
///     2. mapping the user PA into kernel's address space as Identical mapping
///     3. performing the copy operation
///     4. unmapping the address
/// NOTE: we assume the src/dst are of same size!
/// @return: number of bytes copied
unsafe fn copy_from_user(dst_kva: *mut u8, src_uva: *const u8, size: usize) -> usize {
    let dst_kva = dst_kva as usize;
    let src_uva = src_uva as usize;
    let mut s = 0;
    while s < size {
        // Copy contents in a 4K page
        let nbytes = min(0x1000 - ((src_uva + s) & 0xfff), size - s);
        let src = task::virt_to_phys(src_uva + s);
        if src == 0 {
            return s;
        }
        if task::mmap_current_identical(src, nbytes, false) != 0 {
            // give up copy as mmap is failing
            warn!("Unable to mmap SRC=0x{:x} with length=0x{:x}", src, nbytes);
            return s;
        }
        ((dst_kva + s) as *mut u8).copy_from(src as *const u8, nbytes);
        task::munmap_current(src, nbytes);
        s += nbytes;
    }
    s
}

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
    let size = size_of::<TimeVal>();
    unsafe {
        if size != copy_to_user(_ts as *mut u8, &tv as *const TimeVal as *const u8, size) {
            -1
        } else {
            0
        }
    }
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    let val: isize = 0;
    match _trace_request {
        0 => {
            // read *_id as isize
            unsafe {
                // ret = *_id
                warn!("copying from 0x{:x}", _id);
                let size = size_of::<isize>();
                let nb = copy_from_user(&val as *const isize as *mut u8,
                                        _id as *mut u8,
                                        size);
                warn!("copied 0x{:x} bytes from 0x{:x}", nb, _id);
                if size != nb {
                     // copy failed
                     -1
                } else {
                    val
                }
            }
        },
        1 => {
            unsafe {
                // *_id = data
                let size = size_of::<u8>();
                if size != copy_to_user(_id as *mut u8,
                                        &_data as * const usize as *const u8,
                                        size) {
                    -1
                } else {
                    0
                }
            }
        },
        2 => todo!(),
        _ => panic!("Unsupported trace request!"),
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _prot: usize) -> isize {
    if _start == 0 || _start & 0xfff != 0
        || _prot & 0x7 == 0 || _prot & !0x7 != 0 {
        trace!("reject to mmap start va:{:x}, len:{:x}, prot:{:x}", _start, _len, _prot);
        return -1;
    }

    info!("Requesting to map addr={:x} with len={:x}", _start, _len);
    task::mmap_current(_start, _len, _prot)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    if _start == 0 || _start & 0xfff != 0 {
        trace!("reject to munmap start va:{:x}, len:{:x}", _start, _len);
        return -1;
    }
    info!("Requesting to unmap addr={:x} with len={:x}", _start, _len);
    task::munmap_current(_start, _len)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = task::change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
