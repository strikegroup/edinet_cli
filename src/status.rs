// SPDX-FileCopyrightText: 2026 Strike Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};

use crate::store::entities::{document_metadata, updated_document_metadata};

const DEFAULT_STATUS_WINDOW_DAYS: i64 = 365;

/// 保存済みデータと CSV キャッシュの状態を標準出力に表示する。
pub async fn print_status(db: Option<&DatabaseConnection>) -> anyhow::Result<()> {
    let database_path = crate::app_paths::current_database_path()?;
    let csv_cache_dir = crate::app_paths::current_csv_cache_dir()?;
    let api_key_status = crate::app_config::api_key_registration_status()?;
    let today = chrono::Local::now().date_naive();
    let default_window_start = today
        .checked_sub_signed(chrono::Duration::days(DEFAULT_STATUS_WINDOW_DAYS - 1))
        .context("failed to calculate default status start date")?;
    let db_status = match db {
        Some(db) => collect_database_status(db, &default_window_start, &today).await?,
        None => DatabaseStatus {
            missing_recent_dates: dates_in_range(&default_window_start, &today)?,
            ..Default::default()
        },
    };
    let csv_cache_status = inspect_csv_cache(&csv_cache_dir)?;

    println!("Document index: {}", database_path.display());
    println!("CSV cache: {}", csv_cache_dir.display());
    println!("API key: {}", api_key_status.as_str());
    println!();
    println!(
        "Document index exists: {}",
        if db.is_some() { "yes" } else { "no" }
    );
    println!("Updated dates: {}", db_status.updated_date_count);
    if let Some((min_file_date, max_file_date)) = db_status.update_range {
        println!("Updated range: {} .. {}", min_file_date, max_file_date);
    } else {
        println!("Updated range: none");
    }
    if let Some(latest_updated_at) = db_status.latest_updated_at {
        println!("Latest updated at: {}", latest_updated_at);
    } else {
        println!("Latest updated at: none");
    }
    println!(
        "Missing dates in recent {} days: {}",
        DEFAULT_STATUS_WINDOW_DAYS,
        db_status.missing_recent_dates.len()
    );
    if !db_status.missing_recent_dates.is_empty() {
        println!(
            "Missing date examples: {}",
            format_date_examples(&db_status.missing_recent_dates, 8)
        );
    }
    println!("Indexed documents: {}", db_status.item_count);
    println!(
        "Annual security reports in indexed documents: {}",
        db_status.available_asr_count
    );
    println!();
    println!("Cached documents: {}", csv_cache_status.doc_dirs);
    println!(
        "CSV files in cached documents: {}",
        csv_cache_status.csv_files
    );
    println!(
        "CSV cache size: {}",
        format_bytes(csv_cache_status.total_bytes)
    );

    Ok(())
}

pub async fn run() -> anyhow::Result<()> {
    let db = crate::store::open_db::open_existing_db_connection().await?;
    print_status(db.as_ref()).await
}

#[derive(Debug, Default)]
struct DatabaseStatus {
    updated_date_count: u64,
    item_count: u64,
    available_asr_count: u64,
    update_range: Option<(String, String)>,
    latest_updated_at: Option<String>,
    missing_recent_dates: Vec<chrono::NaiveDate>,
}

async fn collect_database_status(
    db: &DatabaseConnection,
    default_window_start: &chrono::NaiveDate,
    today: &chrono::NaiveDate,
) -> anyhow::Result<DatabaseStatus> {
    let updated_date_count = updated_document_metadata::Entity::find()
        .count(db)
        .await
        .context("failed to count updated_document_metadatas_list")?;
    let item_count = document_metadata::Entity::find()
        .count(db)
        .await
        .context("failed to count document_metadatas")?;
    let available_asr_count = document_metadata::Entity::find()
        .filter(available_asr_condition())
        .count(db)
        .await
        .context("failed to count available ASR documents")?;
    let update_range = find_update_range(db).await?;
    let latest_updated_at = find_latest_updated_at(db).await?;
    let missing_recent_dates = find_missing_update_dates(db, default_window_start, today).await?;

    Ok(DatabaseStatus {
        updated_date_count,
        item_count,
        available_asr_count,
        update_range,
        latest_updated_at,
        missing_recent_dates,
    })
}

fn available_asr_condition() -> Condition {
    Condition::all()
        .add(document_metadata::Column::OrdinanceCode.eq("010"))
        .add(document_metadata::Column::DocTypeCode.eq("120"))
        .add(document_metadata::Column::CsvFlag.eq("1"))
        .add(document_metadata::Column::WithdrawalStatus.eq("0"))
        .add(document_metadata::Column::DisclosureStatus.eq("0"))
        .add(document_metadata::Column::LegalStatus.ne("0"))
        .add(document_metadata::Column::SubmitDateTime.is_not_null())
}

