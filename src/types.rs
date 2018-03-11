extern crate futures;

use futures::Future;
use hyper;
use hyper::server::Response;

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
