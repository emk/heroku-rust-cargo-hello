use gotham::http::response::create_response;
use gotham::state::{FromState, State};
use hyper::{Response, StatusCode, Uri};
use hyper::mime::TEXT_PLAIN;
use tokio_core::reactor::Handle;

use send;

pub fn handle_verification(state: State) -> (State, Response) {
    let uri = Uri::borrow_from(&state).clone();
    let handle = Handle::borrow_from(&state).clone();
    let bot = send::get_bot(handle);

    let query = uri.query().unwrap_or(&"");
    let hub_challenge = bot.verify_webhook_query(&query);

    match hub_challenge {
        Some(challenge) => {
            println!("returning success");
            let res = create_response(
                &state,
                StatusCode::Ok,
                Some((challenge.as_bytes().to_vec(), TEXT_PLAIN)),
            );
            (state, res)
        }
        None => {
            let msg = format!(
                "Incorrect webhook_verify_token or No hub.challenge in {}",
                query
            );
            let res = create_response(
                &state,
                StatusCode::BadRequest,
                Some((msg.as_bytes().to_vec(), TEXT_PLAIN)),
            );
            (state, res)
        }
    }
}
