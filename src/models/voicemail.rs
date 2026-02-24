use sea_orm::entity::prelude::*;
use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::{
    boolean, integer, string, string_null, text_null, timestamp, timestamp_null,
};
use sea_orm_migration::sea_query::ColumnDef;
use sea_query::Expr;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "rustpbx_voicemails")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub mailbox_id: String,
    pub caller_id: String,
    pub caller_name: Option<String>,
    pub call_id: String,
    pub recording_path: String,
    pub duration_secs: i32,
    pub is_read: bool,
    pub is_urgent: bool,
    #[sea_orm(column_type = "Text", nullable)]
    pub transcript_text: Option<String>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub deleted_at: Option<DateTimeUtc>,
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
                    .col(string(Column::CallerId).char_len(64))
                    .col(string_null(Column::CallerName).char_len(160))
                    .col(string(Column::CallId).char_len(120))
                    .col(string(Column::RecordingPath).char_len(512))
                    .col(integer(Column::DurationSecs).not_null().default(0))
                    .col(boolean(Column::IsRead).default(false))
                    .col(boolean(Column::IsUrgent).default(false))
                    .col(text_null(Column::TranscriptText))
                    .col(timestamp(Column::CreatedAt).default(Expr::current_timestamp()))
                    .col(timestamp(Column::UpdatedAt).default(Expr::current_timestamp()))
                    .col(timestamp_null(Column::DeletedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_rustpbx_voicemails_mailbox_id")
                    .table(Entity)
                    .col(Column::MailboxId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_rustpbx_voicemails_call_id")
                    .table(Entity)
                    .col(Column::CallId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_rustpbx_voicemails_created_at")
                    .table(Entity)
                    .col(Column::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_rustpbx_voicemails_mailbox_unread")
                    .table(Entity)
                    .col(Column::MailboxId)
                    .col(Column::IsRead)
                    .col(Column::DeletedAt)
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
