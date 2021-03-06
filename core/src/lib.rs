#![deny(rust_2018_idioms)]

use std::{
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
    mongodb::{
        Client as DBClient,
        Database,
    },
    prelude::*
};

use crate::models::DiscordCredentials;

pub mod models;
mod commands;
mod util;

pub const DATABASE_NAME: &str = "ohg";
#[cfg(feature = "rpg")]
pub const RPG_DATABASE_NAME: &str = "rpg";

pub async fn connect_db() -> DatabaseHandle {
    let client = DBClient::with_uri_str(
        &read_to_string("./db-url.txt")
            .expect("Failed to read db-url.txt")
    )
        .await
        .expect("Failed to connect");
    DatabaseHandle {
        base: client.database(DATABASE_NAME),
        #[cfg(feature = "rpg")]
        rpg: client.database(RPG_DATABASE_NAME),
        client,
    }
}

pub struct DatabaseHandle {
    pub base: Database,
    pub client: DBClient,
    #[cfg(feature = "rpg")]
    pub rpg: Database,
}

impl TypeMapKey for DatabaseHandle {
    type Value = DatabaseHandle;
}

pub async fn main() {
    let database_handle = connect_db().await;
    let creds: DiscordCredentials = DiscordCredentials::find_one(
        &database_handle.base,
        None,
        None,
    )
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
            use std::collections::HashSet;

            use cache_2q::Cache;
            use futures::stream::StreamExt;
            use crate::{
                models::{
                    RPGChannel,
                    RPGState,
                },
                util::RPGStateHolder,
            };

            let mut channels = HashSet::new();
            let mut db_channels = RPGChannel::find(&database_handle.base, None, None).await
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
        data.insert::<DatabaseHandle>(database_handle);
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
    print_errors_impl(cmd_name, error)
}

fn print_errors_impl(cmd_name: &str, error: CommandResult) {
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
