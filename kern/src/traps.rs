mod frame;
mod syndrome;
mod syscall;

pub mod irq;
use crate::shell::shell;
pub use self::frame::TrapFrame;

use pi::interrupt::{Interrupt};
use crate::vm::{VirtualAddr};
use self::syndrome::Syndrome;
use self::syscall::handle_syscall;
use crate::console::{kprintln};
use crate::IRQ;
#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Kind {
    Synchronous = 0,
    Irq = 1,
    Fiq = 2,
    SError = 3,
}

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Source {
    CurrentSpEl0 = 0,
    CurrentSpElx = 1,
    LowerAArch64 = 2,
    LowerAArch32 = 3,
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Info {
    source: Source,
    kind: Kind,
}

/// This function is called when an exception occurs. The `info` parameter
/// specifies the source and kind of exception that has occurred. The `esr` is
/// the value of the exception syndrome register. Finally, `tf` is a pointer to
/// the trap frame for the exception.
#[no_mangle]
pub extern "C" fn handle_exception(info: Info, esr: u32, tf: &mut TrapFrame) {
    let syndrome = Syndrome::from(esr);
    if info.kind == Kind::Irq {
        IRQ.invoke(Interrupt::Timer1, tf);
    } else {
        match syndrome {
            Syndrome::Brk(a) => {
                kprintln!("Encountering brk {}", a);
                shell(">");
                tf.elr += 4 as u64; 
            },
            Syndrome::Svc(a) => {
                handle_syscall(a as u16, tf); // sleep: a = 1 : svc(1)
            },
            _ => {
                kprintln!("info: {:?}", info);
                kprintln!("Program counter: {:?}", VirtualAddr::from(tf.elr));
                kprintln!("EXCEPTION ENCOUNTERED...SOMETHING WENT WRONG CAUSE U SUCK :D");
                loop {} // use this when debugging            
            }
        }  
        
    }
    
}
