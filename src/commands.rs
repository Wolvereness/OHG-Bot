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
    join,
};
use crate::{
    models::{
        DatabaseHandle,
        RoleAssociation,
    },
    util::Mentionable,
};
use std::fmt::Write as _;
use wither::mongodb::Database;
use crate::util::get_role_associations;
use std::result::Result;

#[group]
#[commands(ping, join, register_role, parrot, dump_associations)]
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

    let guild = msg.guild_id.ok_or("No guild?")?;
    let typing = msg.channel_id.broadcast_typing(ctx);
    let db = ctx.data.read();
    let (typing, db) = join!(typing, db);
    let (_, db): (_, &Database) = (
        typing?,
        db
            .get::<DatabaseHandle>()
            .ok_or("Database not present")?,
    );

    if args.is_empty() {
        let member = guild.member(ctx, &msg.author.id);
        let associations = get_role_associations(db, msg.channel_id, guild);
        let (member, associations): (Result<Member, _>, Result<Vec<RoleAssociation>, _>) = join!(member, associations);
        let (member, associations) = (member?, &associations?);

        async fn apply_role(ctx: &Context, channel: ChannelId, role: RoleId, mut member: Member, specific: bool) -> CommandResult {
            member.add_role(ctx, role).await?;
            channel.send_message(ctx, |messages| messages
                .embed(|e| {
                    let e = e.title("Join command:");
                    if specific {
                        e.description(format_args!(
                            "{} has joined {} for {}.",
                            Mentionable::from(member.user.id),
                            Mentionable::from(role),
                            Mentionable::from(channel),
                        ))
                    } else {
                        e.description(format_args!(
                            "{} has joined {}.",
                            Mentionable::from(member.user.id),
                            Mentionable::from(role),
                        ))
                    }
                })
            ).await?;
            Ok(())
        }

        for association in associations {
            if association.channel.is_some() {
                return apply_role(ctx, msg.channel_id, association.role, member, true).await;
            }
        }
        for association in associations {
            if association.server.is_some() {
                return apply_role(ctx, msg.channel_id, association.role, member, false).await;
            }
        }
        msg.channel_id.send_message(ctx, |messages| messages.embed(|e| e
            .title("No groups found!")
            .description(format_args!("No group configured for {}.\nNo generic group configured for the server.", Mentionable::from(msg.channel_id)))
        )).await?;
    } else {
        msg.channel_id.send_message(ctx, |messages| messages.embed(|e| e
            .title("I can't do that yet!")
            .description("At some future date, I'll be able to find the group you want.\n\nFor now, please try to join in the respective channel.")
        )).await?;
    }

    Ok(())
}

#[command]
#[only_in("guild")]
async fn dump_associations(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild_id.ok_or("No guild present")?;
    let typing = msg.channel_id.broadcast_typing(ctx);
    let db = ctx.data.read();
    let (typing, db) = join!(typing, db);
    let (_, db): (_, &Database) = (
        typing?,
        db
            .get::<DatabaseHandle>()
            .ok_or("Database not present")?,
    );
    let associations = get_role_associations(db,msg.channel_id, guild).await?;

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
        message.embed(|e| e
            .title("Association Dump:")
            .description(description)
        )
    }).await?;

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
