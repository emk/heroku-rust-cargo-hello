extern crate futures;
extern crate gotham;
extern crate hyper;
extern crate hyper_tls;
extern crate mime;
extern crate pretty_env_logger;
extern crate rmessenger;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tokio_core;
extern crate url;

use gotham::router::Router;
use gotham::router::builder::{build_simple_router, DefineSingleRoute, DrawRoutes};

use std::env;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

mod verification;
mod receive;
mod echo_handler;
mod send;

/// Look up our server port number in PORT, for compatibility with Heroku.
fn get_server_port() -> u16 {
    let port_str = env::var("PORT").unwrap_or(String::new());
    port_str.parse().unwrap_or(8080)
}

// self::verification::handle_verification(req, &self.webhook_verify_token)

fn router() -> Router {
    build_simple_router(|route| {
        route
            .get("/webhook")
            .to(self::verification::handle_verification);
        route
            .post("/webhook")
            .to(self::receive::handle_webhook_post);
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
