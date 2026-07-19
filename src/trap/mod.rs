mod context;

use context::TrapContext;
use core::arch::global_asm;
use riscv::{
    ExceptionNumber, interrupt::{Exception, Trap}, register::{mtvec::TrapMode, scause, stval, stvec},
};

use crate::{println, syscall};

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
                context.x[10] = syscall::sys_call(context.x[17],[context.x[10],context.x[11],context.x[12]])
            }
            Ok(Exception::StoreFault) | Ok(Exception::StorePageFault) => {
                println!("[kernel] PageFault in application, kernel killed it.");
            }
            Ok(Exception::IllegalInstruction) => {
                println!("[kernel] IllegalInstruction in application, kernel killed it.");
            }
            _ => {
                panic!(
                    "Unsupported trap {:?}, stval = {:#x}!",
                    scause.cause(),
                    stval
                );
            }
        },
        Trap::Interrupt(_) => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    context
}
