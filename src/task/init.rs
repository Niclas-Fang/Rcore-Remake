use alloc::sync::Arc;
use lazy_static::lazy_static;

use crate::{
    loader::get_app_data_by_name,
    task::{control_block::TaskControlBlock, manager::add_task},
};

lazy_static! {
    pub static ref INIT: Arc<TaskControlBlock> = Arc::new(TaskControlBlock::new(
        get_app_data_by_name("initproc").unwrap()
    ));
}

pub fn add_initproc() {
    add_task(INIT.clone());
}
