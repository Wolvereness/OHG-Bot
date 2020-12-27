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
        CommandResult,
        macros::{
            command,
            group,
        },
    },
};
use tokio::sync::MutexGuard;
use ohg_bot_headers::{
    Reactions,
    CreateEmbed,
};
use crate::{
    models::{
        RPGState,
        RPGChannel,
    },
    util::{
        Mentionable,
        RPGStateHolder,
    },
    DatabaseHandle,
};

#[group]
#[commands(play, rpg_channel)]
pub struct RPG;

#[command]
#[only_in("guild")]
#[required_permissions("ADMINISTRATOR")]
async fn rpg_channel(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    msg.channel_id.broadcast_typing(ctx).await?;
    async fn bad_message(ctx: &Context, msg: &Message) -> CommandResult {
        const CONTENT: &'static str = "\
            Specify exactly one channel.\
        ";
        msg.reply(ctx, CONTENT).await?;
        return Ok(());
    }

    let guild = msg.guild_id.ok_or("No guild present")?;
    let channel: ChannelId = match args.parse() {
        Ok(channel) => channel,
        Err(_) => return bad_message(ctx, msg).await,
    };
    if !args.advance().is_empty() {
        return bad_message(ctx, msg).await;
    }
    let channel = if let Some(channel) = ctx.cache.guild_channel(channel).await {
        channel
    } else {
        return bad_message(ctx, msg).await;
    };
    if guild != channel.guild_id {
        return bad_message(ctx, msg).await;
    }

    let mut data = ctx.data.write().await;
    let channels = data.get_mut::<RPGChannel>().ok_or("No rpg channels?")?;
    if channels.insert(channel.id) {
        let db = data.get::<DatabaseHandle>().ok_or("No database?")?;
        RPGChannel {
            id: None,
            channel: channel.id,
        }.save(db, None).await?;
        drop(data);
        channel.send_message(ctx, |message| message
            .content("RPG Enabled. Use `!play` with your character name to start.")
        ).await?;
    } else {
        drop(data);
        msg.channel_id.send_message(ctx, |message| message
            .reference_message(msg)
            .content("RPG already enabled for that channel.")
        ).await?;
    };

    Ok(())
}

#[command]
#[only_in("guild")]
#[cfg(feature = "rpg")]
async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.channel_id.broadcast_typing(ctx).await?;
    let mut defined_name = args.rest().trim();
    if
        defined_name.len() > 30
        || defined_name.contains(|c: char|
            !c.is_alphanumeric()
            && c != ' '
        )
    {
        msg.channel_id.send_message(ctx, |message| message
            .reference_message(msg)
            .embed(|e| e
                .title("Who?")
                .description("That's a mouthful of a name.")
            )
        ).await?;
        return Ok(())
    }

    let author_nick: String;
    if defined_name.is_empty() {
        if let Some(nick) = msg.author_nick(ctx).await {
            author_nick = nick;
            defined_name = (&author_nick).trim();
        } else {
            defined_name = (&msg.author.name).trim();
        };
    }
    let author_mention = Mentionable::from(&msg.author);

    let data_lock = ctx.data.read().await;
    if !data_lock.get::<RPGChannel>().ok_or("Channels not present")?.contains(&msg.channel_id) {
        msg.channel_id.send_message(ctx, |message| message
            .reference_message(msg)
            .embed(|e| e
                .title("No RPG to be had here!")
                .description(format_args!("The RPG isn't enabled in {}.", Mentionable::from(msg.channel_id)))
            )
        ).await?;
        return Ok(())
    }
    let db: &Database = data_lock.get::<DatabaseHandle>().ok_or("Database not present")?;

    let initial = ohg_bot_rpg::initial(defined_name);
    let rpg_states = data_lock.get::<RPGState>().ok_or("No RPG states?")?;

    // Get the lock before the message, in case a reaction appears before the unyield.
    let reactions = initial.reactions(db);
    let rpg_states_lock = rpg_states.lock();
    let display = initial.display(db);
    let (reactions, mut rpg_states_lock, display): (_, MutexGuard<'_, RPGStateHolder>, _) =
        join!(reactions, rpg_states_lock, display);
    let reactions: Reactions = reactions?;

    let display: CreateEmbed = display?;
    let message = msg.channel_id.send_message(ctx, |message| message
        .reference_message(msg)
        .content(author_mention)
        .embed(|e| {
            *e = display;
            e
        })
    ).await?;

    let reactions = async {
        for reaction in reactions.into_iter() {
            message.react(
                ctx,
                ReactionType::Unicode(reaction.emoji.to_string()),
            ).await?;
        }
        Ok(()) as CommandResult
    };

    let mut state = RPGState {
        id: None,
        state: initial,
        active: true,
        message: message.id,
        owner: msg.author.id,
        iteration: 0,
    };
    let save_state = state.save(db, None);

    let (reactions, save_state) = join!(reactions, save_state);
    save_state?;
    reactions?;

    if let Some(previous) = rpg_states_lock
        .cache
        .insert(message.id, Some(state))
    {
        return Err(format!(
            "{} duplicate: {:?}",
            message.id,
            previous,
        ).into());
    }

    Ok(())
}
