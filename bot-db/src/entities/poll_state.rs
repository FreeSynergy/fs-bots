use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "poll_state")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub platform:    String,
    #[sea_orm(primary_key)]
    pub room_id:     String,
    pub last_offset: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
