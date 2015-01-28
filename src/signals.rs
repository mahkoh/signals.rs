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
#![crate_type = "lib"]

#[allow(unstable)]
extern crate libc;

use self::libc::{c_int};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Once, ONCE_INIT};
use std::sync::atomic::{AtomicBool, ATOMIC_BOOL_INIT};
use std::sync::atomic::Ordering::Relaxed;
use std::mem::{forget, transmute};

static mut ALIVE: AtomicBool = ATOMIC_BOOL_INIT;
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
        _ if num == Signal::Abort     as c_int => snd.send(Signal::Abort),
        _ if num == Signal::Alarm     as c_int => snd.send(Signal::Alarm),
        _ if num == Signal::Bus       as c_int => snd.send(Signal::Bus),
        _ if num == Signal::Child     as c_int => snd.send(Signal::Child),
        _ if num == Signal::Continue  as c_int => snd.send(Signal::Continue),
        _ if num == Signal::FPE       as c_int => snd.send(Signal::FPE),
        _ if num == Signal::Hangup    as c_int => snd.send(Signal::Hangup),
        _ if num == Signal::Illegal   as c_int => snd.send(Signal::Illegal),
        _ if num == Signal::Interrupt as c_int => snd.send(Signal::Interrupt),
        _ if num == Signal::Kill      as c_int => snd.send(Signal::Kill),
        _ if num == Signal::Pipe      as c_int => snd.send(Signal::Pipe),
        _ if num == Signal::Quit      as c_int => snd.send(Signal::Quit),
        _ if num == Signal::Poll      as c_int => snd.send(Signal::Poll),
        _ if num == Signal::Prof      as c_int => snd.send(Signal::Prof),
        _ if num == Signal::Segfault  as c_int => snd.send(Signal::Segfault),
        _ if num == Signal::Stop      as c_int => snd.send(Signal::Stop),
        _ if num == Signal::TermStop  as c_int => snd.send(Signal::TermStop),
        _ if num == Signal::Sys       as c_int => snd.send(Signal::Sys),
        _ if num == Signal::Terminate as c_int => snd.send(Signal::Terminate),
        _ if num == Signal::Trap      as c_int => snd.send(Signal::Trap),
        _ if num == Signal::TTIN      as c_int => snd.send(Signal::TTIN),
        _ if num == Signal::TTOU      as c_int => snd.send(Signal::TTOU),
        _ if num == Signal::Urgent    as c_int => snd.send(Signal::Urgent),
        _ if num == Signal::User1     as c_int => snd.send(Signal::User1),
        _ if num == Signal::User2     as c_int => snd.send(Signal::User2),
        _ if num == Signal::WinSize   as c_int => snd.send(Signal::WinSize),
        _ if num == Signal::XCPU      as c_int => snd.send(Signal::XCPU),
        _ if num == Signal::XFSZ      as c_int => snd.send(Signal::XFSZ),
        _ => Ok(()),
    }.unwrap_or_else(|_| ());
}

/// Available signals.
#[derive(Copy, Debug)]
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
pub struct Signals;

impl Signals {
    /// Create a new signal handler
    ///
    /// Returns `None` if there is already another signal handler in the program.
    pub fn new() -> Option<Signals> {
        unsafe {
            INITIALIZED.call_once(|| {
                let (s, r) = channel();
                let s = Box::new(s);
                let r = Box::new(r);
                SND = &*s as *const _;
                RCV = &*r as *const _;
                forget(s);
                forget(r);
            });
            if ALIVE.compare_and_swap(false, true, Relaxed) {
                return None;
            }
            Some(Signals)
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
        SignalIter
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
pub struct SignalIter<'a>;

impl<'a> Iterator for SignalIter<'a> {
    type Item = Signal;

    fn next(&mut self) -> Option<Signal> {
        let rcv: &Receiver<Signal> = unsafe { transmute(RCV) };
        match rcv.try_recv() {
            Ok(v) => Some(v),
            _ => None,
        }
    }
}
