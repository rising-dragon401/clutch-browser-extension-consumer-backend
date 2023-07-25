use sea_orm::entity::prelude::*;
use sea_orm::sea_query::Expr;
use sea_orm::Set;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "nominations")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub email: String,
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
    email: String,
    account_id: String,
    guardian_id: String,
    status: String,
) -> anyhow::Result<()> {
    let model = ActiveModel {
        id: Set(id.to_string()),
        email: Set(email.to_owned()),
        account_id: Set(account_id.to_owned()),
        guardian_id: Set(guardian_id.to_owned()),
        status: Set(status.to_owned()),
    };

    Entity::insert(model)
        .exec_without_returning(db)
        .await
        .map(|_| ())
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn find_by_account_and_id(
    db: &DatabaseConnection,
    account_id: String,
    nomination_id: String,
) -> anyhow::Result<Option<Model>> {
    Entity::find()
        .filter(Column::AccountId.eq(account_id))
        .filter(Column::Id.eq(nomination_id))
        .one(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn delete_by_account_and_id(
    db: &DatabaseConnection,
    account_id: String,
    nomination_id: String,
) -> anyhow::Result<()> {
    Entity::delete_many()
        .filter(Column::AccountId.eq(account_id))
        .filter(Column::Id.eq(nomination_id))
        .exec(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
        .map(|_| ())
}

pub async fn find_all_by_account(
    db: &DatabaseConnection,
    account_id: String,
) -> anyhow::Result<Vec<Model>> {
    Entity::find()
        .filter(Column::AccountId.eq(account_id))
        .all(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn find_all_by_account_and_id(
    db: &DatabaseConnection,
    account_id: String,
    nomination_id: String,
) -> anyhow::Result<Vec<Model>> {
    Entity::find()
        .filter(Column::AccountId.eq(account_id))
        .filter(Column::Id.eq(nomination_id))
        .all(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn find_all_by_account_and_status(
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

pub async fn find_all_by_account_and_email(
    db: &DatabaseConnection,
    account_id: String,
    email: String,
) -> anyhow::Result<Vec<Model>> {
    Entity::find()
        .filter(Column::AccountId.eq(account_id))
        .filter(Column::Email.eq(email))
        .all(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn find_all_by_guardian(
    db: &DatabaseConnection,
    guardian_id: String,
) -> anyhow::Result<Vec<Model>> {
    Entity::find()
        .filter(Column::GuardianId.eq(guardian_id))
        .all(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn find_all_by_guardian_and_nomination_id(
    db: &DatabaseConnection,
    guardian_id: String,
    nomination_id: String,
) -> anyhow::Result<Vec<Model>> {
    Entity::find()
        .filter(Column::GuardianId.eq(guardian_id))
        .filter(Column::Id.eq(nomination_id))
        .all(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn find_all_by_guardian_and_status(
    db: &DatabaseConnection,
    guardian_id: String,
    status: String,
) -> anyhow::Result<Vec<Model>> {
    Entity::find()
        .filter(Column::GuardianId.eq(guardian_id))
        .filter(Column::Status.eq(status))
        .all(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn update_status_by_guardian_id(
    db: &DatabaseConnection,
    nomination_id: String,
    guardian_id: String,
    status: String,
) -> anyhow::Result<()> {
    Entity::update_many()
        .col_expr(Column::Status, Expr::value(status))
        .filter(Column::Id.eq(nomination_id))
        .filter(Column::GuardianId.eq(guardian_id))
        .exec(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
        .map(|_| ())
}
