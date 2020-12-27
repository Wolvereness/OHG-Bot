use serenity::{
    prelude::*,
    model::prelude::*,
    framework::standard::{
        CommandResult,
        macros::{
            command,
            group,
        },
    },
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
