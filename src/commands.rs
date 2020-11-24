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
use crate::{
    models::{
        DatabaseHandle,
        RoleAssociation,
        Shim,
    },
    util::Mentionable,
};
use std::fmt::Write as _;
use wither::mongodb::Database;

#[group]
#[commands(ping, join, register_role, parrot)]
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
async fn parrot(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if msg.guild_id.is_none() {
        return Ok(());
    }
    let mut description = String::new();
    let mut ix = 0;
    while !args.is_empty() {
        ix += 1;
        writeln!(&mut description, "{}: {:?}", ix, args.current().unwrap())?;
        args.advance();
    }
    let ix: usize = ix;

    msg.channel_id.send_message(ctx, |message| message
        .embed(|e| e
            .title("I'm a parrot!")
            .description(description)
            .footer(|f| f
                .text(format!("FIN! {}", ix))
            )
        )
    ).await?;

    Ok(())
}

#[command]
#[only_in("guild")]
async fn join(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.len() > 1 {
        msg.reply(ctx, "No spaces in the name of the group to join").await?;
        return Ok(());
    }

    let guild = msg.guild(ctx);
    let typing = msg.channel_id.broadcast_typing(ctx);
    let db = ctx.data.read();
    let (guild, typing, db) = join!(guild, typing, db);
    let (guild, _, db): (Guild, _, &Database) = (
        guild.ok_or("No guild?!")?,
        typing?,
        db
            .get::<DatabaseHandle>()
            .ok_or("Database not present")?,
    );

    if args.is_empty() {
        let associations: std::result::Result<Vec<RoleAssociation>, _> = RoleAssociation::find(
            db,
            Some(doc!{
                "$or": [
                    { "channel": doc!{ "$eq": &Shim::from(msg.channel_id) } },
                    { "server": doc!{ "$eq": &Shim::from(guild.id) } },
                ],
            }),
            None,
        )
            .await?
            .try_collect()
            .await;
        let associations = associations?;

        let mut description = "```\n".to_string();
        for (ix, association) in associations.iter().enumerate() {
            writeln!(&mut description, "{}: {:?}", ix, association)?;
        }
        description.push_str("```\n\n");
        for (ix, association) in associations.iter().enumerate() {
            match association {
                &RoleAssociation {
                    channel: Some(channel),
                    server: None,
                    role,
                    ..
                } => writeln!(
                    &mut description,
                    "{}: {} in {}",
                    ix,
                    Mentionable::from(role),
                    Mentionable::from(channel),
                )?,
                &RoleAssociation {
                    channel: None,
                    server: Some(server),
                    role,
                    ..
                } => writeln!(
                    &mut description,
                    "{}: {} in {:?}",
                    ix,
                    Mentionable::from(role),
                    server,
                )?,
                _ => writeln!(
                    &mut description,
                    "{} ERR: {:?}",
                    ix,
                    association,
                )?,
            }
        }

        msg.channel_id.send_message(ctx, |message| {
            message.embed(|mut e| e
                .title("Association Dump:")
                .description(description)
            )
        }).await?;
    }

    Ok(())
}

#[command]
#[only_in("guild")]
async fn leave(ctx: &Context, _msg: &Message, _args: Args) -> CommandResult {
    let _db = ctx.data
        .read()
        .await
        .get::<DatabaseHandle>()
        .ok_or("Database not present")?;

    Ok(())
}


#[command]
#[only_in("guild")]
#[required_permissions("ADMINISTRATOR")]
async fn register_role(ctx: &Context, _msg: &Message, _args: Args) -> CommandResult {
    let _db = ctx.data
        .read()
        .await
        .get::<DatabaseHandle>()
        .ok_or("Database not present")?;

    Ok(())
}
