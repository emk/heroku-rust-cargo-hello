extern crate futures;

use futures::{future, Future, Stream};
use gotham::handler::{HandlerFuture, IntoHandlerError};
use gotham::http::response::create_response;
use gotham::state::{FromState, State};
use hyper;
use hyper::{Body, Response, StatusCode};
use serde_json;
use tokio_core::reactor::Handle;

use mime;

use crate::facebook_app::FacebookApp;

pub type MessengerFuture = Box<Future<Item = Response, Error = hyper::Error>>;

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
pub struct WebhookPayload {
    pub entry: Vec<WebhookEntry>,
    pub object: String,
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
pub struct WebhookEntry {
    pub id: String,
    pub messaging: Vec<MessageEntry>,
    pub time: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageEntry {
    pub message: MessageDetailsEntry,
    pub recipient: AuthorEntry,
    pub sender: AuthorEntry,
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageDetailsEntry {
    pub mid: String,
    pub seq: i64,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthorEntry {
    pub id: String,
}

pub fn handle_webhook_payload(
    app: &FacebookApp,
    handle: &Handle,
    payload: WebhookPayload,
) -> MessengerFuture {
    let mut message_futures = Vec::new();
    for entry in &payload.entry {
        for message in &entry.messaging {
            let f = app.handle_message(handle, message);
            message_futures.push(f);
        }
    }
    let joined_futures = future::join_all(message_futures);

    let response_future = joined_futures.and_then(move |v| {
        println!("message sending done: {:?}", v);

        let mut res = Response::new();
        res = res.with_body(serde_json::to_string(&payload).unwrap_or_default());
        Ok(res)
    });
    Box::new(response_future)
}

pub fn handle_webhook_body(app: &FacebookApp, handle: &Handle, body: &[u8]) -> MessengerFuture {
    let payload: WebhookPayload = serde_json::from_slice(body).unwrap_or_default();
    println!("got payload: {:?}", payload);
    handle_webhook_payload(&app, handle, payload)
}

pub fn handle_webhook_post(mut state: State, app: FacebookApp) -> Box<HandlerFuture> {
    let handle = Handle::borrow_from(&state).clone();
    // FIXME: make the FacebookApp once in main() and pluck it out here.

    let f = Body::take_from(&mut state)
        .concat2()
        .and_then(move |body| handle_webhook_body(&app, &handle, &body));

    Box::new(f.then(move |result| match result {
        Ok(_) => {
            let res = create_response(
                &state,
                StatusCode::Ok,
                Some((b"".to_vec(), mime::TEXT_PLAIN)),
            );
            Ok((state, res))
        }
        Err(err) => Err((state, err.into_handler_error())),
    }))
}
