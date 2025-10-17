use crate::task::get_current_task_id;
use crate::config::MAX_APP_NUM;
use crate::syscall::{SYSCALL_TRACE, SYSCALL_EXIT, SYSCALL_WRITE, SYSCALL_YIELD, SYSCALL_GET_TIME};

static mut __TRACE: [[usize; 5]; 16] =  [[0 as usize; 5]; MAX_APP_NUM];

fn get_internal_id_from_syscall(syscall: usize) -> usize {
    match syscall{
        SYSCALL_WRITE => 0,
        SYSCALL_EXIT => 1,
        SYSCALL_YIELD => 2,
        SYSCALL_GET_TIME => 3,
        SYSCALL_TRACE => 4,
        _=> {println!("unsupported syscall {}, assigning as 6", syscall); 6 }
    }
}
pub fn trace_syscall(syscall: usize) -> () {
    let id = get_current_task_id();
    let trace_id = get_internal_id_from_syscall(syscall);
    unsafe {__TRACE[id][trace_id] += 1;}
}

pub unsafe fn get_syscall(syscall: usize) -> usize {
    let id = get_current_task_id();
    let trace_id = get_internal_id_from_syscall(syscall);
    unsafe {__TRACE[id][trace_id]}
}