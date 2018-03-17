use gotham::handler::{Handler, HandlerFuture, NewHandler};
use gotham::http::response::create_response;
use gotham::state::State;
use gotham::state::FromState;
use hyper;
use hyper_tls;
use hyper::client::Request;
use hyper::{Method, StatusCode};
use hyper::header::ContentType;
use hyper::mime::APPLICATION_JSON;
use std::io;
use futures::{future, Future, Stream};
use url::form_urlencoded;
use std::env;
use hyper::client::HttpConnector;
use tokio_core::reactor::Handle;

use receive;
use echo_handler;
use verification;
use std::collections::HashMap;

type MessageCallback = fn(&Bot, &receive::MessageEntry) -> StringFuture;
pub type StringFuture = Box<Future<Item = String, Error = hyper::Error>>;

// TODO: move this into hello.rs... and rename hello.rs
pub fn get_app() -> FacebookApp {
    let app_secret = env::var("APP_SECRET").unwrap_or(String::new());
    let webhook_verify_token = env::var("WEBHOOK_VERIFY_TOKEN").unwrap_or(String::new());

    let mut page_config = HashMap::new();
    page_config.insert(
        env::var("ECHO_PAGE_ID").unwrap_or(String::new()),
        FacebookPage {
            access_token: env::var("ECHO_ACCESS_TOKEN").unwrap_or(String::new()),
            message_callback: Some(echo_handler::echo_message),
        },
    );
    page_config.insert(
        env::var("PREFIX_PAGE_ID").unwrap_or(String::new()),
        FacebookPage {
            access_token: env::var("PREFIX_ACCESS_TOKEN").unwrap_or(String::new()),
            message_callback: Some(echo_handler::echo_message_with_prefix),
        },
    );
    FacebookApp {
        app_secret: app_secret.to_string(),
        webhook_verify_token: webhook_verify_token.to_string(),
        page_config: page_config,
    }
}
#[derive(Clone)]
pub struct FacebookPage {
    access_token: String,
    message_callback: Option<MessageCallback>,
}

#[derive(Clone)]
pub struct FacebookApp {
    app_secret: String,
    webhook_verify_token: String,
    page_config: HashMap<String, FacebookPage>,
}

impl NewHandler for FacebookApp {
    type Instance = Self;
    fn new_handler(&self) -> io::Result<Self::Instance> {
        Ok(self.clone())
    }
}
impl Handler for FacebookApp {
    fn handle(self, state: State) -> Box<HandlerFuture> {
        let method = Method::borrow_from(&state).clone();
        match method {
            Method::Post => receive::handle_webhook_post(state, self),
            Method::Get => verification::handle_verification(state, self),
            _ => {
                let response = create_response(&state, StatusCode::MethodNotAllowed, None);
                Box::new(future::ok((state, response)))
            }
        }
    }
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
        let id = message.recipient.id.clone();
        let mut message_callback = None;
        let mut access_token = None;
        match self.page_config.get(&id) {
            Some(page) => {
                access_token = Some(page.access_token.clone());
                message_callback = page.message_callback;
            }
            None => {
                println!("got webhook for unmanaged page {}", id);
            }
        }
        let bot = Bot::new(
            get_http_client(handle),
            &access_token.unwrap_or("".to_string()),
            &self.app_secret,
            &self.webhook_verify_token,
        );
        let callback = message_callback.unwrap_or(echo_handler::echo_message);
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
        let mut request = Request::new(Method::Post, request_url);
        request.headers_mut().set(ContentType(APPLICATION_JSON));
        request.set_body(body.to_owned());

        let fut = client
            .request(request)
            .and_then(|res| res.body().concat2())
            .map(|c| String::from_utf8(c.to_vec()).unwrap());
        Box::new(fut)
    }
}
