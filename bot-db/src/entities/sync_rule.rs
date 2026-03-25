use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "sync_rules")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub source_platform: String,
    pub source_room: String,
    pub target_platform: String,
    pub target_room: String,
    pub direction: String,
    pub sync_members: i64,
    pub enabled: i64,
    pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::sync_message::Entity")]
    Messages,
}

impl Related<super::sync_message::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Messages.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
