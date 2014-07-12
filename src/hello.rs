extern crate http;
extern crate iron;

use std::os::getenv;
use std::io::net::ip::{Ipv4Addr, Port};
use iron::{Iron, Chain, Alloy, Request, Response, Server, Status, Continue, FromFn};
use iron::mixin::Serve;

// An example middleware handler, based on the Iron example code.
fn hello(_req: &mut Request, res: &mut Response, _alloy: &mut Alloy) -> Status {
    let _ = res.serve(::http::status::Ok, "Hello from Rust!");
    Continue
}

/// Look up our server port number in PORT, for
/// compatibility with Heroku.
fn get_server_port() -> Port {
    getenv("PORT")
        .and_then(|s| from_str::<Port>(s.as_slice()))
        .unwrap_or(8080)
}

/// Configure and run our server.
fn main() {
    let mut server: Server = Iron::new();
    server.chain.link(FromFn::new(hello));
    server.listen(Ipv4Addr(0, 0, 0, 0), get_server_port());
}
