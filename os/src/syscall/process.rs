//! Process management syscalls
use crate::mm::{translated_byte_buffer, PTEFlags, PageTable, VirtAddr};
use crate::task::current_user_token;
use crate::task::{
    change_program_brk, exit_current_and_run_next, get_current_syscall_times, mmap_current,
    suspend_current_and_run_next,
};
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

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let time_val = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    let time_val_bytes = unsafe {
        core::slice::from_raw_parts(
            &time_val as *const TimeVal as *const u8,
            core::mem::size_of::<TimeVal>(),
        )
    };
    let mut copied = 0usize;
    for dst in translated_byte_buffer(
        current_user_token(),
        ts as *const u8,
        core::mem::size_of::<TimeVal>(),
    ) {
        let len = dst.len().min(time_val_bytes.len() - copied);
        dst[..len].copy_from_slice(&time_val_bytes[copied..copied + len]);
        copied += len;
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    match trace_request {
        0 => {
            let page_table = PageTable::from_token(current_user_token());
            let va = VirtAddr::from(id);
            if let Some(pte) = page_table.translate(va.floor()) {
                let flags = pte.flags();
                if flags.contains(PTEFlags::V)
                    && flags.contains(PTEFlags::U)
                    && flags.contains(PTEFlags::R)
                {
                    pte.ppn().get_bytes_array()[va.page_offset()] as isize
                } else {
                    -1
                }
            } else {
                -1
            }
        }
        1 => {
            let page_table = PageTable::from_token(current_user_token());
            let va = VirtAddr::from(id);
            if let Some(pte) = page_table.translate(va.floor()) {
                let flags = pte.flags();
                if flags.contains(PTEFlags::V)
                    && flags.contains(PTEFlags::U)
                    && flags.contains(PTEFlags::W)
                {
                    pte.ppn().get_bytes_array()[va.page_offset()] = data as u8;
                    0
                } else {
                    -1
                }
            } else {
                -1
            }
        }
        2 => get_current_syscall_times(id),
        _ => -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    trace!("kernel: sys_mmap");
    mmap_current(start, len, prot)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    -1
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
