`signal-notify` crate provides a simple way to wait for signals in *nix systems through standard
`std::sync::mpsc` API.

```
use signal_notify::{notify, Signal};

let rx = notify(&[Signal::INT, Signal::HUP]);
// block unitl receiving SIGINT or SIGHUP.
// recv always return Ok because the sender channel will be never closed.
rx.recv().unwrap()
```

`signal-notify` doesn't support Windows. I'm not familiar with Windows, so I'd be happy if you
could help me about it.
