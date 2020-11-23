use std::fs::read_to_string;
use ohg_bot_lib::models::{RoleAssociation, RoleStatus};
use wither::{Model, ModelCursor};
use futures::StreamExt;

pub async fn main() {
    let db_url = read_to_string("./db-url.txt").expect("Failed to read db-url.txt");
    let db = wither::mongodb::Client::with_uri_str(&db_url)
        .await
        .expect("Failed to connect")
        .database("ohg");
    dump(RoleAssociation::find(&db, None, None).await.expect("Failed to search RoleAssociation"))
        .await
        .expect("Failed to dump RoleAssociation");
    dump(RoleStatus::find(&db, None, None).await.expect("Failed to search RoleStatus"))
        .await
        .expect("Failed to dump RoleStatus");
}

async fn dump<T: Model + std::fmt::Debug>(mut models: ModelCursor<T>) -> Result<(), wither::WitherError> {
    while let Some(model) = models.next().await {
        println!("{:?}", model?);
    }
    Ok(())
}
