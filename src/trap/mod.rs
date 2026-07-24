mod context;

pub use context::TrapContext;
use core::arch::global_asm;
use riscv::{
    ExceptionNumber, InterruptNumber,
    interrupt::{Exception, Interrupt, Trap},
    register::{mtvec::TrapMode, scause, stval, stvec},
};

use crate::{
    println, syscall,
    task::{exit_current_and_run_next, suspend_current_and_run_next},
    timer::set_trigger,
};

global_asm!(include_str!("trap.S"));

pub fn init() {
    unsafe extern "C" {
        fn _trap_entry();
    }
    let vec = stvec::Stvec::new(_trap_entry as *const () as usize, TrapMode::Direct);
    unsafe {
        stvec::write(vec);
    }
}

#[unsafe(no_mangle)]
pub fn trap_handler(context: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(e) => match Exception::from_number(e) {
            Ok(Exception::UserEnvCall) => {
                context.sepc += 4;
                context.x[10] =
                    syscall::sys_call(context.x[17], [context.x[10], context.x[11], context.x[12]])
            }
            Ok(Exception::StoreFault) | Ok(Exception::StorePageFault) => {
                println!("[kernel] PageFault in application, kernel killed it.");
                exit_current_and_run_next()
            }
            Ok(Exception::IllegalInstruction) => {
                println!("[kernel] IllegalInstruction in application, kernel killed it.");
                exit_current_and_run_next()
            }
            _ => {
                panic!(
                    "Unsupported Exception {:?}, stval = {:#x}!",
                    scause.cause(),
                    stval
                );
            }
        },
        Trap::Interrupt(e) => match Interrupt::from_number(e) {
            Ok(Interrupt::SupervisorTimer) => {
                set_trigger();
                suspend_current_and_run_next();
            }
            _ => {
                panic!(
                    "Unsupported Interrupt {:?}, stval = {:#?}",
                    scause.cause(),
                    stval
                );
            }
        },
    }
    context
}

pub fn goto_user(context: TrapContext) -> ! {
    unsafe extern "C" {
        fn __restore(context_address: usize) -> !;
    }
    unsafe {
        __restore(&context as *const TrapContext as usize);
    }
}
