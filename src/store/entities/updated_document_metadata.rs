use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "updated_document_metadatas_list")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub file_date: String,
    pub updated_at: String,
    pub result_count: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
