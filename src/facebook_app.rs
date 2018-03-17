use hyper;
use hyper_tls;
use hyper::client::Request;
use hyper::Post;
use hyper::header::ContentType;
use hyper::mime::APPLICATION_JSON;
use futures::{Future, Stream};
use url::form_urlencoded;
use std::env;
use hyper::client::HttpConnector;
use tokio_core::reactor::Handle;

use receive;
use echo_handler;

type MessageCallback = fn(&Bot, &receive::MessageEntry) -> StringFuture;
pub type StringFuture = Box<Future<Item = String, Error = hyper::Error>>;

pub fn get_app(message_callback: Option<MessageCallback>) -> FacebookApp {
    let access_token = env::var("ACCESS_TOKEN").unwrap_or(String::new());
    let app_secret = env::var("APP_SECRET").unwrap_or(String::new());
    let webhook_verify_token = env::var("WEBHOOK_VERIFY_TOKEN").unwrap_or(String::new());

    FacebookApp {
        app_secret: app_secret.to_string(),
        webhook_verify_token: webhook_verify_token.to_string(),
        access_token: access_token.to_string(),
        message_callback: message_callback,
    }
}

pub struct FacebookApp {
    app_secret: String,
    webhook_verify_token: String,
    // TODO: These things should be different per page.
    access_token: String,
    pub message_callback: Option<MessageCallback>,
}

impl FacebookApp {
    /// Verify the Get query (after the ?) of a webhook verification request
    /// (see https://developers.facebook.com/docs/graph-api/webhooks#setup)
    /// and return either Some(hub.challenge) for you to put in the body of your
    /// response, or None.
    pub fn verify_webhook_query(&self, query: &str) -> Option<String> {
        let mut maybe_hub_challenge = None;
        let mut webhook_verify_token = false;

        for (key, value) in form_urlencoded::parse(query.as_bytes()) {
            println!("verifying {} = {}", key, value);
            println!("self.webhook_verify_token = {}", self.webhook_verify_token);
            if key == "hub.challenge" {
                println!("hub.challenge received");
                maybe_hub_challenge = Some(value.into_owned());
            } else if key == "hub.verify_token" && value == self.webhook_verify_token {
                println!("hub.verify_token passed");
                webhook_verify_token = true;
            }
        }
        if webhook_verify_token {
            return maybe_hub_challenge;
        } else {
            return None;
        }
    }

    pub fn handle_message(&self, handle: &Handle, message: &receive::MessageEntry) -> StringFuture {
        let bot = get_bot(self, handle);
        let callback = self.message_callback
            .unwrap_or(echo_handler::handle_message);

        callback(&bot, message)
    }
}

type HttpsConnector = hyper_tls::HttpsConnector<HttpConnector>;

fn get_http_client(handle: &Handle) -> hyper::Client<HttpsConnector> {
    let client = hyper::Client::configure()
        .connector(hyper_tls::HttpsConnector::new(4, &handle).unwrap())
        .build(&handle);

    client
}

pub fn get_bot(app: &FacebookApp, handle: &Handle) -> Bot {
    Bot::new(
        get_http_client(handle),
        &app.access_token,
        &app.app_secret,
        &app.webhook_verify_token,
    )
}

// TODO: rename this and generally re-work it: it's currently mostly copy-paste
// from my fork of rmessenger.
#[derive(Clone)]
pub struct Bot {
    client: hyper::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>,
    access_token: String,
    app_secret: String,
    webhook_verify_token: String,
    graph_url: String,
}

impl Bot {
    pub fn new(
        client: hyper::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>,
        access_token: &str,
        app_secret: &str,
        webhook_verify_token: &str,
    ) -> Bot {
        Bot {
            access_token: access_token.to_string(),
            app_secret: app_secret.to_string(),
            webhook_verify_token: webhook_verify_token.to_string(),
            graph_url: "https://graph.facebook.com/v2.7".to_string(),
            client: client,
        }
    }

    pub fn send_text_message(&self, recipient_id: &str, message: &str) -> StringFuture {
        let payload = json!({
            "recipient": {"id": recipient_id},
            "message": {"text": message}
        });

        self.send_raw(payload.to_string())
    }

    /// send payload.
    fn send_raw(&self, payload: String) -> StringFuture {
        let request_endpoint = format!("{}{}", self.graph_url, "/me/messages");

        let data = format!("{}{}", "access_token=", self.access_token).to_string();

        self.post(self.client.clone(), request_endpoint, data, payload)
    }

    /// actually make an http post.
    fn post(
        &self,
        client: hyper::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>,
        url: String,
        data: String,
        body: String,
    ) -> StringFuture {
        let request_url = format!("{}{}{}", url, "?", data).parse().unwrap();
        let mut request = Request::new(Post, request_url);
        request.headers_mut().set(ContentType(APPLICATION_JSON));
        request.set_body(body.to_owned());

        let fut = client
            .request(request)
            .and_then(|res| res.body().concat2())
            .map(|c| String::from_utf8(c.to_vec()).unwrap());
        Box::new(fut)
    }
}
