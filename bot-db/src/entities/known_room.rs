use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "known_rooms")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id:           i64,
    pub platform:     String,
    pub room_id:      String,
    pub room_name:    Option<String>,
    pub member_count: Option<i64>,
    pub last_seen:    String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
