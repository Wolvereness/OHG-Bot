use serenity::{
    async_trait,
    prelude::*,
    model::prelude::*,
    framework::standard::{
        Args,
        CommandResult,
        macros::{
            command,
            group
        },
    },
};
use futures::{
    stream::TryStreamExt,
    join,
};
use wither::bson::doc;
use wither::prelude::*;
use crate::models::{DatabaseHandle, RoleAssociation};

#[group]
#[commands(ping, join, register_role)]
pub struct General;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

#[command]
async fn join(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.len() > 1 {
        msg.reply(ctx, "No spaces in the name of the group to join").await?;
        return Ok(());
    }
    let Context {
        cache,
        http,
        data,
        ..
    } = ctx;
    let guild = msg.guild(cache);
    let channel = msg.channel(cache);
    let (guild, channel): (Guild, GuildChannel) = match join!(guild, channel) {
        (Some(guild), Some(Channel::Guild(channel))) => (guild, channel),
        _ => {
            msg.reply(http, "Sorry, I'm not smart enough to figure out which server you're asking about. Try sending the message in the server.").await?;
            return Ok(());
        },
    };
    let typing = channel.broadcast_typing(http);
    let db = data.read();
    let (typing, db) = join!(typing, db);
    typing?;
    let db = db
        .get::<DatabaseHandle>()
        .ok_or("Database not present")?;

    if args.is_empty() {
        let associations: std::result::Result<Vec<RoleAssociation>, _> = RoleAssociation::find(
            db,
            Some(doc!{
                "channel": doc!{ "$eq": &format!("{}", channel.id) },
                "server": doc!{ "$eq": &format!("{}", guild.id) },
            }),
            None,
        )
            .await?
            .try_collect()
            .await;
        let associations = associations?;
    }

    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

#[command]
async fn leave(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let db = ctx.data
        .read()
        .await
        .get::<DatabaseHandle>()
        .ok_or("Database not present")?;

    Ok(())
}


#[command]
#[required_permissions("ADMINISTRATOR")]
async fn register_role(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let db = ctx.data
        .read()
        .await
        .get::<DatabaseHandle>()
        .ok_or("Database not present")?;

    Ok(())
}
