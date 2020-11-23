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

pub async fn main() {
    let db_url = read_to_string("./db-url.txt").expect("Failed to read db-url.txt");
    let db = wither::mongodb::Client::with_uri_str(&db_url)
        .await
        .expect("Failed to connect")
        .database("ohg");
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
