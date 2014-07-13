#![feature(phase)]

#[phase(plugin, link)] extern crate log;
extern crate http;
extern crate iron;
extern crate logger;
extern crate router;

use std::os::getenv;
use std::io::IoError;
use std::io::net::ip::{Ipv4Addr, Port};
use http::status::Ok;
use http::method::Get;
use iron::{Iron, Chain, Alloy, Request, Response, Server, Status, Unwind, FromFn};
use iron::mixin::Serve;
use logger::Logger;
use router::{Router, Params};

// Log errors that we can't report to the user.
fn log_serve_errors(result: Result<(), IoError>) {
    match result {
        Err(e) => error!("Error serving response: {}", e),
        _ => {}
    }
}

// Serves a string to the user.  Try accessing "/".
fn hello(_req: &mut Request, res: &mut Response, _alloy: &mut Alloy) -> Status {
    log_serve_errors(res.serve(Ok, "Hello from Rust!"));
    Unwind
}

// Serves a customized string to the user.  Try accessing "/world".
fn hello_name(_req: &mut Request, res: &mut Response,
              alloy: &mut Alloy) -> Status {    
    let name = alloy.find::<Params>().unwrap().get("name").unwrap();
    let message = format!("Hello, {}!", name);
    log_serve_errors(res.serve(Ok, message));
    Unwind
}

/// Look up our server port number in PORT, for compatibility with Heroku.
fn get_server_port() -> Port {
    getenv("PORT")
        .and_then(|s| from_str::<Port>(s.as_slice()))
        .unwrap_or(8080)
}

/// Configure and run our server.
fn main() {
    let logger = Logger::new(None);
    let mut router = Router::new();

    router.route(Get, "/".to_string(), vec![], FromFn::new(hello));
    router.route(Get, "/:name".to_string(), vec!["name".to_string()],
                 FromFn::new(hello_name));

    let mut server: Server = Iron::new();
    server.chain.link(logger);
    server.chain.link(router);
    server.listen(Ipv4Addr(0, 0, 0, 0), get_server_port());
}
