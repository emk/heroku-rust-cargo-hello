To deploy this application to Heroku, try running:

``` sh
git clone https://github.com/emk/heroku-rust-cargo-hello.git
cd heroku-rust-cargo-hello
heroku create --buildpack https://github.com/emk/heroku-buildpack-rust.git
git push heroku master
```

This should make a local copy of this application and deploy it to Heroku.

### Building locally

If you trust the Rust maintainers with root access to your machine, run:

``` sh
curl -s https://static.rust-lang.org/rustup.sh | sudo sh
```

Then run:

``` sh
cd heroku-rust-cargo-hello
cargo build
```

To run the binary, try:

``` sh
PORT=5000 target/hello
```

Then visit `0.0.0.0:5000` in your browser.  This is based on the
[iron middleware framework][iron].

For further information on Rust, see the [Rust Guide][guide].

[iron]: https://github.com/iron/iron
[guide]: http://doc.rust-lang.org/guide.html

### Updating to the latest Rust

Grab the latest nightly builds from the usual location:

``` sh
curl -O https://static.rust-lang.org/dist/rust-nightly-x86_64-unknown-linux-gnu.tar.gz
curl -O https://static.rust-lang.org/cargo-dist/cargo-nightly-x86_64-unknown-linux-gnu.tar.gz
```

Then upload these files to an S3 bucket or a webserver that you control,
and edit `RustConfig` to point to the appropriate URLs.  While editing
`RustConfig`, also update `VERSION` and `CARGO_VERSION` to have the correct
date.

To update your library dependencies, run:

``` sh
cargo update
```

Now try to build.  With luck, it shouldn't take too long to adapt to any
API changes.
