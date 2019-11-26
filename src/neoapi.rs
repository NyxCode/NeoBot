use rand::Rng;
use rhai::{Engine, Any};
use rhai::RegisterFn;
use serenity::builder::CreateMessage;
use serenity::client::Context;
use serenity::model::channel::{Message, ReactionType};
use serenity::model::guild::Member;
use serenity::model::id::{GuildId, ChannelId};
use serenity::model::user::User;
use std::fmt::Display;

pub fn register_functionality(engine: &mut Engine, ctx: &Context, src_msg: &Message) {
    engine.register_type::<NeoMessage>();
    engine.register_get("content", NeoMessage::content);
    engine.register_get("author", NeoMessage::author);
    engine.register_get("id", NeoMessage::id);
    engine.register_fn("reply", NeoMessage::reply as fn(&mut NeoMessage, String) -> ());
    engine.register_fn("reply", NeoMessage::reply as fn(&mut NeoMessage, bool) -> ());
    engine.register_fn("reply", NeoMessage::reply as fn(&mut NeoMessage, i64) -> ());
    engine.register_fn("react", NeoMessage::react as fn(&mut NeoMessage, String) -> ());
    engine.register_fn("react", NeoMessage::react as fn(&mut NeoMessage, bool) -> ());
    engine.register_fn("react", NeoMessage::react as fn(&mut NeoMessage, i64) -> ());
    engine.register_fn("delete", NeoMessage::delete);

    engine.register_type::<NeoUser>();
    engine.register_get("id", NeoUser::id);
    engine.register_get("name", NeoUser::name);
    engine.register_get("avatar", NeoUser::avatar);
    engine.register_get("is_bot", NeoUser::is_bot);
    engine.register_get("nick", NeoUser::nick);
    engine.register_fn("direct_message", NeoUser::direct_message as fn(&mut NeoUser, String) -> ());
    engine.register_fn("direct_message", NeoUser::direct_message as fn(&mut NeoUser, bool) -> ());
    engine.register_fn("direct_message", NeoUser::direct_message as fn(&mut NeoUser, i64) -> ());

    fn broadcast_fn<M: Display>(ctx: Context, channel_id: ChannelId) -> impl Fn(M) -> () {
        move |msg: M| {
            channel_id.send_message(&ctx, |c| c.content(msg));
        }
    }
    engine.register_fn("broadcast", broadcast_fn::<String>(ctx.clone(), src_msg.channel_id.clone()));
    engine.register_fn("broadcast", broadcast_fn::<bool>(ctx.clone(), src_msg.channel_id.clone()));
    engine.register_fn("broadcast", broadcast_fn::<i64>(ctx.clone(), src_msg.channel_id.clone()));

    engine.register_fn("random", |min: i64, max: i64|
        rand::thread_rng().gen_range(min, max));

    engine.register_fn("str", to_str as fn(i64) -> String);
    engine.register_fn("str", to_str as fn(bool) -> String);
    engine.register_fn("starts_with", |string: String, arg: String| string.starts_with(&arg));
    engine.register_get("uppercase", |string: &mut String| string.to_uppercase().to_owned());
    engine.register_get("lowercase", |string: &mut String| string.to_lowercase().to_owned());
    engine.register_fn("substring", |string: String, start: i64, end: i64|
        string.as_str()[start as usize..end as usize].to_owned());
    engine.register_get("length", |string: &mut String| string.len() as i64);
    engine.register_fn("contains", |string: String, arg: String| string.contains(&arg));

    engine.register_get("length", |vec: &mut Vec<Box<dyn Any>>| vec.len() as i64);

    if let Some(guild) = src_msg.guild_id {
        let ctx = ctx.clone();
        engine.register_fn("find_user", move |name: String| {
            NeoUser::find(&ctx, guild, name).expect("didnt find user!")
        })
    }
}


fn to_str(value: impl ToString) -> String {
    value.to_string()
}

#[derive(Clone)]
pub struct NeoMessage {
    pub msg: Message,
    pub ctx: Context,
}

impl NeoMessage {
    fn id(&mut self) -> u64 {
        self.msg.id.0
    }

    fn reply<M: Display>(&mut self, msg: M) {
        self.msg.reply(&self.ctx, msg.to_string()).unwrap();
    }

    fn content(&mut self) -> String {
        self.msg.content.clone()
    }

    fn author(&mut self) -> NeoUser {
        NeoUser {
            user: self.msg.author.clone(),
            ctx: self.ctx.clone(),
            guild: self.msg.guild_id,
        }
    }

    fn react<M: Display>(&mut self, emoji: M) {
        let reaction = ReactionType::Unicode(emoji.to_string());
        let _ = self.msg.react(&self.ctx, reaction);
    }

    fn delete(&mut self) {
        let _ = self.msg.delete(&self.ctx);
    }
}

#[derive(Clone)]
pub struct NeoUser {
    pub user: User,
    pub ctx: Context,
    pub guild: Option<GuildId>,
}

impl NeoUser {
    fn find(ctx: &Context, guild: GuildId, find: String) -> Option<Self> {
        guild
            .members_iter(&ctx)
            .flatten()
            .map(|m: Member| m.user)
            .filter(|u| {
                let nick = u.read().nick_in(ctx, &guild);
                let name = &u.read().name;
                &find == name || Some(&find) == nick.as_ref()
            })
            .next()
            .map(|user| NeoUser {
                user: user.read().clone(),
                ctx: ctx.clone(),
                guild: Some(guild),
            })
    }

    fn id(&mut self) -> u64 {
        self.user.id.0
    }

    fn name(&mut self) -> String {
        self.user.name.clone()
    }

    fn is_bot(&mut self) -> bool {
        self.user.bot
    }

    fn direct_message<M: Display>(&mut self, msg: M) {
        let _ = self
            .user
            .direct_message(&mut self.ctx, |c| c.content(msg.to_string()));
    }

    fn avatar(&mut self) -> String {
        self.user.face()
    }

    fn nick(&mut self) -> String {
        self.guild
            .and_then(|id| self.user.nick_in(&self.ctx, id))
            .unwrap_or_else(|| self.name())
    }
}
