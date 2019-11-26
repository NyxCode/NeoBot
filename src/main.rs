use serenity::client::Client;
use serenity::model::channel::Message;
use serenity::prelude::{Context, EventHandler, TypeMapKey};

type AnyResult<T> = Result<T, Box<dyn Error>>;

use serenity::model::id::{MessageId, GuildId, ChannelId};
use serenity::model::prelude::{MessageUpdateEvent, Reaction, Ready};
use std::collections::HashMap;
use std::env;
use std::error::Error;

use rhai::Any;

use crate::neoapi::NeoMessage;
use crate::script::Script;
use crate::utils::{react_failure, react_success};
use serenity::model::guild::Guild;

mod neoapi;
mod script;
mod utils;

/// https://discordapp.com/api/oauth2/authorize?client_id=557684314112917504&scope=bot&permissions=134740032

struct Handler;

impl EventHandler for Handler {
    fn message(&self, ctx: Context, msg: Message) {
        let guild_id = some_or_return!(msg.guild_id);
        if msg.author.bot {
            return;
        }

        let mut lock = ctx.data.write();
        let scripts = lock.get_mut::<Scripts>().unwrap()
            .entry(msg.channel_id.clone())
            .or_insert_with(|| HashMap::new());

        scripts
            .values_mut()
            .for_each(|script| {
                let args = vec![Box::new(NeoMessage {
                    msg: msg.clone(),
                    ctx: ctx.clone(),
                }) as Box<dyn Any>];
                script.notify(&ctx, "on_message", args);
            });

        if let Some(code) = parse_neo_block(&msg.content) {
            if let Some(script) = Script::new(code, &ctx, msg.clone()) {
                scripts.insert(msg.id.clone(), script);
            }
        }
    }

    //noinspection ALL
    fn message_update(
        &self,
        ctx: Context,
        _: Option<Message>,
        _: Option<Message>,
        event: MessageUpdateEvent,
    ) {
        let mut lock = ctx.data.write();
        let scripts = lock.get_mut::<Scripts>().unwrap()
            .entry(event.channel_id.clone())
            .or_insert_with(|| HashMap::new());

        if let Ok(updated_msg) = event.channel_id.message(&ctx, &event.id) {
            scripts
                .values_mut()
                .for_each(|script| {
                    let args = vec![Box::new(NeoMessage {
                        msg: updated_msg.clone(),
                        ctx: ctx.clone(),
                    }) as Box<dyn Any>];
                    script.notify(&ctx, "on_message_update", args);
                });
        }

        if let Some(new_content) = event.content {
            if let Some(old) = scripts.remove(&event.id) {
                let _ = old.source_msg.delete_reactions(&ctx);
                if let Some(code) = parse_neo_block(&new_content) {
                    if let Some(script) = Script::new(code, &ctx, old.source_msg) {
                        scripts.insert(event.id, script);
                    }
                }
            }
        }
    }

    fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        if reaction
            .user_id
            .to_user(&ctx)
            .map(|user| user.bot)
            .unwrap_or(true)
        {
            return;
        }

        let mut lock = ctx.data.write();
        let scripts = lock.get_mut::<Scripts>().unwrap()
            .entry(reaction.channel_id.clone())
            .or_insert_with(|| HashMap::new());

        if let Some(script) = scripts.get_mut(&reaction.message_id) {
            if reaction.user_id == script.source_msg.author.id {
                if reaction.emoji == react_success() {
                    script.set_status(false, &ctx);
                } else if reaction.emoji == react_failure() {
                    script.set_status(true, &ctx);
                }
            }
        }
    }



    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

struct Scripts;

impl TypeMapKey for Scripts {
    type Value = HashMap<ChannelId, HashMap<MessageId, Script>>;
}

fn main() -> AnyResult<()> {
    pretty_env_logger::init();
    dotenv::dotenv()?;
    let mut client =
        Client::new(env::var("DISCORD_API_TOKEN")?, Handler).expect("Error creating client");

    client.data.write().insert::<Scripts>(HashMap::new());

    // start listening for events by starting a single shard
    if let Err(why) = client.start() {
        println!("An error occurred while running the client: {:?}", why);
    }

    Ok(())
}

fn parse_neo_block(msg: &str) -> Option<&str> {
    const PREFIX: &str = "```neo";
    const SUFFIX: &str = "```";

    let msg = msg.trim();
    if msg.starts_with(PREFIX) && msg.ends_with(SUFFIX) {
        let start = PREFIX.len();
        let end = msg.len() - SUFFIX.len();
        Some(&msg[start..end])
    } else {
        None
    }
}
