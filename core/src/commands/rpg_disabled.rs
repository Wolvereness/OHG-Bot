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
    models::RoleAssociation,
    util::{
        get_role_associations,
        Mentionable,
    },
    DatabaseHandle,
};

#[group]
#[commands(play, rpg_channel)]
pub struct RPG;

#[command]
#[only_in("guild")]
async fn play(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.send_message(ctx, |message| message
        .reference_message(msg)
        .embed(|e| e
            .title("RPG")
            .description("Sorry, RPG is unavailable.")
        )
    ).await?;

    Ok(())
}

#[command]
#[only_in("guild")]
#[required_permissions("ADMINISTRATOR")]
async fn rpg_channel(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.send_message(ctx, |message| message
        .reference_message(msg)
        .embed(|e| e
            .title("RPG")
            .description("Sorry, RPG is unavailable.")
        )
    ).await?;

    Ok(())
}
