use crate::facebook_app::{Bot, BoxedStringFuture};
use crate::receive::MessageEntry;

pub fn echo_message(bot: &Bot, message: &MessageEntry) -> BoxedStringFuture {
    let text = &message.message.text;
    let sender = &message.sender.id;
    bot.send_text_message(sender, text)
}

pub fn echo_message_with_prefix(bot: &Bot, message: &MessageEntry) -> BoxedStringFuture {
    let text = format!("you said: {}", message.message.text);
    let sender = &message.sender.id;
    bot.send_text_message(sender, &text)
}
