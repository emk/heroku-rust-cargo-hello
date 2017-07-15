extern crate iron;
extern crate router;

use std::env;
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
fn get_server_port() -> u16 {
    env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8080)
}

/// Configure and run our server.
fn main() {
    // Set up our URL router.
    let mut router: Router = Router::new();
    router.get("/", hello, "index");
    router.get("/:name", hello_name, "name");

    // Run the server.
    Iron::new(router).http(("0.0.0.0", get_server_port())).unwrap();
}
