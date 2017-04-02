#![deny(warnings)]
extern crate hyper;
extern crate futures;
extern crate futures_cpupool;
extern crate pretty_env_logger;
extern crate r2d2;
extern crate r2d2_redis;
extern crate redis;
extern crate rmessenger;

use std::env;
use std::net::{SocketAddr, SocketAddrV4, Ipv4Addr};
use futures::{Future, Stream};
use futures::future::BoxFuture;
use futures_cpupool::CpuPool;

use hyper::{Get, Post, StatusCode};
use hyper::StatusCode::InternalServerError;
use hyper::header::ContentLength;
use hyper::server::{Http, Service, Request, Response};

use r2d2_redis::RedisConnectionManager;
use redis::Commands;
use rmessenger::bot::Bot;

#[derive(Clone)]
struct Echo {
    thread_pool: CpuPool,
    redis_pool: r2d2::Pool<RedisConnectionManager>,
    bot: Bot,
}

fn make_error(string: String) -> hyper::Error {
    println!("error: {}", string);
    hyper::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, string))
}

impl Echo {
    fn handle_get(&self, req: Request) -> BoxFuture<Response, hyper::Error> {
        let other_self = self.clone();
        self.thread_pool
            .spawn_fn(move || {
                // FIXME: There's got to be a more elegant way to translate these error types.
                let conn = match other_self.redis_pool.get() {
                    Ok(v) => v,
                    Err(e) => return Err(make_error(format!("{}", e))),
                };
                let query = req.query().unwrap_or("Nothing");
                let last: String = conn.get("last_response")
                    .unwrap_or("You're the first!".to_string());
                let ret: String = conn.set("last_response", query)
                    .unwrap_or("DB ERROR".to_string());
                let body = format!("Last person said: {} You said: {}. Got back: {}",
                                   last,
                                   query,
                                   ret);
                let res = Response::new()
                    .with_header(ContentLength((body.len() as u64)))
                    .with_body(body);
                Ok(res)
            })
            .boxed()
    }

    fn handle_post(&self, req: Request) -> BoxFuture<Response, hyper::Error> {
        let mut res = Response::new();
        if let Some(len) = req.headers().get::<ContentLength>() {
            res.headers_mut().set(len.clone());
        }
        res = res.with_body(req.body());
        Box::new(futures::future::ok(res))
    }

    fn handle_webhook_verification(&self, req: Request) -> BoxFuture<Response, hyper::Error> {
        let mut res = Response::new();
        println!("got webhook verification {:?}", &req);

        let query = req.query().unwrap_or(&"");
        let hub_challenge = self.bot.verify_webhook_query(query);

        match hub_challenge {
            Some(token) => {
                res = res.with_header(ContentLength((token.len() as u64)));
                res = res.with_body(token);
                println!("returning success");
                return Box::new(futures::future::ok(res));
            }
            None => {
                let msg = format!("Incorrect webhook_verify_token or No hub.challenge in {}",
                                  req.uri().as_ref());
                return Box::new(futures::future::err(make_error(msg)));
            }
        }

    }

    fn handle_webhook_post(&self, req: Request) -> BoxFuture<Response, hyper::Error> {
        let body_fut = req.body()
            .fold(Vec::new(), |mut acc, chunk| {
                acc.extend_from_slice(&chunk);
                Ok::<_, hyper::Error>(acc)
            });
        let response_fut = body_fut.and_then(|body| {
                                                 println!("got webhook: {:?}", body);
                                                 let mut res = Response::new();
                                                 res = res.with_body(body);
                                                 Ok(res)
                                             });
        Box::new(response_fut)
    }
}

impl Service for Echo {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = BoxFuture<Response, hyper::Error>;

    fn call(&self, req: Request) -> Self::Future {
        let resp_fut: Self::Future = match (req.method(), req.path()) {
            (&Get, "/") | (&Get, "/echo") => self.handle_get(req),
            (&Post, "/echo") => self.handle_post(req),
            (&Get, "/webhook") => self.handle_webhook_verification(req),
            (&Post, "/webhook") => self.handle_webhook_post(req),
            _ => Box::new(futures::future::ok(Response::new().with_status(StatusCode::NotFound))),
        };

        let resp = resp_fut.or_else(|err| {
            let mut res = Response::new();
            let body = format!("Something went wrong: {:?}", err);
            res.set_status(InternalServerError);
            res = res.with_body(body);
            println!("translating error");

            return Ok::<_, hyper::Error>(res);
        });
        Box::new(resp)
    }
}
/// Look up our server port number in PORT, for compatibility with Heroku.
fn get_server_port() -> u16 {
    let port_str = env::var("PORT").unwrap_or(String::new());
    port_str.parse().unwrap_or(8080)
}

fn get_redis_pool() -> r2d2::Pool<RedisConnectionManager> {
    let url = env::var("REDIS_URL").unwrap_or(String::new());
    let config = Default::default();
    let manager = RedisConnectionManager::new(url.as_str()).unwrap();
    let redis_pool = r2d2::Pool::new(config, manager).unwrap();
    redis_pool
}

fn get_bot() -> Bot {
    let access_token = env::var("ACCESS_TOKEN").unwrap_or(String::new());
    let app_secret = env::var("APP_SECRET").unwrap_or(String::new());
    let webhook_verify_token = env::var("WEBHOOK_VERIFY_TOKEN").unwrap_or(String::new());
    Bot::new(&access_token, &app_secret, &webhook_verify_token)
}

fn main() {
    pretty_env_logger::init().unwrap();
    // There has got to be a better way specify an ip address.
    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), get_server_port()));

    let thread_pool = CpuPool::new(10);
    let redis_pool = get_redis_pool();
    let bot = get_bot();
    let server = Http::new()
        .bind(&addr, move || {
            Ok(Echo {
                   thread_pool: thread_pool.clone(),
                   redis_pool: redis_pool.clone(),
                   bot: bot.clone(),
               })
        })
        .unwrap();

    println!("Listening on http://{} with 1 thread.",
             server.local_addr().unwrap());

    server.run().unwrap();
}
