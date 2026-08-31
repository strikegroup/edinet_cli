// SPDX-FileCopyrightText: 2026 Strike Co., Ltd.
// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use super::super::asr_report::{
    BalanceSheetItems, BalanceSheetPeriods, FinancialInformation, PrimaryFinancialStatements,
    ProfitAndLossItems, ProfitAndLossPeriods,
};
use super::super::xbrl_fact::XbrlFactIndex;

/// 「セグメント情報等の注記」本文。
const SEGMENT_INFORMATION_IDS: &[&str] = &[
    "jpcrp_cor:NotesSegmentInformationEtcConsolidatedFinancialStatementsTextBlock",
    "jpcrp_cor:NotesSegmentInformationEtcFinancialStatementsTextBlock",
    "jpigp_cor:NotesSegmentInformationConsolidatedFinancialStatementsIFRSTextBlock",
];
/// 総資産。
const ASSETS: &str = "jppfs_cor:Assets";
/// 流動資産。
const CURRENT_ASSETS: &str = "jppfs_cor:CurrentAssets";
/// 固定資産。
const NONCURRENT_ASSETS: &str = "jppfs_cor:NoncurrentAssets";
/// 流動負債。
const CURRENT_LIABILITIES: &str = "jppfs_cor:CurrentLiabilities";
/// 固定負債。
const NONCURRENT_LIABILITIES: &str = "jppfs_cor:NoncurrentLiabilities";
/// 純資産。
const NET_ASSETS: &str = "jppfs_cor:NetAssets";
/// 資本金。
const CAPITAL_STOCK: &str = "jppfs_cor:CapitalStock";
/// 資本剰余金。
const CAPITAL_SURPLUS: &str = "jppfs_cor:CapitalSurplus";
/// 利益剰余金。
const RETAINED_EARNINGS: &str = "jppfs_cor:RetainedEarnings";
/// 第5 経理の状況の売上トップライン候補。
///
/// 一般事業会社は NetSales、インフラ・金融などでは OperatingRevenue1 や
/// OrdinaryRevenues、特定金融業では OperatingRevenueSPF で出ることがある。
const OPERATING_REVENUE_IDS: &[&str] = &[
    "jppfs_cor:NetSales",
    "jppfs_cor:OperatingRevenue1",
    "jppfs_cor:OrdinaryRevenues",
    "jppfs_cor:OperatingRevenueSPF",
    "jpigp_cor:RevenueIFRS",
];
/// 営業費用。
const OPERATING_EXPENSES: &str = "jppfs_cor:OperatingExpenses";
/// 営業利益。
const OPERATING_INCOME: &str = "jppfs_cor:OperatingIncome";
/// 経常利益。
const ORDINARY_INCOME: &str = "jppfs_cor:OrdinaryIncome";
/// 税引前当期純利益。
const INCOME_BEFORE_INCOME_TAXES: &str = "jppfs_cor:IncomeBeforeIncomeTaxes";
/// 当期純利益または純損失。
const PROFIT_LOSS: &str = "jppfs_cor:ProfitLoss";
/// 財務諸表本体の context 候補。
///
/// 連結財務諸表なら suffix なしの context が本命で、単体中心の会社や
/// 個別財務諸表側では NonConsolidatedMember に出ることがある。
const CURRENT_YEAR_INSTANT_CONTEXTS: &[&str] = &[
    "CurrentYearInstant",
    "CurrentYearInstant_NonConsolidatedMember",
];
const PRIOR1_YEAR_INSTANT_CONTEXTS: &[&str] = &[
    "Prior1YearInstant",
    "Prior1YearInstant_NonConsolidatedMember",
];
const CURRENT_YEAR_DURATION_CONTEXTS: &[&str] = &[
    "CurrentYearDuration",
    "CurrentYearDuration_NonConsolidatedMember",
];
const PRIOR1_YEAR_DURATION_CONTEXTS: &[&str] = &[
    "Prior1YearDuration",
    "Prior1YearDuration_NonConsolidatedMember",
];

pub(in crate::getter) fn extract_financial_information(
    index: &XbrlFactIndex,
) -> FinancialInformation {
    FinancialInformation {
        segment_information: text_any(index, SEGMENT_INFORMATION_IDS),
        primary_statements: extract_primary_statements(index),
    }
}

