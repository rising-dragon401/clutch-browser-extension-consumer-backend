use sea_orm::Set;
use sea_orm::{entity::prelude::*, sea_query::Expr};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::api::SigningStrategy;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "guardian_settings")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub signers: SigningStrategy,
    pub account_id: String,
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
    signers: SigningStrategy,
    account_id: String,
) -> anyhow::Result<()> {
    let model = ActiveModel {
        id: Set(id.to_string()),
        signers: Set(signers.to_owned()),
        account_id: Set(account_id.to_owned()),
    };

    Entity::insert(model)
        .exec_without_returning(db)
        .await
        .map(|_| ())
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn find_for_account_id(
    db: &DatabaseConnection,
    account_id: String,
) -> anyhow::Result<Option<Model>> {
    Entity::find()
        .filter(Column::AccountId.eq(account_id))
        .one(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn update_settings_for_account_id(
    db: &DatabaseConnection,
    account_id: String,
    signers: SigningStrategy,
) -> anyhow::Result<()> {
    Entity::update_many()
        .col_expr(Column::Signers, Expr::value(signers.to_owned()))
        .filter(Column::AccountId.eq(account_id))
        .exec(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
        .map(|_| ())
}
