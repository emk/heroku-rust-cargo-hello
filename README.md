To deploy this application to Heroku, try running:

``` sh
git clone https://github.com/emk/heroku-rust-cargo-hello.git
cd heroku-rust-cargo-hello
heroku create --buildpack https://github.com/emk/heroku-buildpack-rust.git
git push heroku master
```

This should make a local copy of this application and deploy it to Heroku.

For further instructions, see
[Deploying Rust applications to Heroku, with example code for Iron][instructions].

[instructions]: http://www.randomhacks.net/2014/09/17/deploying-rust-heroku-iron/
