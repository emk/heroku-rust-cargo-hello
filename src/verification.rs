use std;
use futures::future;
use hyper;
use hyper::header::ContentLength;
use hyper::server::{Request, Response};
use url::form_urlencoded;

use receive::MessengerFuture;

fn make_error(string: String) -> hyper::Error {
    println!("error: {}", string);
    hyper::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, string))
}

/// Verify the Get query (after the ?) of a webhook verification request
/// (see https://developers.facebook.com/docs/graph-api/webhooks#setup)
/// and return either Some(hub.challenge) for you to put in the body of your
/// response, or None.
pub fn verify_webhook_query(query: &str, webhook_verify_token: &str) -> Option<String> {
    let mut maybe_hub_challenge = None;
    let mut have_correct_token = false;

    for (key, value) in form_urlencoded::parse(query.as_bytes()) {
        if key == "hub.challenge" {
            println!("hub.challenge received");
            maybe_hub_challenge = Some(value.into_owned());
        } else if key == "hub.verify_token" && value == webhook_verify_token {
            println!("hub.verify_token passed");
            have_correct_token = true;
        }
    }
    if have_correct_token {
        return maybe_hub_challenge;
    } else {
        return None;
    }
}

pub fn handle_verification(req: Request, webhook_verify_token: &str) -> MessengerFuture {
    let mut res = Response::new();
    println!("got webhook verification {:?}", &req);

    let query = req.query().unwrap_or(&"");
    let hub_challenge = verify_webhook_query(query, webhook_verify_token);

    match hub_challenge {
        Some(token) => {
            res = res.with_header(ContentLength(token.len() as u64));
            res = res.with_body(token);
            println!("returning success");
            Box::new(future::ok(res))
        }
        None => {
            let msg = format!(
                "Incorrect webhook_verify_token or No hub.challenge in {}",
                req.uri().as_ref()
            );
            Box::new(future::err(make_error(msg)))
        }
    }
}
