use crate::batch::run_next_app;
pub fn sys_exit(exit_code: i32) -> ! {
    crate::println!("kernel: sys_exit with code {exit_code}");
    run_next_app();
}
