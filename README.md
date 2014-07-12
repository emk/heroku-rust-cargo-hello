This will eventually replace [heroku-rust-hello][] as soon as there's a way
to lock down external dependencies (something like `cargo freeze`, perhaps).

To build and run:

``` sh
cargo build --verbose
PORT=5000 target/hello
```

Then visit `0.0.0.0:5000` in your browser.  This is based on the
[iron middleware framework][iron].

Note that we use need to use `heroku-buildpack-multi` and
`heroku-buildpack-git` to upgrade Heroku's `git` from 1.7.0 (I think) to
something a bit newer.  This will eventually be simplified.

[heroku-rust-hello]: https://github.com/emk/heroku-rust-hello
[iron]: https://github.com/iron/iron
