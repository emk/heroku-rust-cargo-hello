This will eventually replace [heroku-rust-hello][] as soon as there's a way
to lock down external dependencies (something like `cargo freeze`, perhaps).

To build and run:

``` sh
cargo build --verbose
PORT=5000 target/hello
```

Then visit `0.0.0.0:5000` in your browser.  This is based on the
[iron middleware framework][iron].

[heroku-rust-hello]: https://github.com/emk/heroku-rust-hello
[iron]: https://github.com/iron/iron
