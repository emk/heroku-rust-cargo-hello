use facebook_app::{Bot, StringFuture};
use receive::MessageEntry;

pub fn echo_message(bot: &Bot, message: &MessageEntry) -> StringFuture {
    let text = &message.message.text;
    let sender = &message.sender.id;
    bot.send_text_message(sender, text)
}

pub fn echo_message_with_prefix(bot: &Bot, message: &MessageEntry) -> StringFuture {
    let text = format!("you said: {}", message.message.text);
    let sender = &message.sender.id;
    bot.send_text_message(sender, &text)
}
