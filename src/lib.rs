use std::fs::read_to_string;

use serenity::{
    client::{
        Client,
    },
    framework::standard::{
        StandardFramework,
    }
};
use wither::prelude::*;
use crate::models::{
    DatabaseHandle,
    DiscordCredentials,
};
use wither::mongodb::Database;
use std::time::{
    SystemTime,
    UNIX_EPOCH,
};

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
        .after(print_errors);

    let mut client = Client::builder(&creds.token)
        .event_handler(commands::Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<DatabaseHandle>(db);
    }

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[serenity::framework::standard::macros::hook]
async fn print_errors(
    _: &serenity::prelude::Context,
    _: &serenity::model::channel::Message,
    cmd_name: &str,
    error: serenity::framework::standard::CommandResult,
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
