// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use std::{collections::HashMap, sync::OnceLock};

/// 英語を正規形とする出力 JSON のキーを、日本語の表示名へ再帰的に翻訳する。
///
/// jq フィルタ適用後の値に対して呼び出すことで、フィルタ式は出力言語にかかわらず
/// 英語キーで記述できる。jq が生成した独自キーなど、辞書にないキーは変更しない。
pub fn translate_keys_to_ja(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Array(values) => {
            for value in values {
                translate_keys_to_ja(value);
            }
        }
        serde_json::Value::Object(object) => {
            let mut translated = serde_json::Map::new();
            for (key, mut value) in std::mem::take(object) {
                translate_keys_to_ja(&mut value);
                let translated_key = ja_key_translations()
                    .get(key.as_str())
                    .copied()
                    .unwrap_or(key.as_str());
                translated.insert(translated_key.to_owned(), value);
            }
            *object = translated;
        }
        _ => {}
    }
}

/// 正規 JSON の全キーと日本語表示名の一対一対応。
pub fn ja_key_translations() -> &'static HashMap<&'static str, &'static str> {
    static TRANSLATIONS: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();

    TRANSLATIONS.get_or_init(|| {
        HashMap::from([
            ("metadata", "書類情報"),
            ("report", "有価証券報告書"),
            ("file_date", "提出日"),
            ("doc_id", "書類ID"),
            ("edinet_code", "EDINETコード"),
            ("sec_code", "証券コード"),
            ("filer_name", "提出者名"),
            ("period_start", "事業年度開始日"),
            ("period_end", "事業年度終了日"),
            ("submit_date_time", "提出日時"),
            ("doc_description", "書類概要"),
            ("company_overview", "第1 企業の概況"),
            ("business_overview", "第2 事業の状況"),
            ("facilities", "第3 設備の状況"),
            ("corporate_information", "第4 提出会社の状況"),
            ("financial_information", "第5 経理の状況"),
            ("company_history", "沿革"),
            ("employees_overview", "従業員の状況"),
            ("business_results_summary", "主要な経営指標等の推移"),
            ("business_description", "事業の内容"),
            (
                "performance",
                "経営者による財政状態、経営成績及びキャッシュ・フローの状況の分析",
            ),
            (
                "issues_to_address",
                "経営方針、経営環境及び対処すべき課題等",
            ),
            ("risks", "事業等のリスク"),
            ("sustainability", "サステナビリティ関連情報"),
            ("governance", "ガバナンス"),
            ("strategy", "戦略"),
            ("risk_management", "リスク管理"),
            ("metrics_and_targets", "指標及び目標"),
            ("human_resources_policy", "人材育成方針・社内環境整備方針"),
            ("human_capital_metrics_description", "人的資本指標の説明"),
            ("human_capital_metrics", "人的資本指標"),
            ("research_and_development", "研究開発活動"),
            ("critical_contracts", "重要な契約等"),
            ("capital_expenditures", "設備投資等の概要"),
            ("major_facilities", "主要な設備の状況"),
            ("facility_plans", "設備の新設、除却等の計画"),
            ("shareholding", "株式の保有状況"),
            ("policy_shareholdings", "政策保有株式（銘柄別）"),
            ("major_shareholders", "大株主の状況"),
            ("dividend_policy", "配当政策"),
            ("officers", "役員の状況"),
            ("corporate_governance", "コーポレート・ガバナンスの概要"),
            ("officer_compensation", "役員の報酬等"),
            ("governance_metrics", "ガバナンス指標"),
            ("category", "保有区分"),
            ("holder_scope", "開示主体"),
            ("holder_name", "保有会社名"),
            ("row_number", "行番号"),
            ("issue_name", "銘柄"),
            ("current_fiscal_year", "当事業年度"),
            ("prior_fiscal_year", "前事業年度"),
            ("purpose_of_shareholding", "保有目的"),
            ("business_alliance_overview", "業務提携等の概要"),
            ("quantitative_effects", "定量的な保有効果"),
            ("reason_for_increase", "株式数が増加した理由"),
            ("combined_purpose_and_effects", "保有目的等（結合項目）"),
            (
                "issuer_holds_reporting_company_shares",
                "発行者による提出会社株式の保有の有無",
            ),
            ("shares", "株式数"),
            ("book_value", "貸借対照表計上額"),
            ("shares_not_disclosed", "株式数記載省略"),
            ("book_value_not_disclosed", "貸借対照表計上額記載省略"),
            ("segment_information", "セグメント情報等の注記"),
            ("primary_statements", "主要財務諸表"),
            ("employees_count", "従業員数"),
            ("average_annual_salary", "平均年間給与"),
            ("average_age_years", "平均年齢"),
            ("average_service_years", "平均勤続年数"),
            ("ratio_female_managers", "女性管理職比率"),
            ("ratio_male_childcare_leave", "男性育児休業取得率"),
            ("gender_pay_gap_all", "男女賃金差異（全労働者）"),
            ("gender_pay_gap_regular", "男女賃金差異（正規雇用労働者）"),
            (
                "gender_pay_gap_non_regular",
                "男女賃金差異（非正規雇用労働者）",
            ),
            ("ratio_female_directors", "女性役員比率"),
            ("total_officer_compensation", "役員報酬総額"),
            ("audit_fee", "監査報酬"),
            ("num_male_directors", "男性役員数"),
            ("num_female_directors", "女性役員数"),
            ("balance_sheet", "貸借対照表 (B/S)"),
            ("profit_and_loss", "損益計算書 (P/L)"),
            ("current_period", "当期"),
            ("prior_period", "前期"),
            ("assets", "資産"),
            ("current_assets", "流動資産"),
            ("noncurrent_assets", "固定資産"),
            ("current_liabilities", "流動負債"),
            ("noncurrent_liabilities", "固定負債"),
            ("net_assets", "純資産"),
            ("capital_stock", "資本金"),
            ("capital_surplus", "資本剰余金"),
            ("retained_earnings", "利益剰余金"),
            ("operating_revenue", "売上高"),
            ("operating_expenses", "営業費用"),
            ("operating_income", "営業利益"),
            ("ordinary_income", "経常利益"),
            ("income_before_income_taxes", "税引前純利益"),
            ("profit_loss", "純利益"),
            ("period", "期間コード"),
            ("label", "期間"),
            ("operating_revenue_summary", "売上高・営業収益"),
            ("net_income", "当期純利益"),
            ("total_assets", "総資産"),
            ("issued_shares_total", "発行済株式総数"),
            ("net_assets_per_share", "1株当たり純資産額"),
            ("earnings_per_share", "1株当たり当期純利益"),
            ("dividend_per_share", "1株当たり配当額"),
            ("equity_ratio", "自己資本比率"),
            ("roe", "自己資本利益率 (ROE)"),
            ("per", "株価収益率 (PER)"),
            ("payout_ratio", "配当性向"),
            ("operating_cash_flow", "営業活動によるキャッシュ・フロー"),
            ("investing_cash_flow", "投資活動によるキャッシュ・フロー"),
            ("financing_cash_flow", "財務活動によるキャッシュ・フロー"),
            ("cash_and_equivalents", "現金及び現金同等物"),
            ("total_shareholder_return", "総株主還元率"),
        ])
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn 翻訳辞書は正規jsonの全114キーを重複なく持つ() {
        assert_eq!(ja_key_translations().len(), 114);
    }

    #[test]
    fn ネストしたオブジェクトと配列のキーを日本語化する() {
        let mut value = json!({
            "report": {
                "corporate_information": {
                    "policy_shareholdings": [{
                        "current_fiscal_year": {"shares": 100},
                        "category": "deemed_holding"
                    }]
                },
                "financial_information": {
                    "primary_statements": {
                        "balance_sheet": {"current_period": {"assets": 1000}}
                    }
                }
            }
        });

        translate_keys_to_ja(&mut value);

        assert_eq!(
            value["有価証券報告書"]["第4 提出会社の状況"]["政策保有株式（銘柄別）"][0]["当事業年度"]
                ["株式数"],
            100
        );
        assert_eq!(
            value["有価証券報告書"]["第4 提出会社の状況"]["政策保有株式（銘柄別）"][0]["保有区分"],
            "deemed_holding"
        );
        assert_eq!(
            value["有価証券報告書"]["第5 経理の状況"]["主要財務諸表"]["貸借対照表 (B/S)"]["当期"]["資産"],
            1000
        );
    }

    #[test]
    fn jqが生成した独自キーは変更しない() {
        let mut value = json!({"selected": {"operating_revenue_summary": 42}});

        translate_keys_to_ja(&mut value);

        assert_eq!(value, json!({"selected": {"売上高・営業収益": 42}}));
    }
}
