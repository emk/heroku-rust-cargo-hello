#![deny(warnings)]
extern crate hyper;
extern crate futures;
extern crate futures_cpupool;
extern crate pretty_env_logger;
extern crate r2d2;
extern crate r2d2_redis;
extern crate redis;

use std::env;
use std::net::{SocketAddr, SocketAddrV4, Ipv4Addr};
use futures::Future;
use futures::future::BoxFuture;
use futures_cpupool::CpuPool;

use hyper::{Get, Post, StatusCode};
use hyper::header::ContentLength;
use hyper::server::{Http, Service, Request, Response};

use r2d2_redis::RedisConnectionManager;
use redis::Commands;

#[derive(Clone)]
struct Echo {
    thread_pool: CpuPool,
    redis_pool: r2d2::Pool<RedisConnectionManager>,
}

impl Echo {
    fn handle_get(&self, req: Request) -> BoxFuture<Response, hyper::Error> {
        let other_self = self.clone();
        self.thread_pool
            .spawn_fn(move || {
                // FIXME: There's got to be a more elegant way to translate these error types.
                let conn = match other_self.redis_pool.get() {
                    Ok(v) => v,
                    Err(e) => {
                        return Err(hyper::Error::Io(std::io::Error::new(std::io::ErrorKind::Other,
                                                                        format!("{}", e))))
                    }
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
}

impl Service for Echo {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = BoxFuture<Response, hyper::Error>;

    fn call(&self, req: Request) -> Self::Future {
        let resp = match (req.method(), req.path()) {
            (&Get, "/") | (&Get, "/echo") => self.handle_get(req),
            (&Post, "/echo") => self.handle_post(req),
            _ => Box::new(futures::future::ok(Response::new().with_status(StatusCode::NotFound))),
        };
        resp
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

fn main() {
    pretty_env_logger::init().unwrap();
    // There has got to be a better way specify an ip address.
    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), get_server_port()));

    let thread_pool = CpuPool::new(10);
    let redis_pool = get_redis_pool();
    let server = Http::new()
        .bind(&addr, move || {
            Ok(Echo {
                   thread_pool: thread_pool.clone(),
                   redis_pool: redis_pool.clone(),
               })
        })
        .unwrap();

    println!("Listening on http://{} with 1 thread.",
             server.local_addr().unwrap());

    server.run().unwrap();
}