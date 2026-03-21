use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "room_collections")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id:          i64,
    pub name:        String,
    pub description: Option<String>,
    pub created_at:  String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::room_collection_member::Entity")]
    Members,
}

impl Related<super::room_collection_member::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Members.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
