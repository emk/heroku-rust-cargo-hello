#![feature(os)]
#![feature(io)]
#![feature(core)]

extern crate iron;
extern crate router;

use std::str::FromStr;
use std::os::getenv;
use std::old_io::net::ip::{Ipv4Addr, Port};
use iron::{Iron, Request, Response, IronResult};
use router::Router;
use iron::status;

// Serves a string to the user.  Try accessing "/".
fn hello(_: &mut Request) -> IronResult<Response> {
    let resp = Response::with((status::Ok, "Hello world!"));
    Ok(resp)
}

// Serves a customized string to the user.  Try accessing "/world".
fn hello_name(req: &mut Request) -> IronResult<Response> {
    let params = req.extensions.get::<Router>().unwrap();
    let name = params.find("name").unwrap();
    let resp = Response::with((status::Ok, format!("Hello, {}!", name)));
    Ok(resp)
}

/// Look up our server port number in PORT, for compatibility with Heroku.
fn get_server_port() -> Port {
    getenv("PORT")
        .and_then(|s| FromStr::from_str(&s[]))
        .unwrap_or(8080)
}

/// Configure and run our server.
fn main() {
    // Set up our URL router.
    let mut router = Router::new();
    router.get("/", hello);
    router.get("/:name", hello_name);

    // Run the server.
    Iron::new(router).listen((Ipv4Addr(0, 0, 0, 0), get_server_port())).unwrap();
}
