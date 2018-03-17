use facebook_app::Bot;
use receive::{MessageEntry, StringFuture};

pub fn handle_message(bot: &Bot, message: &MessageEntry) -> StringFuture {
    let text = &message.message.text;
    let sender = &message.sender.id;
    bot.send_text_message(sender, text)
}
