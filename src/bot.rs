use serenity::all::CreateAttachment;
use serenity::async_trait;
use serenity::builder::CreateMessage;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::ChannelId;
use serenity::prelude::*;

use crate::ezgen;
use bytes::Bytes;

pub struct Bot {
    pub ezgen_api_key: String,
}

const REACTION_OK: char = '\u{1F44D}';
const REACTION_FAILED: char = '\u{274C}';

/// Bot responds to messages by interpreting them as an image prompt for the FLUX schnell model.
/// There are two cases it will respond:
///
/// - In a DM, the entire message is taken as the prompt
/// - When @mentioned, the prompt is the message with @mention stripped.
#[async_trait]
impl EventHandler for Bot {
    // Handle messages from Discord
    async fn message(&self, ctx: Context, msg: Message) {
        // Log received messages to stdout
        println!(
            "msg #<{}> @{}: {}",
            msg.channel_id
                .name(&ctx)
                .await
                .unwrap_or_else(|_| "Unknown".to_string()),
            msg.author.name,
            msg.content,
        );

        // Only respond if it's a DM, or if bot is explicitly mentioned.
        let is_dm = msg.guild_id.is_none();
        let mentions_me = msg.mentions_me(&ctx.http).await.unwrap_or(false);
        let is_me = msg.author == **ctx.cache.current_user();
        if is_me || !(is_dm || mentions_me) {
            return;
        }

        // React to the message with a thumbs up to show we're working.
        if let Err(why) = msg.react(&ctx.http, REACTION_OK).await {
            println!("Error reacting to message: {:?}", why);
        }

        // Get prompt by removing the @mention, if present.
        let bot_id = ctx.cache.current_user().id;
        let prompt = msg
            .content
            .replace(&format!("<@{}>", bot_id), "")
            .trim()
            .to_string();

        // Create an image with the prompt, and send it back to the user.
        // If image creation fails, react to the request with a red cross.
        println!("creating image with prompt: {:?}", prompt);
        match ezgen::get_image(&self.ezgen_api_key, &prompt).await {
            // If we got an image, send it back in a reply to the user
            Ok(bytes) => {
                if let Err(why) =
                    send_image_message_reply(&ctx, msg.channel_id, "Here you go!", &bytes, &msg)
                        .await
                {
                    println!("Error sending message: {:?}", why);
                }
            }
            // Otherwise, reply with the error we got
            Err(why) => {
                let message = match why {
                    ezgen::Error::Reqwest(err) => format!("Couldn't connect to API: {:?}", err),
                    ezgen::Error::Ezgen(err) => format!("API error: {:?}", err.error),
                };
                let _ = msg.react(&ctx.http, REACTION_FAILED).await;
                let _ = msg.reply(&ctx.http, message).await;
            }
        }
    }

    // Set a handler for the `ready` event, which fires when the bot is ready to start working
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

/// Send a message with an image attachment as a reply to the given message
async fn send_image_message_reply(
    ctx: &Context,
    channel_id: ChannelId,
    message: &str,
    data: &Bytes,
    reply_to: &Message,
) -> Result<Message, SerenityError> {
    let attachment = CreateAttachment::bytes(data.to_vec(), "image0.webp");
    let builder = CreateMessage::new()
        .content(message)
        .add_file(attachment)
        .reference_message(reply_to);
    channel_id.send_message(&ctx.http, builder).await
}
