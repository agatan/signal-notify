extern crate signal_notify;

use signal_notify::{notify, Signal};

fn main() {
    let sigint = notify(&[Signal::INT]);
    let sigwinch = notify(&[Signal::WINCH]);

    for _ in 0..5 {
        println!("{:?}", sigint.recv());
        println!("{:?}", sigwinch.recv());
    }
}
