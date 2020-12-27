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
use crate::{
    models::{
        RPGState,
        RPGChannel,
    },
    util::Mentionable,
    DatabaseHandle,
};
use ohg_bot_headers::StateReaction;

#[group]
#[commands(play, rpg_channel)]
pub struct RPG;

#[command]
#[only_in("guild")]
#[required_permissions("ADMINISTRATOR")]
async fn rpg_channel(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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
    let defined_name = args.rest();
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

    let author_mention = Mentionable::from(&msg.author);

    let data_lock = ctx.data.read().await;
    let db: &Database = data_lock.get::<DatabaseHandle>().ok_or("Database not present")?;

    let initial = if defined_name.is_empty() {
        ohg_bot_rpg::initial(&author_mention)
    } else {
        ohg_bot_rpg::initial(defined_name)
    };
    let reactions = initial.reactions(db).await;
    let rpg_states = data_lock.get::<RPGState>().ok_or("No RPG states?")?;
    let mut rpg_states_lock = rpg_states.lock().await;
    let message = msg.channel_id.send_message(ctx, |message| message
        .reference_message(msg)
        .embed(|e| {
            if defined_name.is_empty() {
                e.title(format_args!(
                    "{}'s Adventure",
                    author_mention,
                ));
            } else {
                e.title(format_args!(
                    "{}'s ({}) Adventure",
                    defined_name,
                    author_mention,
                ));
            }
            e.description(&initial);
            for StateReaction { emoji, description, } in reactions.iter().copied() {
                e.field(emoji, description, true);
            }
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
