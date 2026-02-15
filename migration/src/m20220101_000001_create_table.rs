use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(pk_auto(Users::Id))
                    .col(string(Users::Username).not_null())
                    .col(string(Users::Email).not_null().unique_key())
                    .col(string(Users::Password).not_null())
                    .col(date_time(Users::JoinedAt).not_null().default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())))
                    .to_owned(),
            )
            .await;

        // Create events table
        manager
            .create_table(
                Table::create()
                    .table(Events::Table)
                    .if_not_exists()
                    .col(pk_auto(Events::Id))
                    .col(string(Events::Title).not_null())
                    .col(string(Events::Location).not_null())
                    .col(date_time(Events::CreatedAt).not_null().default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())))
                    .col(string(Events::Category).not_null())
                    .col(string(Events::Date).not_null())
                    .col(string(Events::Url).not_null())
                    .col(integer(Events::UserId).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_events_user")
                            .from(Events::Table, Events::UserId)
                            .to(Users::Table, Users::Id)
                    )
                    .to_owned(),
            )
            .await?;

        // Create user_events table (junction table)
        manager
            .create_table(
                Table::create()
                    .table(UserEvents::Table)
                    .if_not_exists()
                    .col(pk_auto(UserEvents::Id))
                    .col(integer(UserEvents::UserId).not_null())
                    .col(integer(UserEvents::EventId).not_null())
                    // Foreign keys (optional, but recommended)
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_events_user")
                            .from(UserEvents::Table, UserEvents::UserId)
                            .to(Users::Table, Users::Id)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_events_event")
                            .from(UserEvents::Table, UserEvents::EventId)
                            .to(Events::Table, Events::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables in reverse order
        manager
            .drop_table(Table::drop().table(UserEvents::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Events::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Username,
    Email,
    Password,
    JoinedAt,
}

#[derive(DeriveIden)]
enum Events {
    Table,
    Id,
    Title,
    Location,
    Category,
    CreatedAt,
    Url,
    Date,
    UserId,
}

#[derive(DeriveIden)]
enum UserEvents {
    Table,
    Id,
    UserId,
    EventId,
    RegisteredAt,
}
