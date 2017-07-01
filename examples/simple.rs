extern crate signal_notify;

use signal_notify::{notify, Signal};

fn main() {
    let rx = notify(&[Signal::INT, Signal::WINCH]);
    println!("START");
    println!("RECV: {:?}", rx.recv().unwrap());
    println!("FIN");
}
