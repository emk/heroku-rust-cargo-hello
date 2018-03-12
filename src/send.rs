use std::env;

use hyper;
use hyper_tls;
use hyper::client::HttpConnector;
use tokio_core::reactor::Handle;
use rmessenger::bot::Bot;

type HttpsConnector = hyper_tls::HttpsConnector<HttpConnector>;

fn get_http_client(handle: Handle) -> hyper::Client<HttpsConnector> {
    let client = hyper::Client::configure()
        .connector(hyper_tls::HttpsConnector::new(4, &handle).unwrap())
        .build(&handle);

    client
}

pub fn get_bot(handle: Handle) -> Bot {
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
