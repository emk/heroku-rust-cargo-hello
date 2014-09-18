extern crate http;
extern crate iron;
extern crate router;
extern crate logger;

use std::os::getenv;
use std::io::net::ip::{Ipv4Addr, Port};
use iron::{Chain, ChainBuilder, Iron, Request, Response, IronResult};
use iron::status;
use router::{Router, Params};
use logger::Logger;

// Serves a string to the user.  Try accessing "/".
fn hello(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with(status::Ok, "Hello world!"))
}

// Serves a customized string to the user.  Try accessing "/world".
fn hello_name(req: &mut Request) -> IronResult<Response> {
    let params = req.extensions.find::<Router,Params>().unwrap();
    let name = params.find("name").unwrap();
    Ok(Response::with(status::Ok, format!("Hello, {}!", name)))
}

/// Look up our server port number in PORT, for compatibility with Heroku.
fn get_server_port() -> Port {
    getenv("PORT")
        .and_then(|s| from_str::<Port>(s.as_slice()))
        .unwrap_or(8080)
}

/// Configure and run our server.
fn main() {
    // Set up our URL router.
    let mut router = Router::new();
    router.get("/", hello);
    router.get("/:name", hello_name);

    // Create a chain with our router and a logger.
    let mut chain = ChainBuilder::new(router);
    let (logger_before, logger_after) = Logger::new(None);
    chain.link_before(logger_before);
    chain.link_after(logger_after);

    // Run the server.
    Iron::new(chain).listen(Ipv4Addr(0, 0, 0, 0), get_server_port());
}
