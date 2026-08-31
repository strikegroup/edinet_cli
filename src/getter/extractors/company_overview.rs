// SPDX-FileCopyrightText: 2026 Strike Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use super::super::asr_report::{BusinessResultsPeriod, CompanyOverview};
use super::super::xbrl_fact::XbrlFactIndex;

/// 「沿革」本文。
const COMPANY_HISTORY: &str = "jpcrp_cor:CompanyHistoryTextBlock";
/// 「従業員の状況」本文。
const EMPLOYEES: &str = "jpcrp_cor:InformationAboutEmployeesTextBlock";

/// 売上高・営業収益に相当する経営指標。
///
/// 一般事業会社は NetSales、金融・リース・交通などは OperatingRevenue1、
/// 銀行・保険などは OrdinaryRevenues や OrdinaryIncome、IFRS では RevenueIFRS で出ることがある。
const OPERATING_REVENUE_IDS: &[&str] = &[
    "jpcrp_cor:NetSalesSummaryOfBusinessResults",
    "jpcrp_cor:OperatingRevenue1SummaryOfBusinessResults",
    "jpcrp_cor:OperatingRevenue2SummaryOfBusinessResults",
    "jpcrp_cor:OrdinaryIncomeSummaryOfBusinessResults",
    "jpcrp_cor:OrdinaryRevenuesSummaryOfBusinessResults",
    "jpcrp_cor:RevenueIFRSSummaryOfBusinessResults",
    "jpcrp_cor:RevenueKeyFinancialData",
];
/// 経常利益または経常損失。
const ORDINARY_INCOME: &str = "jpcrp_cor:OrdinaryIncomeLossSummaryOfBusinessResults";
/// 連結会社の親会社株主に帰属する当期純利益または純損失。
const PROFIT_LOSS_ATTRIBUTABLE_TO_OWNERS_OF_PARENT: &str =
    "jpcrp_cor:ProfitLossAttributableToOwnersOfParentSummaryOfBusinessResults";
const PROFIT_LOSS_ATTRIBUTABLE_TO_OWNERS_OF_PARENT_IFRS: &str =
    "jpcrp_cor:ProfitLossAttributableToOwnersOfParentIFRSSummaryOfBusinessResults";
/// 単体会社の当期純利益または純損失。
const NET_INCOME_LOSS: &str = "jpcrp_cor:NetIncomeLossSummaryOfBusinessResults";
/// 純資産額。
const NET_ASSETS: &str = "jpcrp_cor:NetAssetsSummaryOfBusinessResults";
const EQUITY_ATTRIBUTABLE_TO_OWNERS_OF_PARENT_IFRS: &str =
    "jpcrp_cor:EquityAttributableToOwnersOfParentIFRSSummaryOfBusinessResults";
/// 総資産額。
const TOTAL_ASSETS: &str = "jpcrp_cor:TotalAssetsSummaryOfBusinessResults";
const TOTAL_ASSETS_IFRS: &str = "jpcrp_cor:TotalAssetsIFRSSummaryOfBusinessResults";
/// 資本金。
const CAPITAL_STOCK: &str = "jpcrp_cor:CapitalStockSummaryOfBusinessResults";
/// 発行済株式総数。
const ISSUED_SHARES_TOTAL: &str = "jpcrp_cor:TotalNumberOfIssuedSharesSummaryOfBusinessResults";
/// 1 株当たり純資産額。
const NET_ASSETS_PER_SHARE: &str = "jpcrp_cor:NetAssetsPerShareSummaryOfBusinessResults";
/// 1 株当たり当期純利益。EDINET 名では BasicEarningsLossPerShare。
const EARNINGS_PER_SHARE: &str = "jpcrp_cor:BasicEarningsLossPerShareSummaryOfBusinessResults";
const EARNINGS_PER_SHARE_IFRS: &str =
    "jpcrp_cor:BasicEarningsLossPerShareIFRSSummaryOfBusinessResults";
/// 1 株当たり配当額。
const DIVIDEND_PER_SHARE: &str = "jpcrp_cor:DividendPaidPerShareSummaryOfBusinessResults";
/// 自己資本比率。
const EQUITY_RATIO: &str = "jpcrp_cor:EquityToAssetRatioSummaryOfBusinessResults";
const EQUITY_RATIO_IFRS: &str =
    "jpcrp_cor:RatioOfOwnersEquityToGrossAssetsIFRSSummaryOfBusinessResults";
