To deploy this application to Heroku, try running:

``` sh
git clone https://github.com/emk/heroku-rust-cargo-hello.git
cd heroku-rust-cargo-hello
heroku create --buildpack https://github.com/emk/heroku-buildpack-rust.git
git push heroku master
```

This will probably fail, because `RustConfig` locks down a specific version
of Rust and Cargo, but the `Cargo.toml` file allows our library versions to
float free, so we'll wind up with mismatched compilers and libraries.
Aren't pre-1.0 compilers fun?

For an older version of this application which actually deploys reliably,
see [heroku-rust-hello][].

## Updating the compiler and Cargo executables

Grab the latest nightly builds from the usual location:

``` sh
curl -O http://static.rust-lang.org/dist/rust-nightly-x86_64-unknown-linux-gnu.tar.gz
curl -O http://static.rust-lang.org/cargo-dist/cargo-nightly-linux.tar.gz

Then upload these files to an S3 bucket or a webserver that you control,
and edit `RustConfig` to point to the appropriate URLs.

To install these files locally, see installation instructions for
[Rust][rust-install] and [Cargo][cargo-install].

## Building locally

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

[rust-install]: http://doc.rust-lang.org/tutorial.html#getting-started
[cargo-install]: https://github.com/rust-lang/cargo
[heroku-rust-hello]: https://github.com/emk/heroku-rust-hello
[iron]: https://github.com/iron/iron
