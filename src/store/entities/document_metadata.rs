use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "document_metadatas")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub file_date: String,
    pub seq_number: i64,
    pub doc_id: String,
    pub edinet_code: Option<String>,
    pub sec_code: Option<String>,
    pub jcn: Option<String>,
    pub filer_name: Option<String>,
    pub ordinance_code: Option<String>,
    pub form_code: Option<String>,
    pub doc_type_code: Option<String>,
    pub period_start: Option<String>,
    pub period_end: Option<String>,
    pub submit_date_time: Option<String>,
    pub doc_description: Option<String>,
    pub withdrawal_status: String,
    pub doc_info_edit_status: String,
    pub disclosure_status: String,
    pub xbrl_flag: String,
    pub pdf_flag: String,
    pub attach_doc_flag: String,
    pub english_doc_flag: String,
    pub csv_flag: String,
    pub legal_status: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
