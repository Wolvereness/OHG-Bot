use ohg_bot_core::{
    models::{
        RoleAssociation,
        RoleStatus,
    },
    connect_db,
};
use wither::{
    Model,
    ModelCursor,
};
use futures::StreamExt;

pub async fn main() {
    let db = connect_db().await.base;
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