fn extract_primary_statements(index: &XbrlFactIndex) -> PrimaryFinancialStatements {
    PrimaryFinancialStatements {
        balance_sheet: BalanceSheetPeriods {
            current: extract_balance_sheet(index, CURRENT_YEAR_INSTANT_CONTEXTS),
            prior: extract_balance_sheet(index, PRIOR1_YEAR_INSTANT_CONTEXTS),
        },
        profit_and_loss: ProfitAndLossPeriods {
            current: extract_profit_and_loss(index, CURRENT_YEAR_DURATION_CONTEXTS),
            prior: extract_profit_and_loss(index, PRIOR1_YEAR_DURATION_CONTEXTS),
        },
    }
}

fn extract_balance_sheet(index: &XbrlFactIndex, contexts: &[&str]) -> BalanceSheetItems {
    BalanceSheetItems {
        assets: i64_value(index, &[ASSETS], contexts),
        current_assets: i64_value(index, &[CURRENT_ASSETS], contexts),
        noncurrent_assets: i64_value(index, &[NONCURRENT_ASSETS], contexts),
        current_liabilities: i64_value(index, &[CURRENT_LIABILITIES], contexts),
        noncurrent_liabilities: i64_value(index, &[NONCURRENT_LIABILITIES], contexts),
        net_assets: i64_value(index, &[NET_ASSETS], contexts),
        capital_stock: i64_value(index, &[CAPITAL_STOCK], contexts),
        capital_surplus: i64_value(index, &[CAPITAL_SURPLUS], contexts),
        retained_earnings: i64_value(index, &[RETAINED_EARNINGS], contexts),
    }
}

fn extract_profit_and_loss(index: &XbrlFactIndex, contexts: &[&str]) -> ProfitAndLossItems {
    let operating_revenue = i64_value(index, OPERATING_REVENUE_IDS, contexts);
    let operating_income = i64_value(index, &[OPERATING_INCOME], contexts);

    ProfitAndLossItems {
        operating_revenue,
        operating_expenses: i64_value(index, &[OPERATING_EXPENSES], contexts).or_else(|| {
            operating_expenses_from_profit_and_loss(operating_revenue, operating_income)
        }),
        operating_income,
        ordinary_income: i64_value(index, &[ORDINARY_INCOME], contexts),
        income_before_income_taxes: i64_value(index, &[INCOME_BEFORE_INCOME_TAXES], contexts),
        profit_loss: i64_value(index, &[PROFIT_LOSS], contexts),
    }
}

fn operating_expenses_from_profit_and_loss(
    operating_revenue: Option<i64>,
    operating_income: Option<i64>,
) -> Option<i64> {
    Some(operating_revenue? - operating_income?)
}

fn text(index: &XbrlFactIndex, element_id: &str) -> Option<String> {
    index
        .first_by_element(element_id)?
        .value_as_str()
        .map(ToOwned::to_owned)
}

fn text_any(index: &XbrlFactIndex, element_ids: &[&str]) -> Option<String> {
    element_ids
        .iter()
        .find_map(|element_id| text(index, element_id))
}

fn i64_value(index: &XbrlFactIndex, element_ids: &[&str], contexts: &[&str]) -> Option<i64> {
    element_ids.iter().find_map(|element_id| {
        index
            .first_by_contexts(element_id, contexts)
            .and_then(|fact| fact.parse_i64())
    })
}

#[cfg(test)]
mod tests {
    use super::super::super::xbrl_fact::XbrlFact;
    use super::*;

