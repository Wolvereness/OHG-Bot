use std::{
    result::Result,
    fmt::Write as _,
    future::Future,
};
use wither::{
    Model,
    mongodb::Database,
};
use futures::join;
use tokio::sync::RwLockReadGuard;
use serenity::{
    prelude::*,
    model::prelude::*,
    framework::standard::{
        Args,
        CommandError,
        CommandResult,
        macros::{
            command,
            group,
        },
    },
    builder::CreateEmbed,
};
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
#[commands(join, dump_associations, leave, register_role)]
pub struct Roles;

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
        let (mut member, associations) = load_member_and_associations(ctx, msg, guild, db).await?;
        execute_contextual_role_change(
            ctx,
            msg.into(),
            &mut member,
            associations,
            Member::add_role,
            |e, role| e
                .title("Join command:")
                .description(format_args!(
                    "{} has joined {} for {}.",
                    Mentionable::from(msg.author.id),
                    Mentionable::from(role),
                    Mentionable::from(msg.channel_id),
                )),
            |e, role| e
                .title("Join command:")
                .description(format_args!(
                    "{} has joined {}.",
                    Mentionable::from(msg.author.id),
                    Mentionable::from(role),
                )),
        ).await?;
    } else {
        send_message_not_yet_implemented(ctx, msg).await?;
    }

    Ok(())
}

#[command]
#[only_in("guild")]
async fn leave(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
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
        let (mut member, associations) = load_member_and_associations(ctx, msg, guild, db).await?;
        execute_contextual_role_change(
            ctx,
            msg.into(),
            &mut member,
            associations,
            Member::remove_role,
            |e, role| e
                .title("Leave command:")
                .description(format_args!(
                    "{} has left {} for {}.",
                    Mentionable::from(msg.author.id),
                    Mentionable::from(role),
                    Mentionable::from(msg.channel_id),
                )),
            |e, role| e
                .title("Leave command:")
                .description(format_args!(
                    "{} has left {}.",
                    Mentionable::from(msg.author.id),
                    Mentionable::from(role),
                )),
        ).await?;
    } else {
        send_message_not_yet_implemented(ctx, msg).await?;
    }

    Ok(())
}

async fn execute_contextual_role_change<'a, F1, F2, V, E>(
    ctx: &'a Context,
    msg: &Message,
    member: &'a mut Member,
    associations: Vec<RoleAssociation>,
    change_roles: F1,
    embed_channel_context: impl FnOnce(&mut CreateEmbed, RoleId) -> &mut CreateEmbed,
    embed_server_context: impl FnOnce(&mut CreateEmbed, RoleId) -> &mut CreateEmbed,
) -> CommandResult
    where
        F1: FnOnce(
            &'a mut Member,
            &'a Context,
            RoleId,
        ) -> F2,
        F2: Future<Output=Result<V, E>>,
        CommandError: From<E>,
{
    if let Some(association) = associations
        .iter()
        .find(|association| association.channel.is_some())
    {
        change_roles(member, ctx, association.role).await?;
        msg.channel_id.send_message(ctx, |message| message
            .reference_message(msg)
            .embed(|e| embed_channel_context(e, association.role))
        ).await?;
    } else if let Some(association) = associations
        .iter()
        .find(|association| association.server.is_some())
    {
        change_roles(member, ctx, association.role).await?;
        msg.channel_id.send_message(ctx, |message| message
            .reference_message(msg)
            .embed(|e| embed_server_context(e, association.role))
        ).await?;
    } else {
        send_message_no_group_found(ctx, msg).await?;
    }
    return Ok(())
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

    msg.channel_id.send_message(ctx, |message| message
        .reference_message(msg)
        .embed(|e| e
            .title("Association Dump:")
            .description(description)
        )
    ).await?;

    Ok(())
}

async fn load_member_and_associations(ctx: &Context, msg: &Message, guild: GuildId, db: &Database) -> CommandResult<(Member, Vec<RoleAssociation>)> {
    let member = guild.member(ctx, &msg.author.id);
    let associations = get_role_associations(db, msg.channel_id, guild);
    let (member, associations): (Result<Member, _>, Result<Vec<RoleAssociation>, _>) = join!(member, associations);
    Ok((member?, associations?))
}

async fn send_message_not_yet_implemented(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.send_message(ctx, |message| message
        .reference_message(msg)
        .embed(|e| e
            .title("I can't do that yet!")
            .description("At some future date, I'll be able to find the group you want.\n\nFor now, please try to join/leave in the respective channel.")
        )
    ).await?;
    Ok(())
}

async fn send_message_no_group_found(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.send_message(ctx, |message| message
        .reference_message(msg)
        .embed(|e| e
            .title("No groups found!")
            .description(format_args!("No group configured for {}.\nNo generic group configured for the server.", Mentionable::from(msg.channel_id)))
        )
    ).await?;
    Ok(())
}

#[command]
#[only_in("guild")]
#[required_permissions("ADMINISTRATOR")]
async fn register_role(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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
            // This arm triggers when the channel itself is highlighted
            (Ok(parsed), Err(_))
            if channel.is_none() =>
                channel = Some(parsed),
            // This triggers on highlight or number
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
                    .reference_message(msg)
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
            .reference_message(msg)
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
                    .reference_message(msg)
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
            .reference_message(msg)
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
