use std::{
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

mod roles;
pub use roles::ROLES_GROUP;

#[cfg(feature = "rpg")]
mod rpg_enabled;
#[cfg(not(feature = "rpg"))]
mod rpg_disabled;

#[cfg(feature = "rpg")]
pub use rpg_enabled::RPG_GROUP;
#[cfg(not(feature = "rpg"))]
pub use rpg_disabled::RPG_GROUP;


#[group]
#[commands(ping, parrot)]
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
        .reference_message(msg)
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
