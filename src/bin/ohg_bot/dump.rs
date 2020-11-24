use ohg_bot_lib::models::{RoleAssociation, RoleStatus};
use wither::{Model, ModelCursor};
use futures::StreamExt;
use ohg_bot_lib::connect_db;

pub async fn main() {
    let db = connect_db().await;
    dump(
        RoleAssociation::find(&db, None, None)
            .await
            .expect("Failed to search RoleAssociation")
    )
        .await
        .expect("Failed to dump RoleAssociation");
    dump(
        RoleStatus::find(&db, None, None)
            .await
            .expect("Failed to search RoleStatus")
    )
        .await
        .expect("Failed to dump RoleStatus");
}

async fn dump<T: Model + std::fmt::Debug>(mut models: ModelCursor<T>) -> Result<(), wither::WitherError> {
    while let Some(model) = models.next().await {
        println!("{:?}", model?);
    }
    Ok(())
}
