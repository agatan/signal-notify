//! `signal-notify` crate provides a simple way to wait for signals in *nix systems through standard
//! `std::sync::mpsc` API.
//!
//! ```no_run
//! use signal_notify::{notify, Signal};
//!
//! let rx = notify(&[Signal::INT, Signal::HUP]);
//! // block unitl receiving SIGINT or SIGHUP.
//! // recv always return Ok because the sender channel will be never closed.
//! rx.recv().unwrap();
//! ```
//!
//! `signal-notify` doesn't support Windows. I'm not familiar with Windows, so I'd be happy if you
//! could help me about it.

extern crate libc;
#[macro_use]
extern crate lazy_static;

use std::io;
use std::sync::{Mutex, mpsc};
use std::collections::HashMap;

use libc::{SIGHUP, SIGINT, SIGQUIT, SIGILL, SIGABRT, SIGFPE, SIGKILL, SIGSEGV, SIGPIPE, SIGALRM,
           SIGTERM, SIGUSR1, SIGUSR2, SIGCHLD, SIGCONT, SIGSTOP, SIGTSTP, SIGTTIN, SIGTTOU,
           SIGBUS, SIGPROF, SIGSYS, SIGTRAP, SIGURG, SIGVTALRM, SIGXCPU, SIGXFSZ, SIGIO, SIGWINCH};

pub fn notify(signal: &[Signal]) -> mpsc::Receiver<Signal> {
    let (tx, rx) = mpsc::channel();
    notify_on(tx, signal);
    rx
}

pub fn notify_on(tx: mpsc::Sender<Signal>, signal: &[Signal]) {
    unsafe {
        let mut sa: libc::sigaction = ::std::mem::zeroed();
        let f: extern "C" fn(libc::c_int, *const libc::siginfo_t, *const libc::c_void) =
            signal_handler;
        sa.sa_sigaction = ::std::mem::transmute(f);
        sa.sa_flags |= libc::SA_SIGINFO;
        for sig in signal {
            ok_or_errno(
                (),
                libc::sigaction(
                    sig.as_sig(),
                    &sa as *const libc::sigaction,
                    ::std::ptr::null_mut(),
                ),
            ).unwrap();
        }
        let mut notifiers = NOTIFIER.lock().unwrap();
        for sig in signal {
            notifiers.entry(*sig).or_insert(Vec::new()).push(tx.clone());
        }
    }
}

extern "C" fn signal_handler(
    signal: libc::c_int,
    _siginfo: *const libc::siginfo_t,
    _ctx: *const libc::c_void,
) {
    let n = signal as i32;
    unsafe {
        libc::write(
            PIPE[1],
            &n as *const _ as *const _,
            ::std::mem::size_of::<i32>(),
        );
    }
}

static mut PIPE: [libc::c_int; 2] = [0, 0];

lazy_static! {
    static ref NOTIFIER: Mutex<HashMap<Signal, Vec<mpsc::Sender<Signal>>>> = {
        unsafe {
            ok_or_errno((), libc::pipe(PIPE.as_mut_ptr())).unwrap();
        }
        start();
        Mutex::new(HashMap::new())
    };
}

type Sig = libc::c_int;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Signal {
    HUP,
    INT,
    QUIT,
    ILL,
    ABRT,
    FPE,
    KILL,
    SEGV,
    PIPE,
    ALRM,
    TERM,
    USR1,
    USR2,
    CHLD,
    CONT,
    STOP,
    TSTP,
    TTIN,
    TTOU,
    BUS,
    PROF,
    SYS,
    TRAP,
    URG,
    VTALRM,
    XCPU,
    XFSZ,
    IO,
    WINCH,
}

impl Signal {
    fn new(sig: Sig) -> Signal {
        match sig {
            SIGHUP => Signal::HUP,
            SIGINT => Signal::INT,
            SIGQUIT => Signal::QUIT,
            SIGILL => Signal::ILL,
            SIGABRT => Signal::ABRT,
            SIGFPE => Signal::FPE,
            SIGKILL => Signal::KILL,
            SIGSEGV => Signal::SEGV,
            SIGPIPE => Signal::PIPE,
            SIGALRM => Signal::ALRM,
            SIGTERM => Signal::TERM,
            SIGUSR1 => Signal::USR1,
            SIGUSR2 => Signal::USR2,
            SIGCHLD => Signal::CHLD,
            SIGCONT => Signal::CONT,
            SIGSTOP => Signal::STOP,
            SIGTSTP => Signal::TSTP,
            SIGTTIN => Signal::TTIN,
            SIGTTOU => Signal::TTOU,
            SIGBUS => Signal::BUS,
            SIGPROF => Signal::PROF,
            SIGSYS => Signal::SYS,
            SIGTRAP => Signal::TRAP,
            SIGURG => Signal::URG,
            SIGVTALRM => Signal::VTALRM,
            SIGXCPU => Signal::XCPU,
            SIGXFSZ => Signal::XFSZ,
            SIGIO => Signal::IO,
            SIGWINCH => Signal::WINCH,
            sig => panic!("unsupported signal number: {}", sig),
        }
    }

    fn as_sig(self) -> Sig {
        match self {
            Signal::HUP => SIGHUP,
            Signal::INT => SIGINT,
            Signal::QUIT => SIGQUIT,
            Signal::ILL => SIGILL,
            Signal::ABRT => SIGABRT,
            Signal::FPE => SIGFPE,
            Signal::KILL => SIGKILL,
            Signal::SEGV => SIGSEGV,
            Signal::PIPE => SIGPIPE,
            Signal::ALRM => SIGALRM,
            Signal::TERM => SIGTERM,
            Signal::USR1 => SIGUSR1,
            Signal::USR2 => SIGUSR2,
            Signal::CHLD => SIGCHLD,
            Signal::CONT => SIGCONT,
            Signal::STOP => SIGSTOP,
            Signal::TSTP => SIGTSTP,
            Signal::TTIN => SIGTTIN,
            Signal::TTOU => SIGTTOU,
            Signal::BUS => SIGBUS,
            Signal::PROF => SIGPROF,
            Signal::SYS => SIGSYS,
            Signal::TRAP => SIGTRAP,
            Signal::URG => SIGURG,
            Signal::VTALRM => SIGVTALRM,
            Signal::XCPU => SIGXCPU,
            Signal::XFSZ => SIGXFSZ,
            Signal::IO => SIGIO,
            Signal::WINCH => SIGWINCH,
        }
    }
}

fn start() {
    ::std::thread::spawn(|| loop {
        let signal = match read_signal() {
            None => break,
            Some(signal) => signal,
        };
        let notifier = NOTIFIER.lock().unwrap();
        if let Some(senders) = notifier.get(&signal) {
            for tx in senders {
                let _ = tx.send(signal);
            }
        }
    });
}

fn read_signal() -> Option<Signal> {
    let mut buf: [u8; 4] = [0; 4];
    unsafe {
        loop {
            let n = libc::read(PIPE[0], buf.as_mut_ptr() as *mut _, 4);
            if n == 0 {
                return None;
            } else if n == -1 {
                let err = io::Error::last_os_error();
                match err.kind() {
                    io::ErrorKind::WouldBlock |
                    io::ErrorKind::Interrupted => continue,
                    _ => panic!("read error in signal_notify: {}", err),
                }
            } else {
                return Some(Signal::new(std::mem::transmute(buf)));
            }
        }
    }
}

fn ok_or_errno<T>(ok: T, errcode: libc::c_int) -> io::Result<T> {
    if errcode != 0 {
        Err(io::Error::from_raw_os_error(errcode))
    } else {
        Ok(ok)
    }
}