async fn find_update_range(db: &DatabaseConnection) -> anyhow::Result<Option<(String, String)>> {
    let first = updated_document_metadata::Entity::find()
        .select_only()
        .column(updated_document_metadata::Column::FileDate)
        .order_by_asc(updated_document_metadata::Column::FileDate)
        .into_tuple::<String>()
        .one(db)
        .await
        .context("failed to read first updated date")?;
    let last = updated_document_metadata::Entity::find()
        .select_only()
        .column(updated_document_metadata::Column::FileDate)
        .order_by_desc(updated_document_metadata::Column::FileDate)
        .into_tuple::<String>()
        .one(db)
        .await
        .context("failed to read last updated date")?;

    Ok(first.zip(last))
}

async fn find_latest_updated_at(db: &DatabaseConnection) -> anyhow::Result<Option<String>> {
    updated_document_metadata::Entity::find()
        .select_only()
        .column(updated_document_metadata::Column::UpdatedAt)
        .order_by_desc(updated_document_metadata::Column::UpdatedAt)
        .into_tuple::<String>()
        .one(db)
        .await
        .context("failed to read latest updated_at")
}

async fn find_missing_update_dates(
    db: &DatabaseConnection,
    start_date: &chrono::NaiveDate,
    end_date: &chrono::NaiveDate,
) -> anyhow::Result<Vec<chrono::NaiveDate>> {
    let start_date_string = start_date.to_string();
    let end_date_string = end_date.to_string();
    let updated_rows = updated_document_metadata::Entity::find()
        .filter(updated_document_metadata::Column::FileDate.gte(start_date_string))
        .filter(updated_document_metadata::Column::FileDate.lte(end_date_string))
        .order_by_asc(updated_document_metadata::Column::FileDate)
        .all(db)
        .await
        .context("failed to read updated document metadata dates")?;
    let updated_dates = updated_rows
        .into_iter()
        .map(|row| row.file_date)
        .collect::<std::collections::HashSet<_>>();

    let mut missing_dates = Vec::new();
    let mut current_date = *start_date;
    while current_date <= *end_date {
        if !updated_dates.contains(&current_date.to_string()) {
            missing_dates.push(current_date);
        }
        current_date = current_date
            .succ_opt()
            .context("failed to increment current date")?;
    }

    Ok(missing_dates)
}

#[derive(Debug, Default)]
struct CsvCacheStatus {
    doc_dirs: u64,
    csv_files: u64,
    total_bytes: u64,
}

fn inspect_csv_cache(path: &std::path::Path) -> anyhow::Result<CsvCacheStatus> {
    if !path.exists() {
        return Ok(CsvCacheStatus::default());
    }
    if !path.is_dir() {
        return Err(anyhow::anyhow!(
            "CSV cache path is not a directory: {}",
            path.display()
        ));
    }

    let mut status = CsvCacheStatus::default();
    for entry in std::fs::read_dir(path)
        .with_context(|| format!("failed to read CSV cache directory {}", path.display()))?
    {
        let entry = entry.context("failed to read CSV cache directory entry")?;
        let metadata = entry
            .metadata()
            .context("failed to read CSV cache entry metadata")?;
        if metadata.is_dir() {
            status.doc_dirs += 1;
            inspect_cache_tree(&entry.path(), &mut status)?;
        } else if metadata.is_file() {
            status.total_bytes += metadata.len();
            if is_csv_path(&entry.path()) {
                status.csv_files += 1;
            }
        }
    }

    Ok(status)
}

fn inspect_cache_tree(path: &std::path::Path, status: &mut CsvCacheStatus) -> anyhow::Result<()> {
    for entry in std::fs::read_dir(path)
        .with_context(|| format!("failed to read cache directory {}", path.display()))?
    {
        let entry = entry.context("failed to read cache directory entry")?;
        let metadata = entry
            .metadata()
            .context("failed to read cache entry metadata")?;
        if metadata.is_dir() {
            inspect_cache_tree(&entry.path(), status)?;
        } else if metadata.is_file() {
            status.total_bytes += metadata.len();
            if is_csv_path(&entry.path()) {
                status.csv_files += 1;
            }
        }
    }
    Ok(())
}

fn is_csv_path(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("csv"))
}

fn format_date_examples(dates: &[chrono::NaiveDate], limit: usize) -> String {
    let mut values = dates
        .iter()
        .take(limit)
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    if dates.len() > limit {
        values.push(format!("... and {} more", dates.len() - limit));
    }
    values.join(", ")
}

fn dates_in_range(
    start_date: &chrono::NaiveDate,
    end_date: &chrono::NaiveDate,
) -> anyhow::Result<Vec<chrono::NaiveDate>> {
    let mut dates = Vec::new();
    let mut current_date = *start_date;
    while current_date <= *end_date {
        dates.push(current_date);
        current_date = current_date
            .succ_opt()
            .context("failed to increment current date")?;
    }
    Ok(dates)
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit_index = 0;
    while value >= 1024.0 && unit_index + 1 < UNITS.len() {
        value /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", value, UNITS[unit_index])
    }
}
