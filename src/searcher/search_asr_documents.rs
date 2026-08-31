// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
};

use crate::store::asr_document_metadata::AsrDocumentMetadata;
use crate::store::entities::document_metadata;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SearchSort {
    SubmitDateDesc,
    SubmitDateAsc,
}

#[derive(Debug)]
pub struct SearchCondition {
    pub query: Option<String>,
    pub query_sec_code: Option<String>,
    pub edinet_code: Option<String>,
    pub sec_code: Option<String>,
    pub jcn: Option<String>,
    pub filer_name: Option<String>,
    pub submitted_date: Option<String>,
    pub submitted_from: Option<String>,
    pub submitted_to: Option<String>,
    pub submitted_year: Option<u16>,
    pub limit: Option<u64>,
    pub offset: u64,
    pub sort: SearchSort,
}

pub async fn search_asr_document_metadatas(
    db: &DatabaseConnection,
    condition: &SearchCondition,
) -> anyhow::Result<Vec<AsrDocumentMetadata>> {
    let mut filters = available_asr_condition()
        .add(document_metadata::Column::WithdrawalStatus.eq("0"))
        .add(document_metadata::Column::DisclosureStatus.eq("0"))
        .add(document_metadata::Column::LegalStatus.ne("0"))
        .add(document_metadata::Column::SubmitDateTime.is_not_null());

    if let Some(query) = &condition.query {
        let mut query_condition = Condition::any()
            .add(document_metadata::Column::FilerName.like(format!("%{query}%")))
            .add(document_metadata::Column::EdinetCode.eq(query))
            .add(document_metadata::Column::Jcn.eq(query));

        if let Some(query_sec_code) = &condition.query_sec_code {
            query_condition =
                query_condition.add(document_metadata::Column::SecCode.eq(query_sec_code));
        }

        filters = filters.add(query_condition);
    }
    if let Some(edinet_code) = &condition.edinet_code {
        filters = filters.add(document_metadata::Column::EdinetCode.eq(edinet_code));
    }
    if let Some(sec_code) = &condition.sec_code {
        filters = filters.add(document_metadata::Column::SecCode.eq(sec_code));
    }
    if let Some(jcn) = &condition.jcn {
        filters = filters.add(document_metadata::Column::Jcn.eq(jcn));
    }
    if let Some(filer_name) = &condition.filer_name {
        filters = filters.add(document_metadata::Column::FilerName.like(format!("%{filer_name}%")));
    }
    if let Some(submitted_date) = &condition.submitted_date {
        filters = filters
            .add(document_metadata::Column::SubmitDateTime.gte(format!("{submitted_date} 00:00")))
            .add(document_metadata::Column::SubmitDateTime.lte(format!("{submitted_date} 23:59")));
    }
    if let Some(submitted_from) = &condition.submitted_from {
        filters = filters
            .add(document_metadata::Column::SubmitDateTime.gte(format!("{submitted_from} 00:00")));
    }
    if let Some(submitted_to) = &condition.submitted_to {
        filters = filters
            .add(document_metadata::Column::SubmitDateTime.lte(format!("{submitted_to} 23:59")));
    }
    if let Some(submitted_year) = condition.submitted_year {
        let (start, end) = submission_year_bounds(submitted_year);
        filters = filters
            .add(document_metadata::Column::SubmitDateTime.gte(start))
            .add(document_metadata::Column::SubmitDateTime.lte(end));
    }

    let mut query = document_metadata::Entity::find()
        .filter(filters)
        .order_by_desc(document_metadata::Column::FileDate)
        .order_by_desc(document_metadata::Column::SeqNumber);

    if let Some(limit) = condition.limit {
        query = query.limit(limit).offset(condition.offset);
    }

    let rows = match condition.sort {
        SearchSort::SubmitDateDesc => {
            query
                .order_by_desc(document_metadata::Column::SubmitDateTime)
                .all(db)
                .await
        }
        SearchSort::SubmitDateAsc => {
            query
                .order_by_asc(document_metadata::Column::SubmitDateTime)
                .all(db)
                .await
        }
    }
    .context("failed to search ASR documents")?;

    Ok(rows.into_iter().map(AsrDocumentMetadata::from).collect())
}

fn submission_year_bounds(year: u16) -> (String, String) {
    (
        format!("{year:04}-01-01 00:00"),
        format!("{year:04}-12-31 23:59"),
    )
}

fn available_asr_condition() -> Condition {
    Condition::all()
        .add(document_metadata::Column::OrdinanceCode.eq("010"))
        .add(document_metadata::Column::DocTypeCode.eq("120"))
        .add(document_metadata::Column::CsvFlag.eq("1"))
}

#[cfg(test)]
mod tests {
    use super::submission_year_bounds;

    #[test]
    fn builds_full_submission_year_bounds() {
        assert_eq!(
            submission_year_bounds(2025),
            ("2025-01-01 00:00".to_owned(), "2025-12-31 23:59".to_owned(),)
        );
    }
}
