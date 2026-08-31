// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use crate::getter::asr_report::{
    AsrReport, BalanceSheetItems, BalanceSheetPeriods, BusinessOverview, BusinessResultsPeriod,
    CompanyOverview, CorporateInformation, Facilities, FinancialInformation, GovernanceMetrics,
    HumanCapitalMetrics, PrimaryFinancialStatements, ProfitAndLossItems, ProfitAndLossPeriods,
};
use crate::store::asr_document_metadata::AsrDocumentMetadata;

/// `get` コマンドの日本語ラベル付き出力。
#[derive(Debug, Clone, serde::Serialize)]
pub struct GetCommandOutputJa {
    #[serde(rename = "書類情報")]
    pub metadata: Option<AsrDocumentMetadataJa>,
    #[serde(rename = "有価証券報告書")]
    pub report: AsrReportJa,
}

impl GetCommandOutputJa {
    pub fn new(metadata: Option<&AsrDocumentMetadata>, report: &AsrReport) -> Self {
        Self {
            metadata: metadata.map(AsrDocumentMetadataJa::from),
            report: AsrReportJa::from(report),
        }
    }
}

/// 保存済み metadata の日本語ラベル付き出力。
#[derive(Debug, Clone, serde::Serialize)]
pub struct AsrDocumentMetadataJa {
    #[serde(rename = "提出日")]
    pub file_date: String,
    #[serde(rename = "書類ID")]
    pub doc_id: String,
    #[serde(rename = "EDINETコード")]
    pub edinet_code: Option<String>,
    #[serde(rename = "証券コード")]
    pub sec_code: Option<String>,
    #[serde(rename = "提出者名")]
    pub filer_name: Option<String>,
    #[serde(rename = "事業年度開始日")]
    pub period_start: Option<String>,
    #[serde(rename = "事業年度終了日")]
    pub period_end: Option<String>,
    #[serde(rename = "提出日時")]
    pub submit_date_time: Option<String>,
    #[serde(rename = "書類概要")]
    pub doc_description: Option<String>,
}

impl From<&AsrDocumentMetadata> for AsrDocumentMetadataJa {
    fn from(value: &AsrDocumentMetadata) -> Self {
        Self {
            file_date: value.file_date.clone(),
            doc_id: value.doc_id.clone(),
            edinet_code: value.edinet_code.clone(),
            sec_code: value.sec_code.clone(),
            filer_name: value.filer_name.clone(),
            period_start: value.period_start.clone(),
            period_end: value.period_end.clone(),
            submit_date_time: value.submit_date_time.clone(),
            doc_description: value.doc_description.clone(),
        }
    }
}

/// 有価証券報告書の章立てを日本語キーで表した出力。
#[derive(Debug, Clone, serde::Serialize)]
pub struct AsrReportJa {
    #[serde(rename = "第1 企業の概況")]
    pub company_overview: CompanyOverviewJa,
    #[serde(rename = "第2 事業の状況")]
    pub business_overview: BusinessOverviewJa,
    #[serde(rename = "第3 設備の状況")]
    pub facilities: FacilitiesJa,
    #[serde(rename = "第4 提出会社の状況")]
    pub corporate_information: CorporateInformationJa,
    #[serde(rename = "第5 経理の状況")]
    pub financial_information: FinancialInformationJa,
}

