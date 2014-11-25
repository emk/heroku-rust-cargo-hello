extern crate iron;
extern crate router;

use std::os::getenv;
use std::io::net::ip::{Ipv4Addr, Port};
use iron::{Iron, Request, Response, IronResult, Set};
use iron::response::modifiers::{Status, Body};
use iron::status;
use router::{Router, Params};

// Serves a string to the user.  Try accessing "/".
fn hello(_: &mut Request) -> IronResult<Response> {
    let resp = Response::new()
        .set(Status(status::Ok))
        .set(Body("Hello world!"));
    Ok(resp)
}

// Serves a customized string to the user.  Try accessing "/world".
fn hello_name(req: &mut Request) -> IronResult<Response> {
    let params = req.extensions.get::<Router,Params>().unwrap();
    let name = params.find("name").unwrap();
    let resp = Response::new()
        .set(Status(status::Ok))
        .set(Body(format!("Hello, {}!", name)));
    Ok(resp)
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

    // Run the server.
    match Iron::new(router).listen((Ipv4Addr(0, 0, 0, 0), get_server_port())) {
        Ok(_) => {},
        Err(ref err) => { println!("error starting server: {}", err); }
    }
}
