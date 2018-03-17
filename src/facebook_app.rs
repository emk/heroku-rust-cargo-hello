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
    println!("WEBHOOK_VERIFY_TOKEN: {}", webhook_verify_token);
    Bot::new(
        get_http_client(handle),
        &access_token,
        &app_secret,
        &webhook_verify_token,
    )
}

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

    pub fn send_text_message(&self, recipient_id: &str, message: &str) -> receive::StringFuture {
        let payload = json!({
            "recipient": {"id": recipient_id},
            "message": {"text": message}
        });

        self.send_raw(payload.to_string())
    }

    /// send payload.
    fn send_raw(&self, payload: String) -> receive::StringFuture {
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
    ) -> receive::StringFuture {
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
