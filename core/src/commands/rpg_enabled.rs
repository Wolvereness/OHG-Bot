use wither::{
    bson::doc,
    Model,
    mongodb::{
        Database,
        options::FindOneOptions,
    },
};
use futures::{
    join,
    future::join_all,
};
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
use cache_2q::Entry;
use ohg_bot_headers::{
    Action,
    Reactions,
    CreateEmbed,
    StateReaction,
};
use crate::{
    models::{
        RPGState,
        RPGChannel,
        Shim,
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
        const CONTENT: &str = "\
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
        let db = &data.get::<DatabaseHandle>().ok_or("No database?")?.base;
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

const ADDITIONAL_ALLOWED_CHARS: &[char] = &[' ', '-', '.'] as _;

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
            && !ADDITIONAL_ALLOWED_CHARS.contains(&c)
        )
        || (
            !defined_name.is_empty()
            && !defined_name.contains(char::is_alphanumeric)
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
    let db: &Database = &data_lock.get::<DatabaseHandle>().ok_or("Database not present")?.rpg;

    let initial = ohg_bot_rpg::initial(defined_name);
    let rpg_states = data_lock.get::<RPGState>().ok_or("No RPG states?")?;

    // Get the lock before the message, in case a reaction appears before the unyield.
    let rpg_states_lock = rpg_states.lock();
    let display = initial.display(db);
    let (mut rpg_states_lock, display): (MutexGuard<'_, RPGStateHolder>, _) =
        join!(rpg_states_lock, display);
    let (reactions, embed): (Reactions, CreateEmbed) = display?;

    let message = msg.channel_id.send_message(ctx, |message| message
        .reference_message(msg)
        .content(author_mention)
        .embed(|e| {
            *e = embed;
            e
        })
    ).await?;

    let reactions =
        pre_fill_reactions(ctx, reactions, msg.channel_id, message.id);

    let mut state = RPGState {
        id: None,
        state: initial,
        active: true,
        message: message.id,
        owner: msg.author.id,
        iteration: 0,
        previous: None,
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

async fn pre_fill_reactions(ctx: &Context, reactions: Reactions, channel: ChannelId, message: MessageId) -> CommandResult {
    let mut buff = ReactionType::Unicode(String::new());
    for reaction in reactions.into_iter() {
        if let ReactionType::Unicode(buff) = &mut buff {
            buff.clear();
            buff.push_str(reaction.emoji);
        } else {
            unreachable!();
        }

        ctx.http.create_reaction(
            channel.0,
            message.0,
            &buff,
        ).await?;
    }
    Ok(())
}

pub async fn reaction_add(ctx: &Context, reaction: Reaction) -> CommandResult {
    let emoji = if let ReactionType::Unicode(emoji) = &reaction.emoji {
        emoji.as_str()
    } else {
        return Ok(());
    };
    let channel = reaction.channel_id;
    let message = reaction.message_id;
    let user = if let Some(user) = reaction.user_id {
        user
    } else {
        return Ok(());
    };
    if user == ctx.cache.current_user_id().await {
        return Ok(());
    }
    let data_lock = ctx.data.read().await;
    if !data_lock
        .get::<RPGChannel>()
        .ok_or("No RPG Channels")?
        .contains(&channel)
    {
        return Ok(());
    }

    let db = &data_lock.get::<DatabaseHandle>().ok_or("No Database")?.rpg;
    let states_mutex = data_lock.get::<RPGState>().ok_or("No States")?;
    let state =
        if let Some(state) =
            obtain_state(
                db,
                states_mutex,
                message,
                user,
            ).await?
        {
            state
        } else {
            return Ok(());
        };

    let result = operate_on_state(ctx, db, emoji, state, channel, message, user).await;
    match result {
        Ok(state) => {
            unlock(states_mutex, Some(state), message).await;
        },
        Err(e) => {
            unlock(states_mutex, None, message).await;
            return Err(e);
        },
    }

    Ok(())
}

async fn operate_on_state(
    ctx: &Context,
    db: &Database,
    emoji: &str,
    mut state: RPGState,
    channel: ChannelId,
    message: MessageId,
    user: UserId,
) -> CommandResult<Option<RPGState>> {
    let old_reactions = state.state.reactions(db).await?;
    let change: bool;
    state.state = match state.state.action(db, emoji).await {
        Ok(Action::NoChange(state)) => {
            change = false;
            state
        },
        Ok(Action::BadReact(state)) => {
            change = false;
            state
        },
        Ok(Action::Changed(state)) => {
            change = true;
            state
        },
        Err(e) => {
            return Err(e);
        }
    };
    if !change {
        return Ok(Some(state));
    }
    state.previous = state.id.take();
    state.iteration += 1;

    let deletions = join_all(old_reactions
        .into_iter()
        .map(|reaction: StateReaction| channel
            .delete_reaction_emoji(
                ctx,
                message,
                ReactionType::Unicode(reaction.emoji.to_string()),
            )
        )
    );
    let edit = async {
        let (reactions, embed) = state.state.display(db).await?;

        let save = state.save(db, None);
        let edit = channel.edit_message(ctx, message, |e| e
            .content(Mentionable::from(user))
            .embed(|e| {
                *e = embed;
                e
            })
        );

        let (save, edit) = join!(save, edit);
        save?;
        edit?;

        Ok(reactions) as CommandResult<Reactions>
    };
    let (deletions, edit) = join!(deletions, edit);

    for deletion in deletions {
        deletion?;
    }
    pre_fill_reactions(ctx, edit?, channel, message).await?;

    Ok(Some(state))
}

async fn unlock(
    mutex: &Mutex<RPGStateHolder>,
    state: Option<Option<RPGState>>,
    message: MessageId,
) {
    let mut states = mutex.lock().await;
    let RPGStateHolder {
        cache,
        lockout,
    } = &mut *states;
    if let Some(state) = state {
        drop(cache.insert(message, state));
    } else {
        drop(cache.remove(&message));
    }
    lockout.remove(&message);
}

async fn obtain_state(
    db: &Database,
    mutex: &Mutex<RPGStateHolder>,
    message: MessageId,
    user: UserId,
) -> CommandResult<Option<RPGState>> {
    let mut states = mutex.lock().await;
    let RPGStateHolder {
        cache,
        lockout,
    } = &mut *states;
    if lockout.contains(&message) {
        return Ok(None);
    }
    let entry = match cache.entry(message) {
        Entry::Occupied(mut occupied) => {
            let occupied = occupied.get_mut();
            let state = if let Some(state) = occupied.take() {
                if state.owner != user {
                    *occupied = Some(state);
                    return Ok(None);
                }
                lockout.insert(message);
                Some(state)
            } else {
                // There's no lockout,
                // which means it's been looked up and set to None.
                None
            };
            return Ok(state);
        },
        Entry::Vacant(vacant) =>
            vacant.insert(None),
    };

    // We need to check the database

    let mut options = FindOneOptions::default();
    options.sort = Some(doc!{
        "iteration": -1,
    });
    let state: Option<RPGState> = RPGState::find_one(
        db,
        Some(doc!{
            "message": doc!{ "$eq": &Shim::from(message) },
        }),
        Some(options),
    ).await?;

    let state = if let Some(state) = state {
        state
    } else {
        // No value in the database
        return Ok(None);
    };
    if !state.active {
        // Set to inactive
        return Ok(None);
    }
    if state.owner != user {
        // Bad user, but remember the db lookup
        *entry = Some(state);
        return Ok(None);
    }

    lockout.insert(message);
    Ok(Some(state))
}
