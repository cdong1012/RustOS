use alloc::boxed::Box;
use alloc::collections::vec_deque::VecDeque;
use core::fmt;

use aarch64::*;
use crate::console::{kprintln, kprint};
use crate::mutex::Mutex;
use crate::param::{PAGE_MASK, PAGE_SIZE, TICK, USER_IMG_BASE};
use crate::process::{Id, Process, State};
use crate::traps::TrapFrame;
use crate::VMM;
use crate::init::_start;
use pi::timer::tick_in;
use crate::traps::irq::{Irq, IrqHandler, IrqHandlers};
use pi::interrupt::{Interrupt};
use crate::IRQ;
use pi::interrupt::Controller;
use crate::SCHEDULER;
use crate::vm::{VirtualAddr, PagePerm};
use shim::path::Path;
/// Process scheduler for the entire machine.
#[derive(Debug)]
pub struct GlobalScheduler(Mutex<Option<Scheduler>>);

impl GlobalScheduler {
    /// Returns an uninitialized wrapper around a local scheduler.
    pub const fn uninitialized() -> GlobalScheduler {
        GlobalScheduler(Mutex::new(None))
    }

    /// Enter a critical region and execute the provided closure with the
    /// internal scheduler.
    pub fn critical<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Scheduler) -> R,
    {
        let mut guard = self.0.lock();
        f(guard.as_mut().expect("scheduler uninitialized"))
    }


    /// Adds a process to the scheduler's queue and returns that process's ID.
    /// For more details, see the documentation on `Scheduler::add()`.
    pub fn add(&self, process: Process) -> Option<Id> {
        self.critical(move |scheduler| scheduler.add(process))
    }

    /// Performs a context switch using `tf` by setting the state of the current
    /// process to `new_state`, saving `tf` into the current process, and
    /// restoring the next process's trap frame into `tf`. For more details, see
    /// the documentation on `Scheduler::schedule_out()` and `Scheduler::switch_to()`.
    pub fn switch(&self, new_state: State, tf: &mut TrapFrame) -> Id {
        self.critical(|scheduler| scheduler.schedule_out(new_state, tf));
        self.switch_to(tf)
    }

    pub fn switch_to(&self, tf: &mut TrapFrame) -> Id {
        loop {
            let rtn = self.critical(|scheduler| scheduler.switch_to(tf));
            if let Some(id) = rtn {
                return id;
            }
            aarch64::wfe();
        }
    }

    /// Kills currently running process and returns that process's ID.
    /// For more details, see the documentaion on `Scheduler::kill()`.
    #[must_use]
    pub fn kill(&self, tf: &mut TrapFrame) -> Option<Id> {
        self.critical(|scheduler| scheduler.kill(tf))
    }

    pub fn tick_handler(tf : &mut TrapFrame) {
        kprintln!("tick and switch");
        tick_in(TICK);
        SCHEDULER.switch(State::Ready, tf);
    }

    /// Starts executing processes in user space using timer interrupt based
    /// preemptive scheduling. This method should not return under normal conditions.
    pub fn start(&self) -> ! {
        let mut controller : Controller = Controller::new(); // create interrupt controller
        controller.enable(Interrupt::Timer1); // enable timer interrupt
        tick_in(TICK); // set timer interrupt to occur in TICK second
        IRQ.register(Interrupt::Timer1, Box::new(GlobalScheduler::tick_handler)); 

        let mut tf : TrapFrame = TrapFrame::default();
        self.switch_to(&mut tf);
        use crate::param::USER_STACK_BASE;
        let next_page = USER_STACK_BASE - 100*PAGE_SIZE;
        unsafe {
            asm!("mov sp, $0
                  bl context_restore"
                :: "r"(&tf as *const _ as u64)
                :: "volatile");           
        }
        let next_page = (tf.sp + PAGE_SIZE as u64) & !(0b1111);
        unsafe {
            asm!("mov sp, $0
                  eret"
                :: "r"(next_page as u64)
                :: "volatile");
        }

        //1. Before eret, just set your kernel's sp to the next page address of the kernel's sp.. 
        //Just add one page_size to sp, mask it so that it is 16 byte aligned, then set it back to sp. 
        //Since stacks grow downwards, we are resetting the stack pointer to the top of the page.
        loop {}
    }

    /// Initializes the scheduler and add userspace processes to the Scheduler
    pub unsafe fn initialize(&self) {
        let scheduler : Scheduler = Scheduler::new();
        *self.0.lock() = Some(scheduler);
        let p1 = Process::load(Path::new("fib")).unwrap();
        let p2 = Process::load(Path::new("fib")).unwrap();
        let p3 = Process::load(Path::new("fib")).unwrap();
        let p4 = Process::load(Path::new("fib")).unwrap();
        let p5 = Process::load(Path::new("fib")).unwrap();
        
        self.add(p1);
        self.add(p2);
        self.add(p3);
        self.add(p4);
        self.add(p5);
    }

    // The following method may be useful for testing Phase 3:
    //
    // * A method to load a extern function to the user process's page table.
    //
    pub fn test_phase_3(&self, process: &mut Process){
        let mut page = process.vmap.alloc(
            VirtualAddr::from(USER_IMG_BASE as u64), PagePerm::RWX);
        let text = unsafe {
            core::slice::from_raw_parts(test_user_process as *const u8, 24)
        };
        page[0..24].copy_from_slice(text);
    }
}

