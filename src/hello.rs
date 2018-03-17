extern crate futures;
extern crate gotham;
extern crate hyper;
extern crate hyper_tls;
extern crate mime;
extern crate pretty_env_logger;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate tokio_core;
extern crate url;

use gotham::router::Router;
use gotham::router::builder::{build_simple_router, DefineSingleRoute, DrawRoutes};
use hyper::Method;

use std::env;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

mod verification;
mod receive;
mod echo_handler;
mod facebook_app;

/// Look up our server port number in PORT, for compatibility with Heroku.
fn get_server_port() -> u16 {
    let port_str = env::var("PORT").unwrap_or(String::new());
    port_str.parse().unwrap_or(8080)
}

fn router() -> Router {
    build_simple_router(|route| {
        let app = facebook_app::get_app();
        route
            .request(vec![Method::Get, Method::Post], "/webhook")
            .to_new_handler(app);
    })
}

fn main() {
    pretty_env_logger::init();

    let addr = SocketAddr::V4(SocketAddrV4::new(
        Ipv4Addr::new(0, 0, 0, 0),
        get_server_port(),
    ));

    gotham::start(addr, router());
}
