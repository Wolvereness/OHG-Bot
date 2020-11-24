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

pub mod models;
mod commands;

use models::DiscordCredentials;
use crate::models::DatabaseHandle;
use wither::mongodb::Database;

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
        .group(&commands::GENERAL_GROUP);

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
