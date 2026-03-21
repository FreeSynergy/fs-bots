use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "sync_messages")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id:           i64,
    pub rule_id:      i64,
    pub direction:    String,
    pub msg_id_src:   String,
    pub forwarded_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::sync_rule::Entity",
        from = "Column::RuleId",
        to   = "super::sync_rule::Column::Id",
        on_delete = "Cascade"
    )]
    Rule,
}

impl Related<super::sync_rule::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Rule.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
