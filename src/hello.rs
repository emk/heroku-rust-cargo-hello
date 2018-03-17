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

use futures::{Future, Stream};
use futures::future;

use gotham::router::Router;
use gotham::router::builder::{build_simple_router, DefineSingleRoute, DrawRoutes};

use hyper::{Post, StatusCode};
use hyper::server::{Http, Request, Response, Service};

use rmessenger::bot::Bot;

use std::env;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use tokio_core::reactor::{Core, Handle};

mod verification;
mod receive;
mod echo_handler;
mod send;
use self::receive::MessengerFuture;

#[derive(Clone)]
struct MessengerService {
    handle: Handle,
    bot: Bot,
    webhook_verify_token: String,
}

impl MessengerService {
    fn new(handle: &Handle) -> Self {
        let bot = send::get_bot(handle.clone());
        let webhook_verify_token = env::var("WEBHOOK_VERIFY_TOKEN").unwrap_or(String::new());
        Self {
            handle: handle.clone(),
            bot: bot.clone(),
            webhook_verify_token: webhook_verify_token,
        }
    }

    fn handle_webhook_post(&self, req: Request) -> MessengerFuture {
        let bot = self.bot.clone();
        let body_fut = req.body().concat2();
        let response_fut = body_fut.and_then(move |body| receive::handle_webhook_body(&bot, &body));
        Box::new(response_fut)
    }
}

impl Service for MessengerService {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = MessengerFuture;

    fn call(&self, req: Request) -> Self::Future {
        let resp_fut: Self::Future = match (req.method(), req.path()) {
            (&Post, "/webhook") => self.handle_webhook_post(req),
            _ => Box::new(future::ok(
                Response::new().with_status(StatusCode::NotFound),
            )),
        };

        let resp = resp_fut.or_else(|err| {
            let mut res = Response::new();
            let body = format!("Something went wrong: {:?}", err);
            res.set_status(StatusCode::InternalServerError);
            res = res.with_body(body);
            println!("translating error");
            Ok::<_, hyper::Error>(res)
        });
        Box::new(resp)
    }
}

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

    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let client_handle = core.handle();

    gotham::start(addr, router());

    let serve = Http::new()
        .serve_addr_handle(&addr, &handle, move || {
            Ok(MessengerService::new(&client_handle))
        })
        .unwrap();
    println!(
        "Listening on http://{}...",
        serve.incoming_ref().local_addr()
    );

    let h2 = handle.clone();
    handle.spawn(
        serve
            .for_each(move |conn| {
                h2.spawn(
                    conn.map(|_| ())
                        .map_err(|err| println!("serve error: {:?}", err)),
                );
                Ok(())
            })
            .map_err(|_| ()),
    );

    core.run(future::empty::<(), ()>()).unwrap();
}
