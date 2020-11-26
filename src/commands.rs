use std::{
    result::Result,
    fmt::Write as _,
};
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
use futures::join;
use wither::{
    Model,
    mongodb::Database
};
use tokio::sync::RwLockReadGuard;

use crate::{
    models::{
        DatabaseHandle,
        RoleAssociation,
    },
    util::{
        get_role_associations,
        Mentionable,
    },
};

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
    let reply_reference = MessageReference::from(msg);
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
        .reference_message(reply_reference)
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
    let reply_reference = MessageReference::from(msg);
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

        async fn apply_role(ctx: &Context, channel: ChannelId, role: RoleId, mut member: Member, reply_reference: MessageReference, specific: bool) -> CommandResult {
            member.add_role(ctx, role).await?;
            channel.send_message(ctx, |message| message
                .reference_message(reply_reference)
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
                return apply_role(ctx, msg.channel_id, association.role, member, reply_reference,true).await;
            }
        }
        for association in associations {
            if association.server.is_some() {
                return apply_role(ctx, msg.channel_id, association.role, member, reply_reference,false).await;
            }
        }
        msg.channel_id.send_message(ctx, |message| message
            .reference_message(reply_reference)
            .embed(|e| e
                .title("No groups found!")
                .description(format_args!("No group configured for {}.\nNo generic group configured for the server.", Mentionable::from(msg.channel_id)))
            )
        ).await?;
    } else {
        msg.channel_id.send_message(ctx, |message| message
            .reference_message(reply_reference)
            .embed(|e| e
                .title("I can't do that yet!")
                .description("At some future date, I'll be able to find the group you want.\n\nFor now, please try to join in the respective channel.")
            )
        ).await?;
    }

    Ok(())
}

#[command]
#[only_in("guild")]
async fn dump_associations(ctx: &Context, msg: &Message) -> CommandResult {
    let reply_reference = MessageReference::from(msg);
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

    msg.channel_id.send_message(ctx, |message| message
        .reference_message(reply_reference)
        .embed(|e| e
            .title("Association Dump:")
            .description(description)
        )
    ).await?;

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
async fn register_role(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let reply_reference = MessageReference::from(msg);
    let guild = msg.guild_id.ok_or("No guild present")?;
    async fn bad_message(ctx: &Context, msg: &Message) -> CommandResult {
        const CONTENT: &'static str = "One or two parameters.\nOne may be a reference to the channel.\nOne must be either a reference to the group, or the group ID.";
        msg.reply(ctx, CONTENT).await?;
        return Ok(());
    }
    async fn unamused(ctx: &Context, msg: &Message) -> CommandResult {
        msg.reply(ctx, ":unamused:").await?;
        return Ok(());
    }

    let mut channel: Option<ChannelId> = None;
    let mut role: Option<RoleId> = None;
    while !args.is_empty() {
        match (args.parse(), args.parse()) {
            (Ok(parsed), Err(_))
                if channel.is_none() =>
                    channel = Some(parsed),
            (_, Ok(parsed))
                if role.is_none() =>
                    role = Some(parsed),
            _ => return bad_message(ctx, msg).await,
        }
        args.advance();
    }
    if role.is_none() {
        return bad_message(ctx, msg).await;
    }

    let db = async {
        // This is split into a function for ? usage
        // But, even more-so, it's a two-await that can be done concurrently to the single-awaits
        async fn get_db_and_associations(ctx: &Context, guild: GuildId, channel: ChannelId)
            -> CommandResult<(RwLockReadGuard<'_, TypeMap>, Vec<RoleAssociation>)>
        {
            let db = ctx.data.read().await;
            let associations = get_role_associations(
                db
                    .get::<DatabaseHandle>()
                    .ok_or("Database not present")?,
                channel,
                guild,
            ).await?;
            Ok((db, associations))
        }
        get_db_and_associations(ctx, guild, channel.unwrap_or(ChannelId(!0))).await
    };
    let channel = async {
        if let Some(channel) = channel {
            if let Some(channel) = ctx.cache.guild_channel(channel).await {
                if channel.guild_id == guild {
                    Ok(Some(channel.id))
                } else { Err(()) }
            } else { Err(()) }
        } else { Ok(None) }
    };
    let role = async {
        if let Some(role) = role {
            if let Some(role) = ctx.cache.role(guild, role).await {
                Ok(Some(role.id))
            } else { Err(()) }
        } else { Ok(None) }
    };
    let (db, channel, role): (_, Result<Option<ChannelId>, ()>, Result<Option<RoleId>, ()>) = join!(db, channel, role);
    let (db, associations): (_, Vec<RoleAssociation>) = db?;
    let db: &Database = db
        .get::<DatabaseHandle>()
        .ok_or("Database not present")?;
    let (channel, role) = match (channel, role) {
        (_, Ok(None)) => return bad_message(ctx, msg).await,
        (Ok(channel), Ok(Some(role))) => (channel, role),
        _ => return unamused(ctx, msg).await, // Cross-guild poisoning
    };

    if let Some(channel) = channel {
        for mut association in associations {
            if association.channel.is_some() {
                let old = association.role;
                association.role = role;
                association.save(db, None).await?;

                msg.channel_id.send_message(ctx, |message| message
                    .reference_message(reply_reference)
                    .embed(|e| e
                        .title("Role Association:")
                        .description(format_args!(
                            "{} updated to {} from {}",
                            Mentionable::from(channel),
                            Mentionable::from(role),
                            Mentionable::from(old),
                        ))
                    )
                ).await?;
                return Ok(());
            }
        }

        // None exist; make a new one!
        RoleAssociation {
            id: None,
            channel: Some(channel),
            server: None,
            role
        }
            .save(db, None)
            .await?;

        msg.channel_id.send_message(ctx, |message| message
            .reference_message(reply_reference)
            .embed(|e| e
                .title("Role Association:")
                .description(format_args!(
                    "{} is now associated to {}",
                    Mentionable::from(channel),
                    Mentionable::from(role),
                ))
            )
        ).await?;
    } else {
        for mut association in associations {
            if association.server.is_some() {
                let old = association.role;
                association.role = role;
                association.save(db, None).await?;

                msg.channel_id.send_message(ctx, |message| message
                    .reference_message(reply_reference)
                    .embed(|e| e
                        .title("Role Association:")
                        .description(format_args!(
                            "Server's generic role updated to {} from {}",
                            Mentionable::from(role),
                            Mentionable::from(old),
                        ))
                    )
                ).await?;
                return Ok(());
            }
        }

        // None exist; make a new one!
        RoleAssociation {
            id: None,
            channel: None,
            server: Some(guild),
            role
        }
            .save(db, None)
            .await?;

        msg.channel_id.send_message(ctx, |message| message
            .reference_message(reply_reference)
            .embed(|e| e
                .title("Role Association:")
                .description(format_args!(
                    "Server's generic role is now {}",
                    Mentionable::from(role),
                ))
            )
        ).await?;
    }

    Ok(())
}
