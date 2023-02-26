
use serenity::{
    prelude::*,
    model::prelude::*,
    framework::standard::{
        Args,
        CommandResult,
        macros::{
            command,
            group,
        },

    },
    http::CacheHttp,
    json::json,
};




#[group]
#[commands(make_thread)]
pub struct Threads;

#[command]
#[only_in("guild")]
#[required_permissions("ADMINISTRATOR")]
async fn make_thread(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild = msg.guild_id.ok_or("No guild present")?;
    let channel: ChannelId = args.parse()?;
    let channel: GuildChannel = ctx.cache.guild_channel(channel).ok_or("No guild")?;
    if guild != channel.guild_id {
        return Err("Wrong guild".into())
    }
    let name = args.advance().current().map(str::to_string).ok_or("No title")?;
    let rest = args.advance().rest().to_string();
    channel.create_private_thread(ctx.http(), |thread| {
        thread
            .name(name)
            .kind(ChannelType::PublicThread);
        let _ = thread.0.insert("message", json! {{
            "content": rest,
        }});
        thread
    }).await?;

    Ok(())
}

