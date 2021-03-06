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
#[path = "commands/rpg_enabled.rs"]
mod rpg;
#[cfg(not(feature = "rpg"))]
#[path = "commands/rpg_disabled.rs"]
mod rpg;

pub use rpg::RPG_GROUP;

#[group]
#[commands(ping, parrot)]
pub struct General;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    #[cfg(feature = "rpg")]
    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        crate::print_errors_impl(
            "RPG_Reaction_Add",
            rpg::reaction_add(&ctx, reaction).await,
        )
    }
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
