// SPDX-FileCopyrightText: 2026 Strike Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;

const MAX_FETCH_ATTEMPTS: u32 = 4;

#[derive(Debug, serde::Deserialize)]
/// EDINET API から取得した日次書類メタデータ一覧。
pub struct FetchedDocumentMetadatas {
    pub results: Vec<crate::updater::document_metadata::DocumentMetadata>,
}

/// 指定日の EDINET 日次書類メタデータ一覧を取得する。
pub async fn fetch_document_metadatas(
    date: &chrono::NaiveDate,
    api_key: &str,
) -> anyhow::Result<FetchedDocumentMetadatas> {
    let client = reqwest::Client::new();
    fetch_document_metadatas_with_client(&client, date, api_key).await
}

/// 共有 HTTP クライアントを使って日次書類メタデータ一覧を取得する。
pub async fn fetch_document_metadatas_with_client(
    client: &reqwest::Client,
    date: &chrono::NaiveDate,
    api_key: &str,
) -> anyhow::Result<FetchedDocumentMetadatas> {
    let url = format!(
        "https://api.edinet-fsa.go.jp/api/v2/documents.json?date={}&type=2&Subscription-Key={}",
        date.format("%Y-%m-%d"),
        api_key
    );

    for attempt in 0..MAX_FETCH_ATTEMPTS {
        let response = client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("failed to fetch document metadatas for {}", date))?;
        let status = response.status();

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS && attempt + 1 < MAX_FETCH_ATTEMPTS {
            let delay = retry_delay(attempt);
            eprintln!(
                "EDINET API rate limit reached for {}; retrying in {} seconds",
                date,
                delay.as_secs()
            );
            tokio::time::sleep(delay).await;
            continue;
        }

        let response_body = response
            .text()
            .await
            .with_context(|| format!("failed to read response body for {}", date))?;

        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "failed to fetch document metadatas for {}: HTTP {} body: {}",
                date,
                status.as_u16(),
                response_body
            ));
        }

        return serde_json::from_str::<FetchedDocumentMetadatas>(&response_body)
            .with_context(|| format!("failed to deserialize document metadatas for {}", date));
    }

    Err(anyhow::anyhow!(
        "failed to fetch document metadatas for {} after {} attempts",
        date,
        MAX_FETCH_ATTEMPTS
    ))
}

fn retry_delay(attempt: u32) -> std::time::Duration {
    std::time::Duration::from_secs(1_u64 << attempt)
}

#[cfg(test)]
mod tests {
    use crate::updater::fetch_document_metadatas::{FetchedDocumentMetadatas, retry_delay};

    #[test]
    fn test_deserialize_document_metadatas_response() -> anyhow::Result<()> {
        let json = r#"
        {
          "results": [
            {
              "seqNumber": 1,
              "docID": "S100TEST",
              "edinetCode": "E00001",
              "secCode": "12340",
              "JCN": "1234567890123",
              "filerName": "テスト株式会社",
              "ordinanceCode": "010",
              "formCode": "030000",
              "docTypeCode": "120",
              "periodStart": "2025-04-01",
              "periodEnd": "2026-03-31",
              "submitDateTime": "2026-04-01 09:00",
              "docDescription": "有価証券報告書",
              "withdrawalStatus": "0",
              "docInfoEditStatus": "0",
              "disclosureStatus": "0",
              "xbrlFlag": "1",
              "pdfFlag": "1",
              "attachDocFlag": "1",
              "englishDocFlag": "0",
              "csvFlag": "1",
              "legalStatus": "1"
            }
          ]
        }
        "#;

        let response = serde_json::from_str::<FetchedDocumentMetadatas>(json)?;
        assert_eq!(response.results.len(), 1);
        assert_eq!(response.results[0].doc_id, "S100TEST");
        Ok(())
    }

    #[test]
    fn retry_delay_uses_exponential_backoff() {
        assert_eq!(retry_delay(0), std::time::Duration::from_secs(1));
        assert_eq!(retry_delay(1), std::time::Duration::from_secs(2));
        assert_eq!(retry_delay(2), std::time::Duration::from_secs(4));
    }
}
