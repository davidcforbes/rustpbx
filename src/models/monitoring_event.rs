use sea_orm::entity::prelude::*;
use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::{string, text_null, timestamp};
use sea_orm_migration::sea_query::ColumnDef;
use sea_query::Expr;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "rustpbx_monitoring_events")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub session_id: String,
    pub monitor_user_id: String,
    pub agent_extension: String,
    /// One of: "monitor_start", "monitor_stop", "mode_change"
    pub event_type: String,
    /// One of: "silent_listen", "whisper", "barge"
    pub monitor_mode: String,
    pub timestamp: DateTimeUtc,
    #[sea_orm(column_type = "Text", nullable)]
    pub details: Option<String>,
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
                    .col(string(Column::SessionId).char_len(120))
                    .col(string(Column::MonitorUserId).char_len(100))
                    .col(string(Column::AgentExtension).char_len(64))
                    .col(string(Column::EventType).char_len(32))
                    .col(string(Column::MonitorMode).char_len(32))
                    .col(timestamp(Column::Timestamp).default(Expr::current_timestamp()))
                    .col(text_null(Column::Details))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_monitoring_events_session_id")
                    .table(Entity)
                    .col(Column::SessionId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_monitoring_events_monitor_user_id")
                    .table(Entity)
                    .col(Column::MonitorUserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_monitoring_events_timestamp")
                    .table(Entity)
                    .col(Column::Timestamp)
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
