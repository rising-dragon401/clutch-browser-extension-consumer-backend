use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "guardians")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub email: String,
    pub account_id: Option<String>,
    pub wallet_address: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("No RelationDef")
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub async fn create(
    db: &DatabaseConnection,
    id: Uuid,
    email: String,
    account_id: Option<String>,
    wallet_address: Option<String>,
) -> anyhow::Result<()> {
    let model = ActiveModel {
        id: Set(id.to_string()),
        email: Set(email.to_owned()),
        account_id: Set(account_id.to_owned()),
        wallet_address: Set(wallet_address.to_owned()),
    };

    Entity::insert(model)
        .exec_without_returning(db)
        .await
        .map(|_| ())
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn find_by_email(
    db: &DatabaseConnection,
    email: String,
) -> anyhow::Result<Option<Model>> {
    Entity::find()
        .filter(Column::Email.eq(email))
        .one(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn find_by_account_id(
    db: &DatabaseConnection,
    account_id: String,
) -> anyhow::Result<Option<Model>> {
    Entity::find()
        .filter(Column::AccountId.eq(account_id))
        .one(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn find_by_id(
    db: &DatabaseConnection,
    guardian_id: String,
) -> anyhow::Result<Option<Model>> {
    Entity::find()
        .filter(Column::Id.eq(guardian_id))
        .one(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn find_all_by_ids(
    db: &DatabaseConnection,
    guardian_ids: Vec<String>,
) -> anyhow::Result<Vec<Model>> {
    Entity::find()
        .filter(Column::Id.is_in(guardian_ids))
        .all(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}
