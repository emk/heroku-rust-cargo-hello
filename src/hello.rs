#![deny(warnings)]
extern crate hyper;
extern crate futures;
extern crate pretty_env_logger;
//extern crate num_cpus;
use std::env;
use std::net::{SocketAddr, SocketAddrV4, Ipv4Addr};
use futures::future::FutureResult;

use hyper::{Get, Post, StatusCode};
use hyper::header::ContentLength;
use hyper::server::{Http, Service, Request, Response};

static INDEX: &'static [u8] = b"Try POST /echo";

#[derive(Clone, Copy)]
struct Echo;

fn handle_get(_req: Request) -> Response {
    Response::new()
        .with_header(ContentLength(INDEX.len() as u64))
        .with_body(INDEX)
}

fn handle_post(req: Request) -> Response {
    let mut res = Response::new();
    if let Some(len) = req.headers().get::<ContentLength>() {
        res.headers_mut().set(len.clone());
    }
    res.with_body(req.body())
}

impl Service for Echo {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = FutureResult<Response, hyper::Error>;

    fn call(&self, req: Request) -> Self::Future {
        let resp = match (req.method(), req.path()) {
            (&Get, "/") | (&Get, "/echo") => handle_get(req),
            (&Post, "/echo") => handle_post(req),
            _ => Response::new().with_status(StatusCode::NotFound),
        };
        futures::future::ok(resp)
    }
}
/// Look up our server port number in PORT, for compatibility with Heroku.
fn get_server_port() -> u16 {
    let port_str = env::var("PORT").unwrap_or(String::new());
    port_str.parse().unwrap_or(8080)
}

fn main() {
    pretty_env_logger::init().unwrap();
    // There has got to be a better way specify an ip address.
    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), get_server_port()));
    let server = Http::new().bind(&addr, || Ok(Echo)).unwrap();

    println!("Listening on http://{} with 1 thread.",
             server.local_addr().unwrap());

    server.run().unwrap();
}