impl From<&AsrReport> for AsrReportJa {
    fn from(value: &AsrReport) -> Self {
        Self {
            company_overview: CompanyOverviewJa::from(&value.company_overview),
            business_overview: BusinessOverviewJa::from(&value.business_overview),
            facilities: FacilitiesJa::from(&value.facilities),
            corporate_information: CorporateInformationJa::from(&value.corporate_information),
            financial_information: FinancialInformationJa::from(&value.financial_information),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CompanyOverviewJa {
    #[serde(rename = "沿革")]
    pub company_history: Option<String>,
    #[serde(rename = "従業員の状況")]
    pub employees: Option<String>,
    #[serde(rename = "主要な経営指標等の推移")]
    pub business_results_summary: Vec<BusinessResultsPeriodJa>,
}

impl From<&CompanyOverview> for CompanyOverviewJa {
    fn from(value: &CompanyOverview) -> Self {
        Self {
            company_history: value.company_history.clone(),
            employees: value.employees.clone(),
            business_results_summary: value
                .business_results_summary
                .iter()
                .map(BusinessResultsPeriodJa::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BusinessOverviewJa {
    #[serde(rename = "事業の内容")]
    pub business_description: Option<String>,
    #[serde(rename = "経営者による財政状態、経営成績及びキャッシュ・フローの状況の分析")]
    pub performance: Option<String>,
    #[serde(rename = "経営方針、経営環境及び対処すべき課題等")]
    pub issues_to_address: Option<String>,
    #[serde(rename = "事業等のリスク")]
    pub risks: Option<String>,
    #[serde(rename = "サステナビリティ関連情報")]
    pub sustainability: Option<String>,
    #[serde(rename = "ガバナンス")]
    pub governance: Option<String>,
    #[serde(rename = "戦略")]
    pub strategy: Option<String>,
    #[serde(rename = "リスク管理")]
    pub risk_management: Option<String>,
    #[serde(rename = "指標及び目標")]
    pub metrics_and_targets: Option<String>,
    #[serde(rename = "人材育成方針・社内環境整備方針")]
    pub human_resources_policy: Option<String>,
    #[serde(rename = "人的資本指標の説明")]
    pub human_capital_metrics_description: Option<String>,
    #[serde(rename = "人的資本指標")]
    pub human_capital_metrics: HumanCapitalMetricsJa,
    #[serde(rename = "研究開発活動")]
    pub research_and_development: Option<String>,
    #[serde(rename = "重要な契約等")]
    pub critical_contracts: Option<String>,
}

impl From<&BusinessOverview> for BusinessOverviewJa {
    fn from(value: &BusinessOverview) -> Self {
        Self {
            business_description: value.business_description.clone(),
            performance: value.performance.clone(),
            issues_to_address: value.issues_to_address.clone(),
            risks: value.risks.clone(),
            sustainability: value.sustainability.clone(),
            governance: value.governance.clone(),
            strategy: value.strategy.clone(),
            risk_management: value.risk_management.clone(),
            metrics_and_targets: value.metrics_and_targets.clone(),
            human_resources_policy: value.human_resources_policy.clone(),
            human_capital_metrics_description: value.human_capital_metrics_description.clone(),
            human_capital_metrics: HumanCapitalMetricsJa::from(&value.human_capital_metrics),
            research_and_development: value.research_and_development.clone(),
            critical_contracts: value.critical_contracts.clone(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FacilitiesJa {
    #[serde(rename = "設備投資等の概要")]
    pub capital_expenditures: Option<String>,
    #[serde(rename = "主要な設備の状況")]
    pub major_facilities: Option<String>,
    #[serde(rename = "設備の新設、除却等の計画")]
    pub facility_plans: Option<String>,
}

impl From<&Facilities> for FacilitiesJa {
    fn from(value: &Facilities) -> Self {
        Self {
            capital_expenditures: value.capital_expenditures.clone(),
            major_facilities: value.major_facilities.clone(),
            facility_plans: value.facility_plans.clone(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CorporateInformationJa {
    #[serde(rename = "株式の保有状況")]
    pub shareholding: Option<String>,
    #[serde(rename = "大株主の状況")]
    pub major_shareholders: Option<String>,
    #[serde(rename = "配当政策")]
    pub dividend_policy: Option<String>,
    #[serde(rename = "役員の状況")]
    pub officers: Option<String>,
    #[serde(rename = "コーポレート・ガバナンスの概要")]
    pub corporate_governance: Option<String>,
    #[serde(rename = "役員の報酬等")]
    pub officer_compensation: Option<String>,
    #[serde(rename = "ガバナンス指標")]
    pub governance_metrics: GovernanceMetricsJa,
}

impl From<&CorporateInformation> for CorporateInformationJa {
    fn from(value: &CorporateInformation) -> Self {
        Self {
            shareholding: value.shareholding.clone(),
            major_shareholders: value.major_shareholders.clone(),
            dividend_policy: value.dividend_policy.clone(),
            officers: value.officers.clone(),
            corporate_governance: value.corporate_governance.clone(),
            officer_compensation: value.officer_compensation.clone(),
            governance_metrics: GovernanceMetricsJa::from(&value.governance_metrics),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FinancialInformationJa {
    #[serde(rename = "セグメント情報等の注記")]
    pub segment_information: Option<String>,
    #[serde(rename = "主要財務諸表")]
    pub primary_statements: PrimaryFinancialStatementsJa,
}

impl From<&FinancialInformation> for FinancialInformationJa {
    fn from(value: &FinancialInformation) -> Self {
        Self {
            segment_information: value.segment_information.clone(),
            primary_statements: PrimaryFinancialStatementsJa::from(&value.primary_statements),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct HumanCapitalMetricsJa {
    #[serde(rename = "従業員数")]
    pub employees_count: Option<i64>,
    #[serde(rename = "平均年間給与")]
    pub average_annual_salary: Option<i64>,
    #[serde(rename = "平均年齢")]
    pub average_age_years: Option<f64>,
    #[serde(rename = "平均勤続年数")]
    pub average_service_years: Option<f64>,
    #[serde(rename = "女性管理職比率")]
    pub ratio_female_managers: Option<f64>,
    #[serde(rename = "男性育児休業取得率")]
    pub ratio_male_childcare_leave: Option<f64>,
    #[serde(rename = "男女賃金差異（全労働者）")]
    pub gender_pay_gap_all: Option<f64>,
    #[serde(rename = "男女賃金差異（正規雇用労働者）")]
    pub gender_pay_gap_regular: Option<f64>,
    #[serde(rename = "男女賃金差異（非正規雇用労働者）")]
    pub gender_pay_gap_non_regular: Option<f64>,
}

impl From<&HumanCapitalMetrics> for HumanCapitalMetricsJa {
    fn from(value: &HumanCapitalMetrics) -> Self {
        Self {
            employees_count: value.employees_count,
            average_annual_salary: value.average_annual_salary,
            average_age_years: value.average_age_years,
            average_service_years: value.average_service_years,
            ratio_female_managers: value.ratio_female_managers,
            ratio_male_childcare_leave: value.ratio_male_childcare_leave,
            gender_pay_gap_all: value.gender_pay_gap_all,
            gender_pay_gap_regular: value.gender_pay_gap_regular,
            gender_pay_gap_non_regular: value.gender_pay_gap_non_regular,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GovernanceMetricsJa {
    #[serde(rename = "女性役員比率")]
    pub ratio_female_directors: Option<f64>,
    #[serde(rename = "役員報酬総額")]
    pub total_officer_compensation: Option<i64>,
    #[serde(rename = "監査報酬")]
    pub audit_fee: Option<i64>,
    #[serde(rename = "男性役員数")]
    pub num_male_directors: Option<i64>,
    #[serde(rename = "女性役員数")]
    pub num_female_directors: Option<i64>,
}

impl From<&GovernanceMetrics> for GovernanceMetricsJa {
    fn from(value: &GovernanceMetrics) -> Self {
        Self {
            ratio_female_directors: value.ratio_female_directors,
            total_officer_compensation: value.total_officer_compensation,
            audit_fee: value.audit_fee,
            num_male_directors: value.num_male_directors,
            num_female_directors: value.num_female_directors,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PrimaryFinancialStatementsJa {
    #[serde(rename = "貸借対照表 (B/S)")]
    pub balance_sheet: BalanceSheetPeriodsJa,
    #[serde(rename = "損益計算書 (P/L)")]
    pub profit_and_loss: ProfitAndLossPeriodsJa,
}

impl From<&PrimaryFinancialStatements> for PrimaryFinancialStatementsJa {
    fn from(value: &PrimaryFinancialStatements) -> Self {
        Self {
            balance_sheet: BalanceSheetPeriodsJa::from(&value.balance_sheet),
            profit_and_loss: ProfitAndLossPeriodsJa::from(&value.profit_and_loss),
        }
    }
}
#[derive(Debug, Clone, serde::Serialize)]
pub struct BalanceSheetPeriodsJa {
    #[serde(rename = "当期")]
    pub current: BalanceSheetItemsJa,
    #[serde(rename = "前期")]
    pub prior: BalanceSheetItemsJa,
}

impl From<&BalanceSheetPeriods> for BalanceSheetPeriodsJa {
    fn from(value: &BalanceSheetPeriods) -> Self {
        Self {
            current: BalanceSheetItemsJa::from(&value.current),
            prior: BalanceSheetItemsJa::from(&value.prior),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BalanceSheetItemsJa {
    #[serde(rename = "資産")]
    pub assets: Option<i64>,
    #[serde(rename = "流動資産")]
    pub current_assets: Option<i64>,
    #[serde(rename = "固定資産")]
    pub noncurrent_assets: Option<i64>,
    #[serde(rename = "流動負債")]
    pub current_liabilities: Option<i64>,
    #[serde(rename = "固定負債")]
    pub noncurrent_liabilities: Option<i64>,
    #[serde(rename = "純資産")]
    pub net_assets: Option<i64>,
    #[serde(rename = "資本金")]
    pub capital_stock: Option<i64>,
    #[serde(rename = "資本剰余金")]
    pub capital_surplus: Option<i64>,
    #[serde(rename = "利益剰余金")]
    pub retained_earnings: Option<i64>,
}

impl From<&BalanceSheetItems> for BalanceSheetItemsJa {
    fn from(value: &BalanceSheetItems) -> Self {
        Self {
            assets: value.assets,
            current_assets: value.current_assets,
            noncurrent_assets: value.noncurrent_assets,
            current_liabilities: value.current_liabilities,
            noncurrent_liabilities: value.noncurrent_liabilities,
            net_assets: value.net_assets,
            capital_stock: value.capital_stock,
            capital_surplus: value.capital_surplus,
            retained_earnings: value.retained_earnings,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProfitAndLossPeriodsJa {
    #[serde(rename = "当期")]
    pub current: ProfitAndLossItemsJa,
    #[serde(rename = "前期")]
    pub prior: ProfitAndLossItemsJa,
}

impl From<&ProfitAndLossPeriods> for ProfitAndLossPeriodsJa {
    fn from(value: &ProfitAndLossPeriods) -> Self {
        Self {
            current: ProfitAndLossItemsJa::from(&value.current),
            prior: ProfitAndLossItemsJa::from(&value.prior),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProfitAndLossItemsJa {
    #[serde(rename = "売上高")]
    pub operating_revenue: Option<i64>,
    #[serde(rename = "営業費用")]
    pub operating_expenses: Option<i64>,
    #[serde(rename = "営業利益")]
    pub operating_income: Option<i64>,
    #[serde(rename = "経常利益")]
    pub ordinary_income: Option<i64>,
    #[serde(rename = "税引前純利益")]
    pub income_before_income_taxes: Option<i64>,
    #[serde(rename = "純利益")]
    pub profit_loss: Option<i64>,
}

impl From<&ProfitAndLossItems> for ProfitAndLossItemsJa {
    fn from(value: &ProfitAndLossItems) -> Self {
        Self {
            operating_revenue: value.operating_revenue,
            operating_expenses: value.operating_expenses,
            operating_income: value.operating_income,
            ordinary_income: value.ordinary_income,
            income_before_income_taxes: value.income_before_income_taxes,
            profit_loss: value.profit_loss,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BusinessResultsPeriodJa {
    #[serde(rename = "期間コード")]
    pub period: String,
    #[serde(rename = "期間")]
    pub label: String,
    #[serde(rename = "売上高・営業収益")]
    pub operating_revenue: Option<i64>,
    #[serde(rename = "経常利益")]
    pub ordinary_income: Option<i64>,
    #[serde(rename = "当期純利益")]
    pub net_income: Option<i64>,
    #[serde(rename = "純資産")]
    pub net_assets: Option<i64>,
    #[serde(rename = "総資産")]
    pub total_assets: Option<i64>,
    #[serde(rename = "資本金")]
    pub capital_stock: Option<i64>,
    #[serde(rename = "発行済株式総数")]
    pub issued_shares_total: Option<i64>,
    #[serde(rename = "1株当たり純資産額")]
    pub net_assets_per_share: Option<f64>,
    #[serde(rename = "1株当たり当期純利益")]
    pub earnings_per_share: Option<f64>,
    #[serde(rename = "1株当たり配当額")]
    pub dividend_per_share: Option<f64>,
    #[serde(rename = "自己資本比率")]
    pub equity_ratio: Option<f64>,
    #[serde(rename = "自己資本利益率 (ROE)")]
    pub roe: Option<f64>,
    #[serde(rename = "株価収益率 (PER)")]
    pub per: Option<f64>,
    #[serde(rename = "配当性向")]
    pub payout_ratio: Option<f64>,
    #[serde(rename = "営業活動によるキャッシュ・フロー")]
    pub operating_cash_flow: Option<i64>,
    #[serde(rename = "投資活動によるキャッシュ・フロー")]
    pub investing_cash_flow: Option<i64>,
    #[serde(rename = "財務活動によるキャッシュ・フロー")]
    pub financing_cash_flow: Option<i64>,
    #[serde(rename = "現金及び現金同等物")]
    pub cash_and_equivalents: Option<i64>,
    #[serde(rename = "総株主還元率")]
    pub total_shareholder_return: Option<f64>,
    #[serde(rename = "従業員数")]
    pub employees: Option<i64>,
}

impl From<&BusinessResultsPeriod> for BusinessResultsPeriodJa {
    fn from(value: &BusinessResultsPeriod) -> Self {
        Self {
            period: value.period.clone(),
            label: value.label.clone(),
            operating_revenue: value.operating_revenue,
            ordinary_income: value.ordinary_income,
            net_income: value.net_income,
            net_assets: value.net_assets,
            total_assets: value.total_assets,
            capital_stock: value.capital_stock,
            issued_shares_total: value.issued_shares_total,
            net_assets_per_share: value.net_assets_per_share,
            earnings_per_share: value.earnings_per_share,
            dividend_per_share: value.dividend_per_share,
            equity_ratio: value.equity_ratio,
            roe: value.roe,
            per: value.per,
            payout_ratio: value.payout_ratio,
            operating_cash_flow: value.operating_cash_flow,
            investing_cash_flow: value.investing_cash_flow,
            financing_cash_flow: value.financing_cash_flow,
            cash_and_equivalents: value.cash_and_equivalents,
            total_shareholder_return: value.total_shareholder_return,
            employees: value.employees,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_japanese_labels_for_get_output() {
        let metadata = AsrDocumentMetadata {
            id: 1,
            file_date: "2026-04-01".to_owned(),
            seq_number: 1,
            doc_id: "S100TEST".to_owned(),
            edinet_code: Some("E00000".to_owned()),
            sec_code: Some("7203".to_owned()),
            jcn: Some("1234567890123".to_owned()),
            filer_name: Some("テスト株式会社".to_owned()),
            ordinance_code: Some("010".to_owned()),
            form_code: Some("030000".to_owned()),
            doc_type_code: Some("120".to_owned()),
            period_start: Some("2025-04-01".to_owned()),
            period_end: Some("2026-03-31".to_owned()),
            submit_date_time: Some("2026-04-01 10:00".to_owned()),
            doc_description: Some("有価証券報告書".to_owned()),
            withdrawal_status: "0".to_owned(),
            doc_info_edit_status: "0".to_owned(),
            disclosure_status: "0".to_owned(),
            xbrl_flag: "1".to_owned(),
            pdf_flag: "1".to_owned(),
            attach_doc_flag: "0".to_owned(),
            english_doc_flag: "0".to_owned(),
            csv_flag: "1".to_owned(),
            legal_status: "1".to_owned(),
        };
        let report = AsrReport {
            company_overview: CompanyOverview {
                company_history: Some("沿革本文".to_owned()),
                employees: None,
                business_results_summary: vec![BusinessResultsPeriod {
                    period: "CurrentYear".to_owned(),
                    label: "当期".to_owned(),
                    operating_revenue: Some(123),
                    ordinary_income: None,
                    net_income: None,
                    net_assets: None,
                    total_assets: None,
                    capital_stock: None,
                    issued_shares_total: None,
                    net_assets_per_share: None,
                    earnings_per_share: None,
                    dividend_per_share: None,
                    equity_ratio: None,
                    roe: None,
                    per: None,
                    payout_ratio: None,
                    operating_cash_flow: None,
                    investing_cash_flow: None,
                    financing_cash_flow: None,
                    cash_and_equivalents: None,
                    total_shareholder_return: None,
                    employees: None,
                }],
            },
            business_overview: BusinessOverview {
                business_description: None,
                performance: None,
                issues_to_address: None,
                risks: Some("リスク本文".to_owned()),
                sustainability: None,
                governance: None,
                strategy: None,
                risk_management: None,
                metrics_and_targets: None,
                human_resources_policy: None,
                human_capital_metrics_description: None,
                human_capital_metrics: HumanCapitalMetrics::default(),
                research_and_development: None,
                critical_contracts: None,
            },
            facilities: Facilities::default(),
            corporate_information: CorporateInformation {
                shareholding: None,
                major_shareholders: None,
                dividend_policy: None,
                officers: None,
                corporate_governance: None,
                officer_compensation: None,
                governance_metrics: GovernanceMetrics {
                    ratio_female_directors: Some(0.25),
                    total_officer_compensation: None,
                    audit_fee: None,
                    num_male_directors: None,
                    num_female_directors: None,
                },
            },
            financial_information: FinancialInformation {
                segment_information: None,
                primary_statements: PrimaryFinancialStatements {
                    balance_sheet: BalanceSheetPeriods {
                        current: BalanceSheetItems {
                            assets: Some(1000),
                            current_assets: None,
                            noncurrent_assets: None,
                            current_liabilities: None,
                            noncurrent_liabilities: None,
                            net_assets: None,
                            capital_stock: None,
                            capital_surplus: None,
                            retained_earnings: None,
                        },
                        prior: BalanceSheetItems::default(),
                    },
                    profit_and_loss: ProfitAndLossPeriods {
                        current: ProfitAndLossItems {
                            operating_revenue: Some(700),
                            operating_expenses: None,
                            operating_income: None,
                            ordinary_income: None,
                            income_before_income_taxes: None,
                            profit_loss: None,
                        },
                        prior: ProfitAndLossItems::default(),
                    },
                },
            },
        };

        let value = serde_json::to_value(GetCommandOutputJa::new(Some(&metadata), &report))
            .expect("Japanese output should serialize");

        assert_eq!(value["書類情報"]["提出日"], "2026-04-01");
        assert_eq!(
            value["有価証券報告書"]["第1 企業の概況"]["沿革"],
            "沿革本文"
        );
        assert_eq!(
            value["有価証券報告書"]["第1 企業の概況"]["主要な経営指標等の推移"][0]["売上高・営業収益"],
            123
        );
        assert_eq!(
            value["有価証券報告書"]["第4 提出会社の状況"]["ガバナンス指標"]["女性役員比率"],
            0.25
        );
        assert_eq!(
            value["有価証券報告書"]["第5 経理の状況"]["主要財務諸表"]["貸借対照表 (B/S)"]["当期"]["資産"],
            1000
        );
    }
}
