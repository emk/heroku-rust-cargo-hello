#![feature(box_syntax)]
#![feature(conservative_impl_trait)]
#![feature(universal_impl_trait)]
#![feature(dyn_trait)]

extern crate futures;
extern crate futures_cpupool;
extern crate hyper;
extern crate hyper_tls;
extern crate pretty_env_logger;
extern crate r2d2;
extern crate rmessenger;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tokio_core;

use futures::{Future, Stream};
use futures::future;
use futures_cpupool::CpuPool;

use hyper::{Get, Post, StatusCode};
use hyper::client::HttpConnector;
use hyper::header::ContentLength;
use hyper::server::{Http, Request, Response, Service};

use rmessenger::bot::Bot;

use std::env;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use tokio_core::reactor::Handle;

type HttpsConnector = hyper_tls::HttpsConnector<HttpConnector>;

type MessengerFuture = Box<Future<Item = Response, Error = hyper::Error>>;
/*
The following structs are intended to represent the following webhook payload:
Object({
    "entry": Array([
        Object({
            "id": String("971281182990192"),
            "messaging": Array([
                Object({
                    "message": Object({
                        "mid": String("mid.$cAANzYAfQpeBhYL9PMFbL3oG935WY"),
                        "seq": Number(PosInt(4969)),
                        "text": String("ho")
                    }),
                    "recipient": Object({
                        "id": String("971281182990192")
                    }),
                    "sender": Object({
                        "id": String("1249910941788598")
                    }),
                    "timestamp": Number(PosInt(1491150178096))
                })
            ]),
            "time": Number(PosInt(1491150178150))
        })
    ]),
    "object": String("page")
})
*/
#[derive(Serialize, Deserialize, Debug)]
struct WebhookPayload {
    entry: Vec<WebhookEntry>,
    object: String,
}

impl Default for WebhookPayload {
    fn default() -> WebhookPayload {
        WebhookPayload {
            entry: Vec::new(),
            object: String::from("ParseError"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct WebhookEntry {
    id: String,
    messaging: Vec<MessageEntry>,
    time: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct MessageEntry {
    message: MessageDetailsEntry,
    recipient: AuthorEntry,
    sender: AuthorEntry,
    timestamp: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct MessageDetailsEntry {
    mid: String,
    seq: i64,
    text: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct AuthorEntry {
    id: String,
}

#[derive(Clone)]
struct Echo {
    thread_pool: CpuPool,
    handle: Handle,
    bot: Bot,
}

fn make_error(string: String) -> hyper::Error {
    println!("error: {}", string);
    hyper::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, string))
}

impl Echo {
    fn new(handle: &Handle) -> Self {
        let thread_pool = CpuPool::new(10);
        let bot = get_bot(handle.clone());
        Self {
            thread_pool: thread_pool.clone(),
            handle: handle.clone(),
            bot: bot.clone(),
        }
    }

    fn handle_get(&self, _req: Request) -> MessengerFuture {
        box self.thread_pool.spawn_fn(move || {
            let body = format!("Hello world.");
            let res = Response::new()
                .with_header(ContentLength(body.len() as u64))
                .with_body(body);
            Ok(res)
        })
    }

    fn handle_post(&self, req: Request) -> MessengerFuture {
        let mut res = Response::new();
        if let Some(len) = req.headers().get::<ContentLength>() {
            res.headers_mut().set(len.clone());
        }
        res = res.with_body(req.body());
        box future::ok(res)
    }

    fn handle_webhook_verification(&self, req: Request) -> MessengerFuture {
        let mut res = Response::new();
        println!("got webhook verification {:?}", &req);

        let query = req.query().unwrap_or(&"");
        let hub_challenge = self.bot.verify_webhook_query(query);

        match hub_challenge {
            Some(token) => {
                res = res.with_header(ContentLength(token.len() as u64));
                res = res.with_body(token);
                println!("returning success");
                box future::ok(res)
            }
            None => {
                let msg = format!(
                    "Incorrect webhook_verify_token or No hub.challenge in {}",
                    req.uri().as_ref()
                );
                box future::err(make_error(msg))
            }
        }
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
        box response_fut
    }
}

impl Service for Echo {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = MessengerFuture;

    fn call(&self, req: Request) -> Self::Future {
        let resp_fut: Self::Future = match (req.method(), req.path()) {
            (&Get, "/") | (&Get, "/echo") => self.handle_get(req),
            (&Post, "/echo") => self.handle_post(req),
            (&Get, "/webhook") => self.handle_webhook_verification(req),
            (&Post, "/webhook") => self.handle_webhook_post(req),
            _ => box future::ok(Response::new().with_status(StatusCode::NotFound)),
        };

        let resp = resp_fut.or_else(|err| {
            let mut res = Response::new();
            let body = format!("Something went wrong: {:?}", err);
            res.set_status(StatusCode::InternalServerError);
            res = res.with_body(body);
            println!("translating error");
            Ok::<_, hyper::Error>(res)
        });
        box resp
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
    let webhook_verify_token = env::var("WEBHOOK_VERIFY_TOKEN").unwrap_or(String::new());
    Bot::new(
        get_http_client(handle),
        &access_token,
        &app_secret,
        &webhook_verify_token,
    )
}

fn main() {
    pretty_env_logger::init();
    let addr = SocketAddr::V4(SocketAddrV4::new(
        Ipv4Addr::new(0, 0, 0, 0),
        get_server_port(),
    ));

    // let mut core = Core::new().unwrap();
    // let handle: Handle = core.handle();
    // let listener = TcpListener::bind(&addr, &handle).unwrap();
    // let protocol = Http::new();
    // let service = Echo::new(&handle);

    // core.run(listener.incoming().for_each(|(socket, addr)| {
    //     protocol.bind_connection(&handle, socket, addr, service.clone());
    //     Ok(())
    // })).unwrap()

    let mut core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();
    let client_handle = core.handle();

    let serve = Http::new()
        .serve_addr_handle(&addr, &handle, move || Ok(Echo::new(&client_handle)))
        .unwrap();
    println!(
        "Listening on http://{} with 1 thread.",
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

    core.run(futures::future::empty::<(), ()>()).unwrap();
    // let server = Http::new().bind(&addr, || Ok(Echo)).unwrap();
    // println!("Listening on http://{}...", server.local_addr().unwrap());
    // server.run().unwrap();
}