/// 自己資本利益率 (ROE)。
const ROE: &str = "jpcrp_cor:RateOfReturnOnEquitySummaryOfBusinessResults";
const ROE_IFRS: &str = "jpcrp_cor:RateOfReturnOnEquityIFRSSummaryOfBusinessResults";
/// 株価収益率 (PER)。
const PER: &str = "jpcrp_cor:PriceEarningsRatioSummaryOfBusinessResults";
const PER_IFRS: &str = "jpcrp_cor:PriceEarningsRatioIFRSSummaryOfBusinessResults";
/// 配当性向。
const PAYOUT_RATIO: &str = "jpcrp_cor:PayoutRatioSummaryOfBusinessResults";
/// 営業活動によるキャッシュ・フロー。
const OPERATING_CASH_FLOW: &str =
    "jpcrp_cor:NetCashProvidedByUsedInOperatingActivitiesSummaryOfBusinessResults";
const OPERATING_CASH_FLOW_IFRS: &str =
    "jpcrp_cor:CashFlowsFromUsedInOperatingActivitiesIFRSSummaryOfBusinessResults";
/// 投資活動によるキャッシュ・フロー。
const INVESTING_CASH_FLOW: &str =
    "jpcrp_cor:NetCashProvidedByUsedInInvestingActivitiesSummaryOfBusinessResults";
const INVESTING_CASH_FLOW_IFRS: &str =
    "jpcrp_cor:CashFlowsFromUsedInInvestingActivitiesIFRSSummaryOfBusinessResults";
/// 財務活動によるキャッシュ・フロー。
const FINANCING_CASH_FLOW: &str =
    "jpcrp_cor:NetCashProvidedByUsedInFinancingActivitiesSummaryOfBusinessResults";
const FINANCING_CASH_FLOW_IFRS: &str =
    "jpcrp_cor:CashFlowsFromUsedInFinancingActivitiesIFRSSummaryOfBusinessResults";
/// 現金及び現金同等物の期末残高。
const CASH_AND_EQUIVALENTS: &str = "jpcrp_cor:CashAndCashEquivalentsSummaryOfBusinessResults";
const CASH_AND_EQUIVALENTS_IFRS: &str =
    "jpcrp_cor:CashAndCashEquivalentsIFRSSummaryOfBusinessResults";
/// 総株主還元率。
const TOTAL_SHAREHOLDER_RETURN: &str = "jpcrp_cor:TotalShareholderReturn";
/// 従業員数。
const NUMBER_OF_EMPLOYEES: &str = "jpcrp_cor:NumberOfEmployees";

struct PeriodContext {
    period: &'static str,
    label: &'static str,
    duration_contexts: &'static [&'static str],
    instant_contexts: &'static [&'static str],
}

/// `*SummaryOfBusinessResults` は「第1 企業の概況 > 主要な経営指標等の推移」に対応する。
///
/// 同じ経営指標でも提出会社によって 連結 member / 個別 member / member なし に出し分けられるため、
/// ここでは suffix なしの連結値を優先し、なければ member 付きへフォールバックする。
const PERIOD_CONTEXTS: &[PeriodContext] = &[
    PeriodContext {
        period: "CurrentYear",
        label: "当期",
        duration_contexts: &[
            "CurrentYearDuration",
            "CurrentYearDuration_ConsolidatedMember",
            "CurrentYearDuration_NonConsolidatedMember",
        ],
        instant_contexts: &[
            "CurrentYearInstant",
            "CurrentYearInstant_ConsolidatedMember",
            "CurrentYearInstant_NonConsolidatedMember",
        ],
    },
    PeriodContext {
        period: "Prior1Year",
        label: "前期",
        duration_contexts: &[
            "Prior1YearDuration",
            "Prior1YearDuration_ConsolidatedMember",
            "Prior1YearDuration_NonConsolidatedMember",
        ],
        instant_contexts: &[
            "Prior1YearInstant",
            "Prior1YearInstant_ConsolidatedMember",
            "Prior1YearInstant_NonConsolidatedMember",
        ],
    },
    PeriodContext {
        period: "Prior2Year",
        label: "前々期",
        duration_contexts: &[
            "Prior2YearDuration",
            "Prior2YearDuration_ConsolidatedMember",
            "Prior2YearDuration_NonConsolidatedMember",
        ],
        instant_contexts: &[
            "Prior2YearInstant",
            "Prior2YearInstant_ConsolidatedMember",
            "Prior2YearInstant_NonConsolidatedMember",
        ],
    },
    PeriodContext {
        period: "Prior3Year",
        label: "三期前",
        duration_contexts: &[
            "Prior3YearDuration",
            "Prior3YearDuration_ConsolidatedMember",
            "Prior3YearDuration_NonConsolidatedMember",
        ],
        instant_contexts: &[
            "Prior3YearInstant",
            "Prior3YearInstant_ConsolidatedMember",
            "Prior3YearInstant_NonConsolidatedMember",
        ],
    },
    PeriodContext {
        period: "Prior4Year",
        label: "四期前",
        duration_contexts: &[
            "Prior4YearDuration",
            "Prior4YearDuration_ConsolidatedMember",
            "Prior4YearDuration_NonConsolidatedMember",
        ],
        instant_contexts: &[
            "Prior4YearInstant",
            "Prior4YearInstant_ConsolidatedMember",
            "Prior4YearInstant_NonConsolidatedMember",
        ],
    },
];

