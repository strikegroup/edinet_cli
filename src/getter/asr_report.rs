// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, Default, serde::Serialize)]
/// 有価証券報告書を、EDINET の章立て単位で再構成した抽出結果。
///
/// `jpcrp_cor:*TextBlock` の文章と、`jpcrp_cor:` / `jppfs_cor:` の数値 fact を
/// 同じ章の下へ寄せて、後続処理が「有報のどの章の情報か」を意識しやすい形にする。
pub struct AsrReport {
    pub company_overview: CompanyOverview,
    pub business_overview: BusinessOverview,
    pub facilities: Facilities,
    pub corporate_information: CorporateInformation,
    pub financial_information: FinancialInformation,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
/// 第1 企業の概況。
pub struct CompanyOverview {
    pub company_history: Option<String>,
    pub employees: Option<String>,
    /// EDINET タクソノミの `*SummaryOfBusinessResults` 群を、
    /// 「主要な経営指標等の推移」の 5 期時系列として並べたもの。
    pub business_results_summary: Vec<BusinessResultsPeriod>,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
/// 第2 事業の状況。
pub struct BusinessOverview {
    pub business_description: Option<String>,
    pub performance: Option<String>,
    pub issues_to_address: Option<String>,
    pub risks: Option<String>,
    pub sustainability: Option<String>,
    pub governance: Option<String>,
    pub strategy: Option<String>,
    pub risk_management: Option<String>,
    pub metrics_and_targets: Option<String>,
    pub human_resources_policy: Option<String>,
    pub human_capital_metrics_description: Option<String>,
    /// 人材育成方針・社内環境整備方針に紐づく人的資本開示の定量項目。
    ///
    /// 2023 年改正で追加された、女性管理職比率・男女賃金差異・男性育休取得率などを入れる。
    pub human_capital_metrics: HumanCapitalMetrics,
    pub research_and_development: Option<String>,
    pub critical_contracts: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
/// 第3 設備の状況。
pub struct Facilities {
    pub capital_expenditures: Option<String>,
    pub major_facilities: Option<String>,
    pub facility_plans: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
/// 第4 提出会社の状況。
pub struct CorporateInformation {
    pub shareholding: Option<String>,
    pub major_shareholders: Option<String>,
    pub dividend_policy: Option<String>,
    pub officers: Option<String>,
    pub corporate_governance: Option<String>,
    pub officer_compensation: Option<String>,
    /// コーポレートガバナンスや役員情報に紐づく定量項目。
    ///
    /// 女性役員比率や役員報酬総額など、第4 提出会社の状況で読むことが多い数値をまとめる。
    pub governance_metrics: GovernanceMetrics,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
/// 第5 経理の状況。
pub struct FinancialInformation {
    pub segment_information: Option<String>,
    /// `jppfs_cor:` の主要科目を、財務諸表本体の意味づけでまとめたもの。
    ///
    /// 第1 の経営指標サマリーではなく、第5 経理の状況にある B/S・P/L の fact を直接使う。
    pub primary_statements: PrimaryFinancialStatements,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
/// 第2 事業の状況のうち、人的資本・多様性開示に属する定量項目。
///
/// 会社全体の人員属性や処遇差を示す指標で、財務諸表数値ではなくサステナビリティ開示側の KPI。
pub struct HumanCapitalMetrics {
    pub employees_count: Option<i64>,
    pub average_annual_salary: Option<i64>,
    pub average_age_years: Option<f64>,
    pub average_service_years: Option<f64>,
    pub ratio_female_managers: Option<f64>,
    pub ratio_male_childcare_leave: Option<f64>,
    pub gender_pay_gap_all: Option<f64>,
    pub gender_pay_gap_regular: Option<f64>,
    pub gender_pay_gap_non_regular: Option<f64>,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
/// 第4 提出会社の状況のうち、ガバナンス体制や役員報酬に属する定量項目。
pub struct GovernanceMetrics {
    pub ratio_female_directors: Option<f64>,
    pub total_officer_compensation: Option<i64>,
    pub audit_fee: Option<i64>,
    pub num_male_directors: Option<i64>,
    pub num_female_directors: Option<i64>,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
/// 第5 経理の状況に含まれる、財務諸表本体の主要数値。
///
/// EDINET の `jppfs_cor:` 要素を、貸借対照表と損益計算書という会計上の見出しに再配置している。
pub struct PrimaryFinancialStatements {
    pub balance_sheet: BalanceSheetPeriods,
    pub profit_and_loss: ProfitAndLossPeriods,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
/// 貸借対照表主要科目の当期・前期。
pub struct BalanceSheetPeriods {
    pub current: BalanceSheetItems,
    pub prior: BalanceSheetItems,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
/// 貸借対照表主要科目。
pub struct BalanceSheetItems {
    pub assets: Option<i64>,
    pub current_assets: Option<i64>,
    pub noncurrent_assets: Option<i64>,
    pub current_liabilities: Option<i64>,
    pub noncurrent_liabilities: Option<i64>,
    pub net_assets: Option<i64>,
    pub capital_stock: Option<i64>,
    pub capital_surplus: Option<i64>,
    pub retained_earnings: Option<i64>,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
/// 損益計算書主要科目の当期・前期。
pub struct ProfitAndLossPeriods {
    pub current: ProfitAndLossItems,
    pub prior: ProfitAndLossItems,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
/// 損益計算書主要科目。
pub struct ProfitAndLossItems {
    pub operating_revenue: Option<i64>,
    pub operating_expenses: Option<i64>,
    pub operating_income: Option<i64>,
    pub ordinary_income: Option<i64>,
    pub income_before_income_taxes: Option<i64>,
    pub profit_loss: Option<i64>,
}

#[derive(Debug, Clone, serde::Serialize)]
/// 「主要な経営指標等の推移」の 1 期分。
///
/// ここに入るのは `jpcrp_cor:*SummaryOfBusinessResults` 系の fact で、
/// 財務諸表本体ではなく「サマリーとして再掲された経営指標」である。
///
/// EDINET では同じ期でも連結・個別・member なしの複数 context に分かれて出るため、
/// extractor 側で優先順を持って 1 つの期間値に正規化している。
pub struct BusinessResultsPeriod {
    pub period: String,
    pub label: String,
    pub operating_revenue: Option<i64>,
    pub ordinary_income: Option<i64>,
    pub net_income: Option<i64>,
    pub net_assets: Option<i64>,
    pub total_assets: Option<i64>,
    pub capital_stock: Option<i64>,
    pub issued_shares_total: Option<i64>,
    pub net_assets_per_share: Option<f64>,
    pub earnings_per_share: Option<f64>,
    pub dividend_per_share: Option<f64>,
    pub equity_ratio: Option<f64>,
    pub roe: Option<f64>,
    pub per: Option<f64>,
    pub payout_ratio: Option<f64>,
    pub operating_cash_flow: Option<i64>,
    pub investing_cash_flow: Option<i64>,
    pub financing_cash_flow: Option<i64>,
    pub cash_and_equivalents: Option<i64>,
    pub total_shareholder_return: Option<f64>,
    pub employees: Option<i64>,
}
