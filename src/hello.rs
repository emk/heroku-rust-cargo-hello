extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate pretty_env_logger;
extern crate rmessenger;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tokio_core;
extern crate url;

use futures::{Future, Stream};
use futures::future;

use hyper::{Get, Post, StatusCode};
use hyper::client::HttpConnector;
use hyper::server::{Http, Request, Response, Service};

use rmessenger::bot::Bot;

use std::env;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use tokio_core::reactor::{Core, Handle};

mod types;
mod verification;
use self::types::{MessengerFuture, WebhookPayload};

#[derive(Clone)]
struct Echo {
    handle: Handle,
    bot: Bot,
    webhook_verify_token: String,
}

type HttpsConnector = hyper_tls::HttpsConnector<HttpConnector>;

impl Echo {
    fn new(handle: &Handle) -> Self {
        let bot = get_bot(handle.clone());
        let webhook_verify_token = env::var("WEBHOOK_VERIFY_TOKEN").unwrap_or(String::new());
        Self {
            handle: handle.clone(),
            bot: bot.clone(),
            webhook_verify_token: webhook_verify_token,
        }
    }

    fn handle_webhook_verification(&self, req: Request) -> MessengerFuture {
        self::verification::handle_verification(req, &self.webhook_verify_token)
    }

    fn handle_webhook_post(&self, req: Request) -> MessengerFuture {
        let bot = self.bot.clone();
        let body_fut = req.body().fold(Vec::new(), |mut acc, chunk| {
            acc.extend_from_slice(&chunk);
            Ok::<_, hyper::Error>(acc)
        });
        let response_fut = body_fut.and_then(move |body| {
            let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or_default();
            println!("got webhook: {}", json.to_string());
            println!("got webhook: {:?}", json);
            let typed_json: WebhookPayload = serde_json::from_slice(&body).unwrap_or_default();
            println!("got typed: {:?}", typed_json);

            let mut message_futures = Vec::new();
            for entry in &typed_json.entry {
                for message in &entry.messaging {
                    let text = &message.message.text;
                    let sender = &message.sender.id;
                    message_futures.push(bot.send_text_message(sender, text));
                }
            }
            let joined_futures = future::join_all(message_futures);

            let response_future = joined_futures.and_then(move |v| {
                println!("message sending done: {:?}", v);

                let mut res = Response::new();
                res = res.with_body(serde_json::to_string(&typed_json).unwrap_or_default());
                Ok(res)
            });
            response_future
        });
        Box::new(response_fut)
    }
}

impl Service for Echo {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = MessengerFuture;

    fn call(&self, req: Request) -> Self::Future {
        let resp_fut: Self::Future = match (req.method(), req.path()) {
            (&Get, "/webhook") => self.handle_webhook_verification(req),
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

fn get_http_client(handle: Handle) -> hyper::Client<HttpsConnector> {
    let client = hyper::Client::configure()
        .connector(hyper_tls::HttpsConnector::new(4, &handle).unwrap())
        .build(&handle);

    client
}

fn get_bot(handle: Handle) -> Bot {
    let access_token = env::var("ACCESS_TOKEN").unwrap_or(String::new());
    let app_secret = env::var("APP_SECRET").unwrap_or(String::new());
    Bot::new(get_http_client(handle), &access_token, &app_secret, "")
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

    let serve = Http::new()
        .serve_addr_handle(&addr, &handle, move || Ok(Echo::new(&client_handle)))
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
