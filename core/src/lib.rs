#![deny(rust_2018_idioms)]

use std::{
    collections::HashSet,
    fs::read_to_string,
    time::{
        SystemTime,
        UNIX_EPOCH,
    },
};

use serenity::{
    client::{
        Client,
    },
    framework::standard::{
        CommandResult,
        macros::hook,
        StandardFramework,
    },
    prelude::TypeMapKey,
};
use wither::{
    mongodb::Database,
    prelude::*
};
use futures::stream::StreamExt;

use crate::{
    models::{
        DiscordCredentials,
        RPGChannel,
        RPGState,
    },
    util::RPGStateHolder,
};
use cache_2q::Cache;

pub mod models;
mod commands;
mod util;

pub const DATABASE_NAME: &'static str = "ohg";

pub async fn connect_db() -> Database {
    wither::mongodb::Client::with_uri_str(
        &read_to_string("./db-url.txt")
            .expect("Failed to read db-url.txt")
    )
        .await
        .expect("Failed to connect")
        .database(DATABASE_NAME)
}

struct DatabaseHandle;

impl TypeMapKey for DatabaseHandle {
    type Value = Database;
}

pub async fn main() {
    let db = connect_db().await;
    let creds: DiscordCredentials = DiscordCredentials::find_one(&db, None, None)
        .await
        .expect("Failed to search discord credentials")
        .expect("Failed to find discord credentials");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix(&creds.prefix)) // set the bot's prefix to "~"
        .group(&commands::GENERAL_GROUP)
        .group(&commands::ROLES_GROUP)
        .group(&commands::RPG_GROUP)
        .after(print_errors);

    let mut client = Client::builder(&creds.token)
        .event_handler(commands::Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        #[cfg(feature = "rpg")]
        {
            let mut channels = HashSet::new();
            let mut db_channels = RPGChannel::find(&db, None, None).await
                .expect("Failed to retrieve RPG channels");
            while let Some(channel) = db_channels.next().await {
                let RPGChannel { channel, .. } = channel.expect("Failed to retrieve RPG channel");
                channels.insert(channel);
            }
            data.insert::<RPGChannel>(channels);
            data.insert::<RPGState>(
                RPGStateHolder {
                    cache: Cache::new(128),
                    lockout: Default::default(),
                }.into()
            );
        }
        data.insert::<DatabaseHandle>(db);
        data.insert::<DiscordCredentials>(creds);
    }

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[hook]
async fn print_errors(
    _: &serenity::prelude::Context,
    _: &serenity::model::channel::Message,
    cmd_name: &str,
    error: CommandResult,
) {
    let error = if let Err(e) = error {
        e
    } else {
        return;
    };
    println!(
        "{} {}: {:#?}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("<1970 not supported")
            .as_millis(),
        cmd_name,
        error,
    );
}
