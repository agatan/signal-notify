extern crate signal_notify;

use signal_notify::{notify, Signal};
use std::thread;

fn main() {
    let rx = notify(&[Signal::INT]);
    println!("Send 5 SIGINTs to kill this process");
    thread::spawn(|| {
        let rx = notify(&[Signal::INT]);
        for sig in rx.iter().take(5) {
            println!("in thread: {:?}", sig);
        }
        loop {
            thread::sleep(::std::time::Duration::from_secs(10));
        }
    });
    for sig in rx.iter().take(5) {
        println!("main: {:?}", sig);
    }
}
