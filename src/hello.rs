extern crate futures;
extern crate gotham;
extern crate hyper;
extern crate hyper_tls;
extern crate mime;
extern crate pretty_env_logger;
extern crate rmessenger;
#[macro_use]
extern crate serde_derive;
#[macro_use]
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
mod facebook_app;

/// Look up our server port number in PORT, for compatibility with Heroku.
fn get_server_port() -> u16 {
    let port_str = env::var("PORT").unwrap_or(String::new());
    port_str.parse().unwrap_or(8080)
}

/// TODO:
/// Currently, we call send::get_bot() every time we handle a request, to
/// construct a bot out of global environment variables. We also hard-code
/// echo_handler::handle_message() as the callback in
/// receive::handle_webhook_payload(). This is all terrible. Here are a few\
/// ideas of things that we could do to make this more felxible:
///
/// * Make Bot not need a Handle on init, so that we can construct one in main()
///   and attach it to the handlers that we use in Router. This will allow us to
///   run multiple facebook apps from the same server.
/// * It should be possible to attach the same app to multiple pages, by
///   providing multiple access tokens. It might be sensible to create a
///   FacebookApp struct that holds an APP_SECRET, a WEBHOOK_VERIFY_TOKEN, and
///   a mapping from PAGE_ID to ACCESS_TOKEN.
/// * Make FacebookApp expose get and post callbacks (that implement
///   gotham::handler::Handler) for easy configuration.
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
