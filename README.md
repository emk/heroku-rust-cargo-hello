[![Build Status](https://travis-ci.org/emk/heroku-rust-cargo-hello.svg?branch=master)](https://travis-ci.org/emk/heroku-rust-cargo-hello)

To deploy this application to Heroku, use this button:

[![Deploy](https://www.herokucdn.com/deploy/button.png)](https://heroku.com/deploy)

Or, if you'd prefer to use the command line, try running:

``` sh
git clone https://github.com/emk/heroku-rust-cargo-hello.git
cd heroku-rust-cargo-hello
heroku create --buildpack https://github.com/emk/heroku-buildpack-rust.git
git push heroku master
```

This should make a local copy of this application and deploy it to Heroku.

For further instructions, see the [page for this buildpack][buildpack].

[instructions]: http://www.randomhacks.net/2014/09/17/deploying-rust-heroku-iron/
[buildpack]: https://github.com/emk/heroku-buildpack-rust

### Does this work with the latest version of Rust?

This application works with version 1.25.0 of Rust, which theoretically means
that it should run on any future 1.x release of Rust.  If it doesn't work,
please file a bug.

### Does this work with Cloud Foundry?

The application can be deployed to Cloud Foundry as an alternative to
Heroku. From the command line, run:

``` sh
cd heroku-rust-cargo-hello
cf push heroku-rust-cargo-hello -b https://github.com/emk/heroku-buildpack-rust.git
```
