//! A library for handling signals in UNIX-like environments.
//!
//! Using this library it is possible to subscribe and unsubscribe from signals and to
//! handle them asynchronously. 
//! 
//! # Example
//! 
//! ```rust
//! let signals = Signals::new().unwrap();
//! signals.subscribe(Interrupt);
//! for s in signals.receiver().iter() {
//!     println!("{:?}", s);
//! }
//! ```
//! 
//! At any given time there can only be one signal handler in the program.
//! `Signals::new()` returns `None` if there is already another signal handler.

#![crate_id = "signals#0.1.0"]
#![crate_type = "lib"]
#![license = "MIT"]

extern crate libc;

use self::libc::{c_int};
use std::sync::{Once, ONCE_INIT};
use std::sync::atomics::{AtomicBool, INIT_ATOMIC_BOOL, Relaxed};
use std::mem::{forget, transmute};

static mut ALIVE: AtomicBool = INIT_ATOMIC_BOOL;
static mut INITIALIZED: Once = ONCE_INIT;
static mut SND: *const Sender<Signal> = 0 as *const Sender<Signal>;
static mut RCV: *const Receiver<Signal> = 0 as *const Receiver<Signal>;

extern {
    fn signal(signum: c_int, hdlr: Option<unsafe extern fn(c_int)>);
}

unsafe extern fn handler(num: c_int) {
    if !ALIVE.load(Relaxed) {
        return;
    }
    let snd: &Sender<Signal> = transmute(SND);
    match num {
        _ if num == Abort     as c_int => snd.send(Abort),
        _ if num == Alarm     as c_int => snd.send(Alarm),
        _ if num == Bus       as c_int => snd.send(Bus),
        _ if num == Child     as c_int => snd.send(Child),
        _ if num == Continue  as c_int => snd.send(Continue),
        _ if num == FPE       as c_int => snd.send(FPE),
        _ if num == Hangup    as c_int => snd.send(Hangup),
        _ if num == Illegal   as c_int => snd.send(Illegal),
        _ if num == Interrupt as c_int => snd.send(Interrupt),
        _ if num == Kill      as c_int => snd.send(Kill),
        _ if num == Pipe      as c_int => snd.send(Pipe),
        _ if num == Quit      as c_int => snd.send(Quit),
        _ if num == Poll      as c_int => snd.send(Poll),
        _ if num == Prof      as c_int => snd.send(Prof),
        _ if num == Segfault  as c_int => snd.send(Segfault),
        _ if num == Stop      as c_int => snd.send(Stop),
        _ if num == TermStop  as c_int => snd.send(TermStop),
        _ if num == Sys       as c_int => snd.send(Sys),
        _ if num == Terminate as c_int => snd.send(Terminate),
        _ if num == Trap      as c_int => snd.send(Trap),
        _ if num == TTIN      as c_int => snd.send(TTIN),
        _ if num == TTOU      as c_int => snd.send(TTOU),
        _ if num == Urgent    as c_int => snd.send(Urgent),
        _ if num == User1     as c_int => snd.send(User1),
        _ if num == User2     as c_int => snd.send(User2),
        _ if num == WinSize   as c_int => snd.send(WinSize),
        _ if num == XCPU      as c_int => snd.send(XCPU),
        _ if num == XFSZ      as c_int => snd.send(XFSZ),
        _ => { },
    }
}

/// Available signals.
pub enum Signal {
    /// Process abort signal
    Abort     = 6,
    /// Alarm clock
    Alarm     = 14,
    /// Access to an undefined portion of a memory object
    Bus       = 10,
    /// Child process terminated, stopped,
    Child     = 18,
    /// Continue executing, if stopped.
    Continue  = 25,
    /// Erroneous arithmetic operation.
    FPE       = 8,
    /// Hangup.
    Hangup    = 1,
    /// Illegal instruction.
    Illegal   = 4,
    /// Terminal interrupt signal.
    Interrupt = 2,
    /// Kill (cannot be caught or ignored).
    Kill      = 9,
    /// Abnormal termination of the process	Write on a pipe with no one to read it.
    Pipe      = 13,
    /// Abnormal termination of the process	Terminal quit signal.
    Quit      = 3,
    /// Pollable event.
    Poll      = 22,
    /// Profiling timer expired.
    Prof      = 29,
    /// Invalid memory reference.
    Segfault  = 11,
    /// Stop executing (cannot be caught or ignored).
    Stop      = 23,
    /// Terminal stop signal.
    TermStop  = 20,
    /// Bad system call.
    Sys       = 12,
    /// Termination signal.
    Terminate = 15,
    /// Trace/breakpoint trap.
    Trap      = 5,
    /// Background process attempting read.
    TTIN      = 26,
    /// Background process attempting write.
    TTOU      = 27,
    /// High bandwidth data is available at a socket.
    Urgent    = 21,
    /// User-defined signal 1.
    User1     = 16,
    /// User-defined signal 2.
    User2     = 17,
    /// Window resized.
    WinSize   = 28,
    /// CPU time limit exceeded.
    XCPU      = 30,
    /// File size limit exceeded.
    XFSZ      = 31,
}

/// Signal handler
pub struct Signals {
    _unit: (),
}

impl Signals {
    /// Create a new signal handler
    ///
    /// Returns `None` if there is already another signal handler in the program.
    pub fn new() -> Option<Signals> {
        unsafe {
            INITIALIZED.doit(|| {
                let (s, r) = channel();
                let s = box s;
                let r = box r;
                SND = &*s as *const _;
                RCV = &*r as *const _;
                forget(s);
                forget(r);
            });
            if ALIVE.compare_and_swap(false, true, Relaxed) {
                return None;
            }
            Some(Signals { _unit: () })
        }
    }

    /// Subscribe to a signal.
    ///
    /// Note: Dropping the signal handler doesn't automatically unsubscribe.
    /// To return to the default behavior, one has to explicitly call `unsubscribe`.
    pub fn subscribe(&self, sig: Signal) {
        unsafe { signal(sig as c_int, Some(handler)); }
    }

    /// Unsubscribe from a signal.
    pub fn unsubscribe(&self, sig: Signal) {
        unsafe { signal(sig as c_int, None); }
    }

    /// Create a non-blocking iterator over all received signals.
    pub fn iter<'a>(&'a self) -> SignalIter<'a> {
        SignalIter { _unit: () }
    }

    /// Return a reference to the internal `Receiver`.
    pub fn receiver<'a>(&'a self) -> &'a Receiver<Signal> {
        unsafe { transmute(RCV) }
    }
}

impl Drop for Signals {
    fn drop(&mut self) {
        unsafe { ALIVE.store(false, Relaxed); }
    }
}

/// Non-blocking iterator over the available signals.
pub struct SignalIter<'a> {
    _unit: (),
}

impl<'a> Iterator<Signal> for SignalIter<'a> {
    fn next(&mut self) -> Option<Signal> {
        let rcv: &Receiver<Signal> = unsafe { transmute(RCV) };
        match rcv.try_recv() {
            Ok(v) => Some(v),
            _ => None,
        }
    }
}
