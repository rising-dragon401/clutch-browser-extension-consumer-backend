use sea_orm::Set;
use sea_orm::{entity::prelude::*, sea_query::Expr};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "accounts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub email: String,
    pub wallet_address: String,
    pub eoa_address: String,
    pub eoa_private_address: String,
    pub updated_at: i64,
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
    wallet_address: String,
    eoa_address: String,
    eoa_private: String,
    updated_at: i64,
) -> anyhow::Result<()> {
    let model = ActiveModel {
        id: Set(id.to_string()),
        email: Set(email.to_owned()),
        wallet_address: Set(wallet_address.to_owned()),
        eoa_address: Set(eoa_address.to_owned()),
        eoa_private_address: Set(eoa_private.to_owned()),
        updated_at: Set(updated_at.to_owned()),
    };

    Entity::insert(model)
        .exec_without_returning(db)
        .await
        .map(|_| ())
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn find_by_id(
    db: &DatabaseConnection,
    account_id: String,
) -> anyhow::Result<Option<Model>> {
    Entity::find()
        .filter(Column::Id.eq(account_id))
        .one(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn find_by_email(db: &DatabaseConnection, email: &str) -> anyhow::Result<Option<Model>> {
    Entity::find()
        .filter(Column::Email.eq(email))
        .one(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn find_by_wallet_address(
    db: &DatabaseConnection,
    wallet_address: String,
) -> anyhow::Result<Option<Model>> {
    Entity::find()
        .filter(Column::WalletAddress.eq(wallet_address))
        .one(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn find_all_by_wallet_address(
    db: &DatabaseConnection,
    wallet_address: String,
) -> anyhow::Result<Vec<Model>> {
    Entity::find()
        .filter(Column::WalletAddress.eq(wallet_address))
        .all(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn find_all_by_eoa_address(
    db: &DatabaseConnection,
    eao_address: String,
) -> anyhow::Result<Vec<Model>> {
    Entity::find()
        .filter(Column::EoaAddress.eq(eao_address))
        .all(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn find_all_by_email_address(
    db: &DatabaseConnection,
    email: String,
) -> anyhow::Result<Vec<Model>> {
    Entity::find()
        .filter(Column::Email.eq(email))
        .all(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn find_all_by_account_ids(
    db: &DatabaseConnection,
    account_ids: Vec<String>,
) -> anyhow::Result<Vec<Model>> {
    Entity::find()
        .filter(Column::Id.is_in(account_ids))
        .all(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn find_all(db: &DatabaseConnection) -> anyhow::Result<Vec<Model>> {
    Entity::find().all(db).await.map_err(|e| anyhow::anyhow!(e))
}

pub async fn update(
    db: &DatabaseConnection,
    id: String,
    wallet_address: Option<String>,
    eoa_address: Option<String>,
) -> anyhow::Result<()> {
    Entity::update_many()
        .col_expr(
            Column::WalletAddress,
            Expr::value(wallet_address.unwrap_or_default()),
        )
        .col_expr(
            Column::EoaAddress,
            Expr::value(eoa_address.unwrap_or_default()),
        )
        .filter(Column::Id.eq(id))
        .exec(db)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(())
}