pub(in crate::getter) fn extract_company_overview(index: &XbrlFactIndex) -> CompanyOverview {
    CompanyOverview {
        company_history: text(index, COMPANY_HISTORY),
        employees: text(index, EMPLOYEES),
        business_results_summary: extract_business_results_summary(index),
    }
}

fn extract_business_results_summary(index: &XbrlFactIndex) -> Vec<BusinessResultsPeriod> {
    PERIOD_CONTEXTS
        .iter()
        .map(|period| {
            let dividend_per_share =
                dividend_per_share_value(index, &[DIVIDEND_PER_SHARE], period.duration_contexts);
            let earnings_per_share = f64_value(
                index,
                &[EARNINGS_PER_SHARE, EARNINGS_PER_SHARE_IFRS],
                period.duration_contexts,
            );

            BusinessResultsPeriod {
                period: period.period.to_owned(),
                label: period.label.to_owned(),
                operating_revenue: i64_value(
                    index,
                    OPERATING_REVENUE_IDS,
                    period.duration_contexts,
                ),
                ordinary_income: i64_value(index, &[ORDINARY_INCOME], period.duration_contexts),
                net_income: i64_value(
                    index,
                    &[
                        PROFIT_LOSS_ATTRIBUTABLE_TO_OWNERS_OF_PARENT,
                        PROFIT_LOSS_ATTRIBUTABLE_TO_OWNERS_OF_PARENT_IFRS,
                        NET_INCOME_LOSS,
                    ],
                    period.duration_contexts,
                ),
                net_assets: i64_value(
                    index,
                    &[NET_ASSETS, EQUITY_ATTRIBUTABLE_TO_OWNERS_OF_PARENT_IFRS],
                    period.instant_contexts,
                ),
                total_assets: i64_value(
                    index,
                    &[TOTAL_ASSETS, TOTAL_ASSETS_IFRS],
                    period.instant_contexts,
                ),
                capital_stock: i64_value(index, &[CAPITAL_STOCK], period.instant_contexts),
                issued_shares_total: i64_value(
                    index,
                    &[ISSUED_SHARES_TOTAL],
                    period.instant_contexts,
                ),
                net_assets_per_share: f64_value(
                    index,
                    &[NET_ASSETS_PER_SHARE],
                    period.instant_contexts,
                ),
                earnings_per_share,
                dividend_per_share,
                equity_ratio: f64_value(
                    index,
                    &[EQUITY_RATIO, EQUITY_RATIO_IFRS],
                    period.instant_contexts,
                ),
                roe: f64_value(index, &[ROE, ROE_IFRS], period.duration_contexts),
                per: f64_value(index, &[PER, PER_IFRS], period.duration_contexts),
                payout_ratio: f64_value(index, &[PAYOUT_RATIO], period.duration_contexts).or_else(
                    || payout_ratio_from_per_share(dividend_per_share, earnings_per_share),
                ),
                operating_cash_flow: i64_value(
                    index,
                    &[OPERATING_CASH_FLOW, OPERATING_CASH_FLOW_IFRS],
                    period.duration_contexts,
                ),
                investing_cash_flow: i64_value(
                    index,
                    &[INVESTING_CASH_FLOW, INVESTING_CASH_FLOW_IFRS],
                    period.duration_contexts,
                ),
                financing_cash_flow: i64_value(
                    index,
                    &[FINANCING_CASH_FLOW, FINANCING_CASH_FLOW_IFRS],
                    period.duration_contexts,
                ),
                cash_and_equivalents: i64_value(
                    index,
                    &[CASH_AND_EQUIVALENTS, CASH_AND_EQUIVALENTS_IFRS],
                    period.instant_contexts,
                ),
                total_shareholder_return: f64_value(
                    index,
                    &[TOTAL_SHAREHOLDER_RETURN],
                    period.instant_contexts,
                ),
                employees: i64_value(index, &[NUMBER_OF_EMPLOYEES], period.instant_contexts),
            }
        })
        .collect()
}

