use sea_orm_migration::prelude::*;

/// Combined migration that creates both voicemail tables.
/// Each individual table migration is defined in its own module
/// (voicemail.rs and voicemail_greeting.rs). This migration delegates
/// to both so they share a single migration entry in the registry.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create voicemails table
        super::voicemail::Migration.up(manager).await?;
        // Create voicemail greetings table
        super::voicemail_greeting::Migration.up(manager).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop in reverse order
        super::voicemail_greeting::Migration.down(manager).await?;
        super::voicemail::Migration.down(manager).await?;
        Ok(())
    }
}
