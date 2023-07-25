use sea_orm::Set;
use sea_orm::{entity::prelude::*, sea_query::Expr};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "account_guardians")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub guardian_id: String,
    pub account_id: String,
    pub status: String,
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
    guardian_id: String,
    account_id: String,
    status: String,
) -> anyhow::Result<()> {
    let model = ActiveModel {
        id: Set(id.to_string()),
        guardian_id: Set(guardian_id.to_owned()),
        account_id: Set(account_id.to_owned()),
        status: Set(status.to_owned()),
    };

    Entity::insert(model)
        .exec_without_returning(db)
        .await
        .map(|_| ())
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn find_all_accounts_by_guardian_id(
    db: &DatabaseConnection,
    guardian_id: String,
) -> anyhow::Result<Vec<Model>> {
    Entity::find()
        .filter(Column::GuardianId.eq(guardian_id))
        .all(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn find_guardian_by_guardian_id_and_account_id(
    db: &DatabaseConnection,
    guardian_id: String,
    account_id: String,
) -> anyhow::Result<Option<Model>> {
    Entity::find()
        .filter(Column::GuardianId.eq(guardian_id))
        .filter(Column::AccountId.eq(account_id))
        .one(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn find_all_accounts_by_guardian_id_and_account_id(
    db: &DatabaseConnection,
    guardian_id: String,
    account_id: String,
) -> anyhow::Result<Vec<Model>> {
    Entity::find()
        .filter(Column::GuardianId.eq(guardian_id))
        .filter(Column::AccountId.eq(account_id))
        .all(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn find_all_guardians_by_account_id(
    db: &DatabaseConnection,
    account_id: String,
) -> anyhow::Result<Vec<Model>> {
    Entity::find()
        .filter(Column::AccountId.eq(account_id))
        .all(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn find_all_guardians_by_account_id_and_status(
    db: &DatabaseConnection,
    account_id: String,
    status: String,
) -> anyhow::Result<Vec<Model>> {
    Entity::find()
        .filter(Column::AccountId.eq(account_id))
        .filter(Column::Status.eq(status))
        .all(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn find_all_active_guardians_by_account_id(
    db: &DatabaseConnection,
    account_id: String,
) -> anyhow::Result<Vec<Model>> {
    find_all_guardians_by_account_id_and_status(db, account_id, "ACTIVE".to_owned()).await
}

pub async fn delete_by_id(db: &DatabaseConnection, id: String) -> anyhow::Result<()> {
    Entity::delete_many()
        .filter(Column::Id.eq(id))
        .exec(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
        .map(|_| ())
}

pub async fn find_all_guardians_for_account_by_ids(
    db: &DatabaseConnection,
    ids: Vec<String>,
    account_id: String,
) -> anyhow::Result<Vec<Model>> {
    Entity::find()
        .filter(Column::AccountId.eq(account_id))
        .filter(Column::Id.is_in(ids))
        .all(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn update_all_guardians_for_account_to_status(
    db: &DatabaseConnection,
    account_id: String,
    status: String,
) -> anyhow::Result<()> {
    Entity::update_many()
        .filter(Column::AccountId.eq(account_id))
        .col_expr(Column::Status, Expr::value(status.to_owned()))
        .exec(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
        .map(|_| ())
}

pub async fn update_guardians_for_account_to_status(
    db: &DatabaseConnection,
    account_id: String,
    ids: Vec<String>,
    status: String,
) -> anyhow::Result<()> {
    Entity::update_many()
        .filter(Column::AccountId.eq(account_id))
        .filter(Column::Id.is_in(ids))
        .col_expr(Column::Status, Expr::value(status.to_owned()))
        .exec(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
        .map(|_| ())
}