fn dividend_per_share_value(
    index: &XbrlFactIndex,
    element_ids: &[&str],
    contexts: &[&str],
) -> Option<f64> {
    element_ids.iter().find_map(|element_id| {
        contexts.iter().find_map(|context| {
            let fact = index.get(element_id, context)?;
            match fact.parse_f64() {
                Some(value) => Some(value),
                None if is_no_dividend_value(&fact.value) => Some(0.0),
                None => None,
            }
        })
    })
}

fn is_no_dividend_value(value: &str) -> bool {
    matches!(value.trim(), "－" | "-" | "ー" | "―")
}

fn payout_ratio_from_per_share(
    dividend_per_share: Option<f64>,
    earnings_per_share: Option<f64>,
) -> Option<f64> {
    let dividend_per_share = dividend_per_share?;
    let earnings_per_share = earnings_per_share?;
    if earnings_per_share <= 0.0 {
        return None;
    }

    Some(dividend_per_share / earnings_per_share)
}

fn text(index: &XbrlFactIndex, element_id: &str) -> Option<String> {
    index
        .first_by_element(element_id)?
        .value_as_str()
        .map(ToOwned::to_owned)
}

fn i64_value(index: &XbrlFactIndex, element_ids: &[&str], contexts: &[&str]) -> Option<i64> {
    element_ids.iter().find_map(|element_id| {
        index
            .first_by_contexts(element_id, contexts)
            .and_then(|fact| fact.parse_i64())
    })
}

fn f64_value(index: &XbrlFactIndex, element_ids: &[&str], contexts: &[&str]) -> Option<f64> {
    element_ids.iter().find_map(|element_id| {
        index
            .first_by_contexts(element_id, contexts)
            .and_then(|fact| fact.parse_f64())
    })
}

#[cfg(test)]
mod tests {
    use super::super::super::xbrl_fact::XbrlFact;
    use super::*;

    #[test]
    fn extracts_business_results_by_context_priority() {
        let index = XbrlFactIndex::new(vec![
            fact(
                "jpcrp_cor:OperatingRevenue1SummaryOfBusinessResults",
                "CurrentYearDuration",
                "200",
            ),
            fact(
                "jpcrp_cor:OperatingRevenue1SummaryOfBusinessResults",
                "CurrentYearDuration_ConsolidatedMember",
                "50",
            ),
            fact(
                "jpcrp_cor:OperatingRevenue1SummaryOfBusinessResults",
                "CurrentYearDuration_NonConsolidatedMember",
                "100",
            ),
            fact(
                PROFIT_LOSS_ATTRIBUTABLE_TO_OWNERS_OF_PARENT,
                "CurrentYearDuration",
                "80",
            ),
            fact(
                NET_INCOME_LOSS,
                "CurrentYearDuration_NonConsolidatedMember",
                "70",
            ),
            fact(NET_ASSETS, "Prior1YearInstant", "300"),
        ]);

        let overview = extract_company_overview(&index);
        let current = &overview.business_results_summary[0];
        let prior1 = &overview.business_results_summary[1];

        assert_eq!(current.operating_revenue, Some(200));
        assert_eq!(current.net_income, Some(80));
        assert_eq!(prior1.net_assets, Some(300));
    }

