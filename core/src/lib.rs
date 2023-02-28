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
    prelude::{
        TypeMapKey,
        GatewayIntents,
    },
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

pub async fn connect_db() -> DatabaseHandle {
    let client = DBClient::with_uri_str(
        &read_to_string("./db-url.txt")
            .expect("Failed to read db-url.txt")
    )
        .await
        .expect("Failed to connect");
    DatabaseHandle {
        base: client.database(DATABASE_NAME),
        client,
    }
}

pub struct DatabaseHandle {
    pub base: Database,
    pub client: DBClient,
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
        .group(&commands::THREADS_GROUP)
        .after(print_errors);

    let mut client = Client::builder(&creds.token, GatewayIntents::all())
        .framework(framework)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
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