    #[test]
    fn extracts_primary_financial_statements() {
        let index = XbrlFactIndex::new(vec![
            fact(
                "jpcrp_cor:NotesSegmentInformationEtcFinancialStatementsTextBlock",
                "FilingDateInstant",
                "セグメント本文",
            ),
            fact(ASSETS, "CurrentYearInstant", "1000"),
            fact(CURRENT_ASSETS, "CurrentYearInstant", "400"),
            fact(NONCURRENT_ASSETS, "CurrentYearInstant", "600"),
            fact(
                CURRENT_LIABILITIES,
                "CurrentYearInstant_NonConsolidatedMember",
                "250",
            ),
            fact(
                NONCURRENT_LIABILITIES,
                "CurrentYearInstant_NonConsolidatedMember",
                "150",
            ),
            fact(NET_ASSETS, "CurrentYearInstant", "600"),
            fact(CAPITAL_STOCK, "CurrentYearInstant", "100"),
            fact(CAPITAL_SURPLUS, "CurrentYearInstant", "50"),
            fact(RETAINED_EARNINGS, "CurrentYearInstant", "200"),
            fact(ASSETS, "Prior1YearInstant_NonConsolidatedMember", "900"),
            fact(NET_ASSETS, "Prior1YearInstant_NonConsolidatedMember", "500"),
            fact("jppfs_cor:NetSales", "CurrentYearDuration", "700"),
            fact(OPERATING_EXPENSES, "CurrentYearDuration", "500"),
            fact(OPERATING_INCOME, "CurrentYearDuration", "200"),
            fact(
                ORDINARY_INCOME,
                "CurrentYearDuration_NonConsolidatedMember",
                "180",
            ),
            fact(INCOME_BEFORE_INCOME_TAXES, "CurrentYearDuration", "170"),
            fact(PROFIT_LOSS, "CurrentYearDuration", "120"),
            fact(
                "jppfs_cor:OperatingRevenue1",
                "Prior1YearDuration_NonConsolidatedMember",
                "650",
            ),
            fact(
                "jppfs_cor:OperatingRevenueSPF",
                "CurrentYearDuration",
                "700",
            ),
            fact("jpigp_cor:RevenueIFRS", "Prior1YearDuration", "660"),
            fact(
                PROFIT_LOSS,
                "Prior1YearDuration_NonConsolidatedMember",
                "110",
            ),
        ]);

        let financial_information = extract_financial_information(&index);

        assert_eq!(
            financial_information.segment_information.as_deref(),
            Some("セグメント本文")
        );
        assert_eq!(
            financial_information
                .primary_statements
                .balance_sheet
                .current
                .assets,
            Some(1000)
        );
        assert_eq!(
            financial_information
                .primary_statements
                .balance_sheet
                .current
                .current_liabilities,
            Some(250)
        );
        assert_eq!(
            financial_information
                .primary_statements
                .balance_sheet
                .prior
                .assets,
            Some(900)
        );
        assert_eq!(
            financial_information
                .primary_statements
                .profit_and_loss
                .current
                .operating_revenue,
            Some(700)
        );
        assert_eq!(
            financial_information
                .primary_statements
                .profit_and_loss
                .current
                .ordinary_income,
            Some(180)
        );
        assert_eq!(
            financial_information
                .primary_statements
                .profit_and_loss
                .prior
                .operating_revenue,
            Some(650)
        );
        assert_eq!(
            financial_information
                .primary_statements
                .profit_and_loss
                .prior
                .profit_loss,
            Some(110)
        );
    }

    #[test]
    fn extracts_ifrs_revenue_and_segment_information() {
        let index = XbrlFactIndex::new(vec![
            fact(
                "jpigp_cor:NotesSegmentInformationConsolidatedFinancialStatementsIFRSTextBlock",
                "CurrentYearDuration",
                "IFRSセグメント本文",
            ),
            fact("jpigp_cor:RevenueIFRS", "CurrentYearDuration", "1000"),
            fact("jpigp_cor:RevenueIFRS", "Prior1YearDuration", "900"),
        ]);

        let financial_information = extract_financial_information(&index);

        assert_eq!(
            financial_information.segment_information.as_deref(),
            Some("IFRSセグメント本文")
        );
        assert_eq!(
            financial_information
                .primary_statements
                .profit_and_loss
                .current
                .operating_revenue,
            Some(1000)
        );
        assert_eq!(
            financial_information
                .primary_statements
                .profit_and_loss
                .prior
                .operating_revenue,
            Some(900)
        );
    }

    #[test]
    fn derives_operating_expenses_when_explicit_fact_is_missing() {
        let index = XbrlFactIndex::new(vec![
            fact("jppfs_cor:NetSales", "CurrentYearDuration", "700"),
            fact(OPERATING_INCOME, "CurrentYearDuration", "200"),
        ]);

        let financial_information = extract_financial_information(&index);

        assert_eq!(
            financial_information
                .primary_statements
                .profit_and_loss
                .current
                .operating_expenses,
            Some(500)
        );
    }

    #[test]
    fn keeps_explicit_operating_expenses_over_derived_value() {
        let index = XbrlFactIndex::new(vec![
            fact("jppfs_cor:NetSales", "CurrentYearDuration", "700"),
            fact(OPERATING_EXPENSES, "CurrentYearDuration", "480"),
            fact(OPERATING_INCOME, "CurrentYearDuration", "200"),
        ]);

        let financial_information = extract_financial_information(&index);

        assert_eq!(
            financial_information
                .primary_statements
                .profit_and_loss
                .current
                .operating_expenses,
            Some(480)
        );
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