    #[test]
    fn extracts_net_sales_as_operating_revenue_for_general_business() {
        let index = XbrlFactIndex::new(vec![fact(
            "jpcrp_cor:NetSalesSummaryOfBusinessResults",
            "CurrentYearDuration_NonConsolidatedMember",
            "18138469000",
        )]);

        let overview = extract_company_overview(&index);
        let current = &overview.business_results_summary[0];

        assert_eq!(current.operating_revenue, Some(18138469000));
    }

    #[test]
    fn extracts_key_financial_data_revenue_as_operating_revenue() {
        let index = XbrlFactIndex::new(vec![fact(
            "jpcrp_cor:RevenueKeyFinancialData",
            "CurrentYearDuration",
            "10832411000",
        )]);

        let overview = extract_company_overview(&index);
        let current = &overview.business_results_summary[0];

        assert_eq!(current.operating_revenue, Some(10832411000));
    }

    #[test]
    fn extracts_operating_revenue2_as_operating_revenue() {
        let index = XbrlFactIndex::new(vec![fact(
            "jpcrp_cor:OperatingRevenue2SummaryOfBusinessResults",
            "CurrentYearDuration",
            "16447000000",
        )]);

        let overview = extract_company_overview(&index);
        let current = &overview.business_results_summary[0];

        assert_eq!(current.operating_revenue, Some(16447000000));
    }

    #[test]
    fn extracts_ordinary_income_summary_as_operating_revenue() {
        let index = XbrlFactIndex::new(vec![fact(
            "jpcrp_cor:OrdinaryIncomeSummaryOfBusinessResults",
            "CurrentYearDuration",
            "9002775000000",
        )]);

        let overview = extract_company_overview(&index);
        let current = &overview.business_results_summary[0];

        assert_eq!(current.operating_revenue, Some(9002775000000));
    }

    #[test]
    fn extracts_additional_business_result_metrics() {
        let index = XbrlFactIndex::new(vec![
            fact(
                CAPITAL_STOCK,
                "CurrentYearInstant_NonConsolidatedMember",
                "1000",
            ),
            fact(ISSUED_SHARES_TOTAL, "CurrentYearInstant", "2000"),
            fact(NET_ASSETS_PER_SHARE, "CurrentYearInstant", "5725.64"),
            fact(EARNINGS_PER_SHARE, "CurrentYearDuration", "951.18"),
            fact(DIVIDEND_PER_SHARE, "CurrentYearDuration", "951.00"),
            fact(PER, "CurrentYearDuration", "12.5"),
            fact(PAYOUT_RATIO, "CurrentYearDuration", "0.42"),
            fact(EQUITY_RATIO, "CurrentYearInstant", "0.31"),
            fact(OPERATING_CASH_FLOW, "CurrentYearDuration", "300"),
            fact(
                INVESTING_CASH_FLOW,
                "CurrentYearDuration_NonConsolidatedMember",
                "-50",
            ),
            fact(FINANCING_CASH_FLOW, "CurrentYearDuration", "25"),
            fact(CASH_AND_EQUIVALENTS, "CurrentYearInstant", "999"),
            fact(
                TOTAL_SHAREHOLDER_RETURN,
                "CurrentYearInstant_NonConsolidatedMember",
                "1.15",
            ),
            fact(NUMBER_OF_EMPLOYEES, "CurrentYearInstant", "1234"),
        ]);

        let overview = extract_company_overview(&index);
        let current = &overview.business_results_summary[0];

        assert_eq!(current.capital_stock, Some(1000));
        assert_eq!(current.issued_shares_total, Some(2000));
        assert_eq!(current.net_assets_per_share, Some(5725.64));
        assert_eq!(current.earnings_per_share, Some(951.18));
        assert_eq!(current.dividend_per_share, Some(951.0));
        assert_eq!(current.equity_ratio, Some(0.31));
        assert_eq!(current.per, Some(12.5));
        assert_eq!(current.payout_ratio, Some(0.42));
        assert_eq!(current.operating_cash_flow, Some(300));
        assert_eq!(current.investing_cash_flow, Some(-50));
        assert_eq!(current.financing_cash_flow, Some(25));
        assert_eq!(current.cash_and_equivalents, Some(999));
        assert_eq!(current.total_shareholder_return, Some(1.15));
        assert_eq!(current.employees, Some(1234));
    }

