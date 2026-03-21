use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "join_requests")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id:          i64,
    pub platform:    String,
    pub room_id:     String,
    pub user_id:     String,
    pub status:      String,
    pub iam_result:  Option<String>,
    pub created_at:  String,
    pub resolved_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
