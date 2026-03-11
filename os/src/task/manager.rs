//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::sync::UPSafeCell;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::cmp::Ordering;
use lazy_static::*;

const BIG_STRIDE: isize = 65536;

fn stride_cmp(lhs: isize, rhs: isize) -> Ordering {
    if lhs == rhs {
        Ordering::Equal
    } else if lhs.wrapping_sub(rhs).is_negative() {
        Ordering::Less
    } else {
        Ordering::Greater
    }
}
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_queue: Vec<Arc<TaskControlBlock>>,
}

/// A simple stride scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: Vec::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push(task);
    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        if self.ready_queue.is_empty() {
            return None;
        }
        let mut min = None;
        for (idx, task) in self.ready_queue.iter().enumerate() {
            let stride = task.inner_exclusive_access().stride;
            if min
                .map(|(_, min_stride)| stride_cmp(stride, min_stride).is_lt())
                .unwrap_or(true)
            {
                min = Some((idx, stride));
            }
        }
        let (min_stride_idx, _) = min.expect("ready_queue is not empty");
        let task = self.ready_queue.swap_remove(min_stride_idx);
        let mut task_inner = task.inner_exclusive_access();
        task_inner.stride = task_inner
            .stride
            .wrapping_add(BIG_STRIDE / task_inner.priority);
        drop(task_inner);
        Some(task)
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
}