    #[test]
    fn treats_dash_dividend_per_share_as_zero() {
        let index = XbrlFactIndex::new(vec![
            fact(DIVIDEND_PER_SHARE, "CurrentYearDuration", "－"),
            fact(EARNINGS_PER_SHARE, "CurrentYearDuration", "100.0"),
        ]);

        let overview = extract_company_overview(&index);
        let current = &overview.business_results_summary[0];

        assert_eq!(current.dividend_per_share, Some(0.0));
        assert_eq!(current.payout_ratio, Some(0.0));
    }

    #[test]
    fn derives_payout_ratio_when_explicit_fact_is_missing() {
        let index = XbrlFactIndex::new(vec![
            fact(DIVIDEND_PER_SHARE, "CurrentYearDuration", "25.0"),
            fact(EARNINGS_PER_SHARE, "CurrentYearDuration", "100.0"),
        ]);

        let overview = extract_company_overview(&index);
        let current = &overview.business_results_summary[0];

        assert_eq!(current.dividend_per_share, Some(25.0));
        assert_eq!(current.payout_ratio, Some(0.25));
    }

    #[test]
    fn keeps_payout_ratio_empty_when_earnings_per_share_is_not_positive() {
        let index = XbrlFactIndex::new(vec![
            fact(DIVIDEND_PER_SHARE, "CurrentYearDuration", "10.0"),
            fact(EARNINGS_PER_SHARE, "CurrentYearDuration", "-50.0"),
        ]);

        let overview = extract_company_overview(&index);
        let current = &overview.business_results_summary[0];

        assert_eq!(current.dividend_per_share, Some(10.0));
        assert_eq!(current.payout_ratio, None);
    }

    #[test]
    fn extracts_ifrs_business_result_metrics() {
        let index = XbrlFactIndex::new(vec![
            fact(
                PROFIT_LOSS_ATTRIBUTABLE_TO_OWNERS_OF_PARENT_IFRS,
                "CurrentYearDuration",
                "100",
            ),
            fact(
                EQUITY_ATTRIBUTABLE_TO_OWNERS_OF_PARENT_IFRS,
                "CurrentYearInstant",
                "500",
            ),
            fact(TOTAL_ASSETS_IFRS, "CurrentYearInstant", "900"),
            fact(EARNINGS_PER_SHARE_IFRS, "CurrentYearDuration", "12.34"),
            fact(EQUITY_RATIO_IFRS, "CurrentYearInstant", "0.55"),
            fact(ROE_IFRS, "CurrentYearDuration", "0.12"),
            fact(PER_IFRS, "CurrentYearDuration", "15.2"),
            fact(OPERATING_CASH_FLOW_IFRS, "CurrentYearDuration", "300"),
            fact(INVESTING_CASH_FLOW_IFRS, "CurrentYearDuration", "-100"),
            fact(FINANCING_CASH_FLOW_IFRS, "CurrentYearDuration", "50"),
            fact(CASH_AND_EQUIVALENTS_IFRS, "CurrentYearInstant", "700"),
        ]);

        let overview = extract_company_overview(&index);
        let current = &overview.business_results_summary[0];

        assert_eq!(current.net_income, Some(100));
        assert_eq!(current.net_assets, Some(500));
        assert_eq!(current.total_assets, Some(900));
        assert_eq!(current.earnings_per_share, Some(12.34));
        assert_eq!(current.equity_ratio, Some(0.55));
        assert_eq!(current.roe, Some(0.12));
        assert_eq!(current.per, Some(15.2));
        assert_eq!(current.operating_cash_flow, Some(300));
        assert_eq!(current.investing_cash_flow, Some(-100));
        assert_eq!(current.financing_cash_flow, Some(50));
        assert_eq!(current.cash_and_equivalents, Some(700));
    }

    fn fact(element_id: &str, context_id: &str, value: &str) -> XbrlFact {
        XbrlFact {
            element_id: element_id.to_owned(),
            item_name: None,
            context_id: context_id.to_owned(),
            relative_year: None,
            consolidation: None,
            period_or_instant: None,
            unit_id: None,
            unit: None,
            value: value.to_owned(),
        }
    }
}
