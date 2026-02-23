use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table_name = "rustpbx_call_records";

        if !manager
            .has_column(table_name, "quality_score")
            .await?
        {
            manager
                .alter_table(
                    Table::alter()
                        .table(crate::models::call_record::Entity)
                        .add_column(
                            ColumnDef::new(crate::models::call_record::Column::QualityScore)
                                .double()
                                .null(),
                        )
                        .to_owned(),
                )
                .await?;
        }

        if !manager
            .has_column(table_name, "packet_loss_pct")
            .await?
        {
            manager
                .alter_table(
                    Table::alter()
                        .table(crate::models::call_record::Entity)
                        .add_column(
                            ColumnDef::new(crate::models::call_record::Column::PacketLossPct)
                                .double()
                                .null(),
                        )
                        .to_owned(),
                )
                .await?;
        }

        if !manager
            .has_column(table_name, "avg_jitter_ms")
            .await?
        {
            manager
                .alter_table(
                    Table::alter()
                        .table(crate::models::call_record::Entity)
                        .add_column(
                            ColumnDef::new(crate::models::call_record::Column::AvgJitterMs)
                                .double()
                                .null(),
                        )
                        .to_owned(),
                )
                .await?;
        }

        if !manager
            .has_column(table_name, "total_rtp_packets")
            .await?
        {
            manager
                .alter_table(
                    Table::alter()
                        .table(crate::models::call_record::Entity)
                        .add_column(
                            ColumnDef::new(crate::models::call_record::Column::TotalRtpPackets)
                                .big_integer()
                                .null(),
                        )
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
