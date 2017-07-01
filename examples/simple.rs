extern crate signal_notify;

use signal_notify::{notify, Signal};

fn main() {
    let rx = notify(&[Signal::INT, Signal::WINCH]);
    println!("Waiting SIGINT and SIGWINCH...");
    println!("got: {:?}", rx.recv().unwrap());
}
