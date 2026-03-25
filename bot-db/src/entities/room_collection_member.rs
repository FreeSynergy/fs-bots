use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "room_collection_members")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub collection_id: i64,
    #[sea_orm(primary_key)]
    pub platform: String,
    #[sea_orm(primary_key)]
    pub room_id: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::room_collection::Entity",
        from = "Column::CollectionId",
        to = "super::room_collection::Column::Id",
        on_delete = "Cascade"
    )]
    Collection,
}

impl Related<super::room_collection::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Collection.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
