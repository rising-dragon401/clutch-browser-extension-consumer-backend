use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "verifications")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub email: String,
    pub code: String,
    pub expires_at: i64,
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
    email: &str,
    code: &str,
    expires_at: i64,
) -> anyhow::Result<()> {
    let model = ActiveModel {
        id: Set(id.to_string()),
        email: Set(email.to_owned()),
        code: Set(code.to_owned()),
        expires_at: Set(expires_at.to_owned()),
    };

    Entity::insert(model)
        .exec_without_returning(db)
        .await
        .map(|_| ())
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn count_by_email(db: &DatabaseConnection, email: &str) -> anyhow::Result<u64> {
    Entity::find()
        .filter(Column::Email.eq(email))
        .count(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn find_all_by_email(
    db: &DatabaseConnection,
    email: String,
) -> anyhow::Result<Vec<Model>> {
    Entity::find()
        .filter(Column::Email.eq(email))
        .all(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn find_by_email_and_code(
    db: &DatabaseConnection,
    email: &str,
    code: &str,
) -> anyhow::Result<Option<Model>> {
    Entity::find()
        .filter(Column::Email.eq(email))
        .filter(Column::Code.eq(code))
        .one(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}
