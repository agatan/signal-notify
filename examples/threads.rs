extern crate signal_notify;

use signal_notify::{notify, Signal};
use std::thread;

fn main() {
    let rx = notify(&[Signal::INT]);
    thread::spawn(|| {
        let rx = notify(&[Signal::INT]);
        for sig in rx.iter() {
            println!("in thread: {:?}", sig);
        }
        loop {
            thread::sleep(::std::time::Duration::from_secs(10));
        }
    });
    for sig in rx.iter() {
        println!("main: {:?}", sig);
    }
}