#[derive(Debug)]
pub struct Scheduler {
    processes: VecDeque<Process>,
    last_id: Option<Id>,
}

impl Scheduler {
    /// Returns a new `Scheduler` with an empty queue.
    fn new() -> Scheduler {
        let vec = VecDeque::new();
        Scheduler {
            processes : vec,
            last_id : None
        }
    }

    /// Adds a process to the scheduler's queue and returns that process's ID if
    /// a new process can be scheduled. The process ID is newly allocated for
    /// the process and saved in its `trap_frame`. If no further processes can
    /// be scheduled, returns `None`.
    ///
    /// It is the caller's responsibility to ensure that the first time `switch`
    /// is called, that process is executing on the CPU.
    fn add(&mut self, mut process: Process) -> Option<Id> {
       let return_id = match self.last_id {
            Some(id) => {
                process.context.tpidr = id.checked_add(1).unwrap();
                id + 1
            },
            None => { // queue is empty, process ID 1
                process.context.tpidr = 1u64;
                1u64
            }
        };
        self.processes.push_back(process); // add to the back of the queue
        self.last_id = Some(return_id);
        return Some(return_id);
    }

    /// Finds the currently running process, sets the current process's state
    /// to `new_state`, prepares the context switch on `tf` by saving `tf`
    /// into the current process, and push the current process back to the
    /// end of `processes` queue.
    ///
    /// If the `processes` queue is empty or there is no current process,
    /// returns `false`. Otherwise, returns `true`.
    // we remove(schedule out) the current running process.
    // we find this by looking at the ID in our current trapframe and look for a running
    // process that match. If find it, we move it to the back of the queue(remove -> push_back). 
    // save the current trap frame into this process to make sure we can return to this trapframe 
    // when we run it again.
    fn schedule_out(&mut self, new_state: State, tf: &mut TrapFrame) -> bool {
        if self.processes.is_empty() {
            return false;
        }
        let mut count = 0;
        for process in self.processes.iter_mut() {
            if process.context.tpidr == tf.tpidr { // find one that matches with current trapframe ID
                match process.state {
                    State::Running => {
                        let mut running_process = match self.processes.remove(count) {
                            Some(p) => p, // remove running process
                            None => {
                                return false;
                            }
                        };
                        running_process.state = new_state; // set it to the new state
                        *running_process.context = *tf; // set up trapframe
                        self.processes.push_back(running_process); // push to the end of queue
                        return true;
                    },
                    _ => {
                    }
                }
            }
            count += 1;
        }
        return false; // does not find a running process..
    }

    /// Finds the next process to switch to, brings the next process to the
    /// front of the `processes` queue, changes the next process's state to
    /// `Running`, and performs context switch by restoring the next process`s
    /// trap frame into `tf`.
    ///
    /// If there is no process to switch to, returns `None`. Otherwise, returns
    /// `Some` of the next process`s process ID.
    // find the process that is ready to the front of the queue 
    // change state to running, then context switch
    // context switching: restore this running process's trapframe into 
    // the current trapframe. After this, we can call context_restore to reload registers
    // back into the vector!
    fn switch_to(&mut self, tf: &mut TrapFrame) -> Option<Id> {
        use crate::vm::*;
        if self.processes.is_empty() {
            return None;
        }
        let mut count = 0;
        for process in self.processes.iter_mut() {
            if process.is_ready() {     // if find the first process that is ready
                let mut ready_process = match self.processes.remove(count) {
                    Some(p) => p,
                    None => {
                        return None;
                    }
                };
                ready_process.state = State::Running; // set state to running
                *tf = *(ready_process.context); // restore by setting tf to process trap frame
                let id = ready_process.context.tpidr;
                self.processes.push_front(ready_process);
                return Some(id);
            }
            count += 1; // count index for remove
        }
        return None;
    }

    /// Kills currently running process by scheduling out the current process
    /// as `Dead` state. Removes the dead process from the queue, drop the
    /// dead process's instance, and returns the dead process's process ID.
    fn kill(&mut self, tf: &mut TrapFrame) -> Option<Id> {
        self.schedule_out(State::Dead, tf); // move this process to the back of the queue;
        match self.processes.pop_back() { // pop this process, drop it's instance
            Some(dead_process) => {
                let id = dead_process.context.tpidr; // store id
                drop(dead_process); // drop it
                return Some(id); // return id
            }, 
            None => {
                return None;
            }
        } 
    }
}

pub extern "C" fn  test_user_process() -> ! {
    loop {
        let ms = 10000;
        let error: u64;
        let elapsed_ms: u64;
        unsafe {
            asm!("mov x0, $2
              svc 1
              mov $0, x0
              mov $1, x7"
                 : "=r"(elapsed_ms), "=r"(error)
                 : "r"(ms)
                 : "x0", "x7"
                 : "volatile");
        }
    }
}

