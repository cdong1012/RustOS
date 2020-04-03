use alloc::boxed::Box;
use core::time::Duration;

use crate::console::CONSOLE;
use crate::process::{State, Process};
use crate::traps::TrapFrame;
use crate::SCHEDULER;
use kernel_api::*;
use pi::timer::current_time;
use crate::console::{kprintln, kprint};
use crate::vm::{VirtualAddr, PagePerm};
/// Sleep for `ms` milliseconds.
///
/// This system call takes one parameter: the number of milliseconds to sleep.
///
/// In addition to the usual status value, this system call returns one
/// parameter: the approximate true elapsed time from when `sleep` was called to
/// when `sleep` returned.
pub fn sys_sleep(ms: u32, tf: &mut TrapFrame) {
    let time_start = current_time().as_millis();
    let mut time_done = 0u128;
    let final_time = current_time().as_millis() + ms as u128;
    let boxed_fnmut = Box::new(move |process : &mut Process| -> bool { 
        kprintln!("Progress for rocess: {}, current_time: {}. Final time: {}",process.context.tpidr, current_time().as_millis(), final_time);
        if current_time().as_millis() <= final_time {
            kprintln!("TIME IS NOT UP");
            false
        } else {
            time_done = current_time().as_millis();
            kprintln!("TIME IS UP!!");
            process.state = State::Ready;
            process.context.x0 = (time_done - time_start) as u64; // elapsed time
            process.context.x7 = 1;
            true
        }
    });
    
    SCHEDULER.switch(State::Waiting(boxed_fnmut), tf);
}

/// Returns current time.
///
/// This system call does not take parameter.
///
/// In addition to the usual status value, this system call returns two
/// parameter:
///  - current time as seconds
///  - fractional part of the current time, in nanoseconds.
pub fn sys_time(tf: &mut TrapFrame) {
    let curr = current_time();
    let current_time = curr.as_secs() as u64;
    let fraction = curr.as_nanos() as u64;
    tf.x0 = current_time;
    tf.x1 = fraction;
    tf.x7 = 1;
}

/// Kills current process.
///
/// This system call does not take paramer and does not return any value.
pub fn sys_exit(tf: &mut TrapFrame) {
    kprintln!("Sys_exit is called for process {}", tf.tpidr);
    SCHEDULER.switch(State::Dead, tf);
}

/// Write to console.
///
/// This system call takes one parameter: a u8 character to print.
///
/// It only returns the usual status value.
pub fn sys_write(b: u8, tf: &mut TrapFrame) {
    tf.x7 = 1;
    kprint!("{}", b as char);
}

/// Returns current process's ID.
///
/// This system call does not take parameter.
///
/// In addition to the usual status value, this system call returns a
/// parameter: the current process's ID.
pub fn sys_getpid(tf: &mut TrapFrame) {
    tf.x0 = tf.tpidr;
    tf.x7 = 1;
}

pub fn handle_syscall(num: u16, tf: &mut TrapFrame) {
    use crate::console::kprintln;
    if num == NR_SLEEP as u16 { // sleep 
        let ms = tf.x0 as u32; 
        sys_sleep(ms, tf);
    } else if num == NR_WRITE as u16 { // write
        let byte = tf.x0 as u8;
        sys_write(byte, tf);
    } else if num == NR_GETPID as u16 { // get id
        sys_getpid(tf);
    } else if num == NR_EXIT as u16 { // exit
        sys_exit(tf);
    } else if num == NR_TIME as u16 {// time
        sys_time(tf);
    }
}
