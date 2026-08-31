// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

use crate::store::entities::updated_document_metadata;

const DEFAULT_UPDATE_WINDOW_DAYS: i64 = 365;

/// 直近 1 年を更新する。通常は未更新日のみ、`force` 指定時は全日を対象とする。
pub async fn update_recent_days(
    db: &DatabaseConnection,
    today: &chrono::NaiveDate,
    api_key: &str,
    concurrency: usize,
    force: bool,
) -> anyhow::Result<()> {
    let start_date = today
        .checked_sub_signed(chrono::Duration::days(DEFAULT_UPDATE_WINDOW_DAYS - 1))
        .context("failed to calculate default update start date")?;
    update_days(db, &start_date, today, api_key, concurrency, force).await
}

/// 指定範囲を更新する。通常は未更新日のみ、`force` 指定時は全日を対象とする。
pub async fn update_days(
    db: &DatabaseConnection,
    start_date: &chrono::NaiveDate,
    end_date: &chrono::NaiveDate,
    api_key: &str,
    concurrency: usize,
    force: bool,
) -> anyhow::Result<()> {
    if force {
        update_range(db, start_date, end_date, api_key, concurrency).await
    } else {
        update_missing_days(db, start_date, end_date, api_key, concurrency).await
    }
}

/// 指定範囲のうち、まだ更新していない日付だけを順に更新する。
pub async fn update_missing_days(
    db: &DatabaseConnection,
    start_date: &chrono::NaiveDate,
    end_date: &chrono::NaiveDate,
    api_key: &str,
    concurrency: usize,
) -> anyhow::Result<()> {
    if start_date > end_date {
        return Err(anyhow::anyhow!(
            "start_date must be earlier than or equal to end_date"
        ));
    }
    let missing_dates = find_missing_update_dates(db, start_date, end_date).await?;

    if missing_dates.is_empty() {
        println!(
            "No missing update dates found from {} to {}",
            start_date, end_date
        );
        return Ok(());
    }

    println!(
        "Updating {} missing dates from {} to {}",
        missing_dates.len(),
        start_date,
        end_date
    );
    update_dates(db, missing_dates, api_key, concurrency).await
}

/// 指定した日付範囲を 1 日ずつ更新する。
pub async fn update_range(
    db: &DatabaseConnection,
    start_date: &chrono::NaiveDate,
    end_date: &chrono::NaiveDate,
    api_key: &str,
    concurrency: usize,
) -> anyhow::Result<()> {
    if start_date > end_date {
        return Err(anyhow::anyhow!(
            "start_date must be earlier than or equal to end_date"
        ));
    }

    let dates = dates_in_range(start_date, end_date)?;
    update_dates(db, dates, api_key, concurrency).await
}

/// 指定日の EDINET 書類メタデータを取得して DB に保存する。
pub async fn update_one_day(
    db: &DatabaseConnection,
    file_date: &chrono::NaiveDate,
    api_key: &str,
) -> anyhow::Result<()> {
    println!("Updating document metadata for {}", file_date);
    let fetched_document_metadatas =
        crate::updater::fetch_document_metadatas::fetch_document_metadatas(file_date, api_key)
            .await?;

    crate::updater::save_document_metadatas::save_document_metadatas(
        db,
        file_date,
        &fetched_document_metadatas,
    )
    .await
    .with_context(|| format!("failed to save document metadata for {}", file_date))?;

    println!(
        "Saved {} documents for {}",
        fetched_document_metadatas.results.len(),
        file_date
    );

    Ok(())
}

/// 日次 API の取得を限定並列化し、取得できたデータを順に DB へ保存する。
async fn update_dates(
    db: &DatabaseConnection,
    dates: Vec<chrono::NaiveDate>,
    api_key: &str,
    concurrency: usize,
) -> anyhow::Result<()> {
    if concurrency == 0 {
        return Err(anyhow::anyhow!("concurrency must be 1 or greater"));
    }

    let client = reqwest::Client::new();
    let mut pending_dates = dates.into_iter();
    let mut fetch_tasks = tokio::task::JoinSet::new();

    for _ in 0..concurrency {
        let Some(file_date) = pending_dates.next() else {
            break;
        };
        spawn_fetch_task(&mut fetch_tasks, &client, file_date, api_key);
    }

    while let Some(task_result) = fetch_tasks.join_next().await {
        let (file_date, fetched) = task_result.context("document metadata fetch task failed")?;
        let fetched = fetched.with_context(|| format!("failed to update {}", file_date))?;

        crate::updater::save_document_metadatas::save_document_metadatas(db, &file_date, &fetched)
            .await
            .with_context(|| format!("failed to save document metadata for {}", file_date))?;

        println!(
            "Saved {} documents for {}",
            fetched.results.len(),
            file_date
        );

        if let Some(next_date) = pending_dates.next() {
            spawn_fetch_task(&mut fetch_tasks, &client, next_date, api_key);
        }
    }

    Ok(())
}

fn spawn_fetch_task(
    tasks: &mut tokio::task::JoinSet<(
        chrono::NaiveDate,
        anyhow::Result<crate::updater::fetch_document_metadatas::FetchedDocumentMetadatas>,
    )>,
    client: &reqwest::Client,
    file_date: chrono::NaiveDate,
    api_key: &str,
) {
    let client = client.clone();
    let api_key = api_key.to_owned();
    tasks.spawn(async move {
        println!("Fetching document metadata for {}", file_date);
        let result =
            crate::updater::fetch_document_metadatas::fetch_document_metadatas_with_client(
                &client, &file_date, &api_key,
            )
            .await;
        (file_date, result)
    });
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

#[cfg(test)]
mod tests {
    use super::dates_in_range;

    #[test]
    fn builds_inclusive_date_range() {
        let start = chrono::NaiveDate::from_ymd_opt(2026, 1, 30).unwrap();
        let end = chrono::NaiveDate::from_ymd_opt(2026, 2, 1).unwrap();

        assert_eq!(
            dates_in_range(&start, &end).unwrap(),
            vec![
                chrono::NaiveDate::from_ymd_opt(2026, 1, 30).unwrap(),
                chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
                chrono::NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
            ]
        );
    }

    #[test]
    fn returns_empty_for_reversed_date_range() {
        let start = chrono::NaiveDate::from_ymd_opt(2026, 2, 1).unwrap();
        let end = chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();

        assert!(dates_in_range(&start, &end).unwrap().is_empty());
    }
}
