use ohg_bot_core::{models, connect_db};
use wither::Model;
use serenity::model::prelude::*;

pub async fn main() {
    let db = connect_db().await;

    if models::DiscordCredentials::find_one(&db, None, None)
        .await
        .expect("Failed to search discord credentials")
        .is_none()
    {
        models::DiscordCredentials {
            id: None,
            private: input("Private key:"),
            public: input("Public key:"),
            token: input("Bot token:"),
            bot_id: input("Bot id:"),
            prefix: input("Command prefix:"),
            operator: input("Operator id:")
                .parse()
                .map(UserId)
                .expect("Bad operator id"),
        }
            .save(&db, None)
            .await
            .expect("Failed to save");
    }
    while input("Insert RoleAssociation (true/false)?").parse().expect("Not true/false") {
        models::RoleAssociation {
            id: None,
            channel: if input("Add channel?").parse().expect("Not true/false") {
                input("Channel id:")
                    .parse()
                    .map(ChannelId)
                    .map(Some)
                    .expect("Bad channel id")
            } else {
                None
            },
            server: if input("Add server?").parse().expect("Not true/false") {
                input("Server id:")
                    .parse()
                    .map(GuildId)
                    .map(Some)
                    .expect("Bad server id")
            } else {
                None
            },
            role: input("Role id:")
                .parse()
                .map(RoleId)
                .expect("Bad role id"),
        }
            .save(&db, None)
            .await
            .expect("Failed to save");
    }
    while input("Insert RoleStatus (true/false)?").parse().expect("Not true/false") {
        models::RoleStatus {
            id: None,
            role: input("Role id:")
                .parse()
                .map(RoleId)
                .expect("Bad role id"),
        }
            .save(&db, None)
            .await
            .expect("Failed to save");
    }
}

fn input<T: std::fmt::Display>(msg: T) -> String {
    println!("{}", msg);
    let mut ret = "".into();
    std::io::stdin().read_line(&mut ret).expect("Failed to get input");
    while ret.ends_with(|c: char| c.is_whitespace()) {
        ret.remove(ret.len() - 1);
    }
    ret
}