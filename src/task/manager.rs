use super::task::{self, TaskControlBlock};
use crate::{loader::num_apps, sync_refcell::SyncRefCell};
use alloc::{collections::VecDeque, sync::Arc};
use lazy_static::lazy_static;

pub struct TaskManager {
    ready_queue: VecDeque<Arc<task::TaskControlBlock>>,
}

impl TaskManager {
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue.pop_front()
    }
}
pub fn add_task(task: Arc<TaskControlBlock>) {
    TASK_MANAGER.borrow_mut().add(task);
}
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.borrow_mut().fetch()
}
lazy_static! {
    pub static ref TASK_MANAGER: SyncRefCell<TaskManager> = {
        let app_num = num_apps();
        let tasks: VecDeque<_> = (0..app_num)
            .map(TaskControlBlock::new)
            .map(Arc::new)
            .collect();
        unsafe { SyncRefCell::new(TaskManager { ready_queue: tasks }) }
    };
}
