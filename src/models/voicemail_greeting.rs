use sea_orm::entity::prelude::*;
use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::{boolean, string, timestamp};
use sea_orm_migration::sea_query::ColumnDef;
use sea_query::Expr;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "rustpbx_voicemail_greetings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub mailbox_id: String,
    pub greeting_type: String,
    pub recording_path: String,
    pub is_active: bool,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Entity)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Column::Id)
                            .big_integer()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(string(Column::MailboxId).char_len(32))
                    .col(string(Column::GreetingType).char_len(32))
                    .col(string(Column::RecordingPath).char_len(512))
                    .col(boolean(Column::IsActive).default(true))
                    .col(timestamp(Column::CreatedAt).default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_rustpbx_voicemail_greetings_mailbox_id")
                    .table(Entity)
                    .col(Column::MailboxId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_rustpbx_voicemail_greetings_mailbox_type")
                    .table(Entity)
                    .col(Column::MailboxId)
                    .col(Column::GreetingType)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Entity).to_owned())
            .await
    }
}
