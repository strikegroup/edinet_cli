// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use super::super::asr_report::{
    CorporateInformation, GovernanceMetrics, PolicyShareholding, PolicyShareholdingCategory,
    PolicyShareholdingHolderScope, PolicyShareholdingPeriod,
};
use super::super::xbrl_fact::XbrlFactIndex;

/// 「株式の保有状況」本文。
const SHAREHOLDING: &str = "jpcrp_cor:ShareholdingsTextBlock";
/// 「大株主の状況」本文。
const MAJOR_SHAREHOLDERS: &str = "jpcrp_cor:MajorShareholdersTextBlock";
/// 「配当政策」本文。
const DIVIDEND_POLICY: &str = "jpcrp_cor:DividendPolicyTextBlock";
/// 「役員の状況」本文。
const OFFICERS: &str = "jpcrp_cor:InformationAboutOfficersTextBlock";
/// 「コーポレート・ガバナンスの概要」本文。
const CORPORATE_GOVERNANCE: &str = "jpcrp_cor:OverviewOfCorporateGovernanceTextBlock";
/// 「役員の報酬等」本文。
const OFFICER_COMPENSATION: &str = "jpcrp_cor:RemunerationForDirectorsAndOtherOfficersTextBlock";
/// 役員区分別に開示される報酬総額の taxonomy 要素。
const TOTAL_OFFICER_COMPENSATION: &str =
    "jpcrp_cor:TotalAmountOfRemunerationEtcRemunerationEtcByCategoryOfDirectorsAndOtherOfficers";
/// 監査報酬額。
const AUDIT_FEE: &str = "jpcrp_cor:AuditFeesReportingCompany";
/// 女性役員比率。
const RATIO_FEMALE_DIRECTORS: &str = "jpcrp_cor:RatioOfFemaleDirectorsAndOtherOfficers";
/// 男性役員数。
const NUM_MALE_DIRECTORS: &str = "jpcrp_cor:NumberOfMaleDirectorsAndOtherOfficers";
/// 女性役員数。
const NUM_FEMALE_DIRECTORS: &str = "jpcrp_cor:NumberOfFemaleDirectorsAndOtherOfficers";
/// 第4 提出会社の状況では、役員構成は「提出日時点の体制」なので FilingDateInstant、
/// 監査報酬は「当期に発生した金額」なので CurrentYearDuration で読む。
const OFFICER_INSTANT_CONTEXTS: &[&str] = &[
    "FilingDateInstant",
    "CurrentYearInstant",
    "RecordDateInstant",
];
const CURRENT_YEAR_DURATION_CONTEXTS: &[&str] = &["CurrentYearDuration"];
const LARGEST_HOLDING_COMPANY_NAME: &str =
    "jpcrp_cor:NameOfGroupCompanyHoldingLargestAmountOfInvestmentSharesInGroup";
const SECOND_LARGEST_HOLDING_COMPANY_NAME: &str =
    "jpcrp_cor:NameOfGroupCompanyHoldingSecondLargestAmountOfInvestmentSharesInGroup";

struct PolicyShareholdingElements {
    category: PolicyShareholdingCategory,
    holder_scope: PolicyShareholdingHolderScope,
    issue_name: &'static str,
    shares: &'static str,
    shares_not_disclosed: &'static str,
    book_value: &'static str,
    book_value_not_disclosed: &'static str,
    purpose: &'static str,
    business_alliance: &'static str,
    quantitative_effects: &'static str,
    reason_for_increase: &'static str,
    combined_purpose_and_effects: &'static str,
    issuer_holds_reporting_company_shares: &'static str,
}

macro_rules! holding_elements {
    ($category:ident, $scope:ident, $kind:literal, $equity_kind:literal, $holder:literal) => {
        PolicyShareholdingElements {
            category: PolicyShareholdingCategory::$category,
            holder_scope: PolicyShareholdingHolderScope::$scope,
            issue_name: concat!(
                "jpcrp_cor:NameOfSecuritiesDetailsOf",
                $equity_kind,
                "HeldForPurposesOtherThanPureInvestment",
                $holder
            ),
            shares: concat!(
                "jpcrp_cor:NumberOfSharesHeldDetailsOf",
                $equity_kind,
                "HeldForPurposesOtherThanPureInvestment",
                $holder
            ),
            shares_not_disclosed: concat!(
                "jpcrp_cor:NumberOfSharesNotDisclosedAsBelowThresholdDetailsOf",
                $kind,
                "HeldForPurposesOtherThanPureInvestment",
                $holder
            ),
            book_value: concat!(
                "jpcrp_cor:BookValueDetailsOf",
                $equity_kind,
                "HeldForPurposesOtherThanPureInvestment",
                $holder
            ),
            book_value_not_disclosed: concat!(
                "jpcrp_cor:CarryingAmountNotDisclosedAsBelowThresholdDetailsOf",
                $kind,
                "HeldForPurposesOtherThanPureInvestment",
                $holder
            ),
            purpose: concat!(
                "jpcrp_cor:PurposesOfHoldingDetailsOf",
                $equity_kind,
                "HeldForPurposesOtherThanPureInvestment",
                $holder
            ),
            business_alliance: concat!(
                "jpcrp_cor:OverviewOfBusinessAllianceDetailsOf",
                $kind,
                "HeldForPurposesOtherThanPureInvestment",
                $holder
            ),
            quantitative_effects: concat!(
                "jpcrp_cor:QuantitativeEffectsOfShareholdingDetailsOf",
                $kind,
                "HeldForPurposesOtherThanPureInvestment",
                $holder
            ),
            reason_for_increase: concat!(
                "jpcrp_cor:ReasonForIncreaseInNumberOfSharesDetailsOf",
                $kind,
                "HeldForPurposesOtherThanPureInvestment",
                $holder
            ),
            combined_purpose_and_effects: concat!(
                "jpcrp_cor:PurposeOfShareholdingOverviewOfBusinessAllianceQuantitativeEffectsOfShareholdingAndReasonForIncreaseInNumberOfSharesDetailsOf",
                $kind,
                "HeldForPurposesOtherThanPureInvestment",
                $holder
            ),
            issuer_holds_reporting_company_shares: concat!(
                "jpcrp_cor:WhetherIssuerOfAforementionedSharesHoldsReportingCompanysSharesDetailsOf",
                $kind,
                "HeldForPurposesOtherThanPureInvestment",
                $holder
            ),
        }
    };
}

const POLICY_SHAREHOLDING_ELEMENTS: &[PolicyShareholdingElements] = &[
    holding_elements!(
        SpecifiedInvestment,
        Reporting,
        "SpecifiedInvestmentShares",
        "SpecifiedInvestmentEquitySecurities",
        "ReportingCompany"
    ),
    holding_elements!(
        DeemedHolding,
        Reporting,
        "DeemedHoldingsOfShares",
        "DeemedHoldingsOfEquitySecurities",
        "ReportingCompany"
    ),
    holding_elements!(
        SpecifiedInvestment,
        Largest,
        "SpecifiedInvestmentShares",
        "SpecifiedInvestmentEquitySecurities",
        "LargestHoldingCompany"
    ),
    holding_elements!(
        DeemedHolding,
        Largest,
        "DeemedHoldingsOfShares",
        "DeemedHoldingsOfEquitySecurities",
        "LargestHoldingCompany"
    ),
    holding_elements!(
        SpecifiedInvestment,
        SecondLargest,
        "SpecifiedInvestmentShares",
        "SpecifiedInvestmentEquitySecurities",
        "SecondLargestHoldingCompany"
    ),
    holding_elements!(
        DeemedHolding,
        SecondLargest,
        "DeemedHoldingsOfShares",
        "DeemedHoldingsOfEquitySecurities",
        "SecondLargestHoldingCompany"
    ),
];

pub(in crate::getter) fn extract_corporate_information(
    index: &XbrlFactIndex,
) -> CorporateInformation {
    CorporateInformation {
        shareholding: text(index, SHAREHOLDING),
        policy_shareholdings: extract_policy_shareholdings(index),
        major_shareholders: text(index, MAJOR_SHAREHOLDERS),
        dividend_policy: text(index, DIVIDEND_POLICY),
        officers: text(index, OFFICERS),
        corporate_governance: text(index, CORPORATE_GOVERNANCE),
        officer_compensation: text(index, OFFICER_COMPENSATION),
        governance_metrics: extract_governance_metrics(index),
    }
}

fn extract_policy_shareholdings(index: &XbrlFactIndex) -> Vec<PolicyShareholding> {
    POLICY_SHAREHOLDING_ELEMENTS
        .iter()
        .flat_map(|elements| extract_policy_shareholdings_for_elements(index, elements))
        .collect()
}

fn extract_policy_shareholdings_for_elements(
    index: &XbrlFactIndex,
    elements: &PolicyShareholdingElements,
) -> Vec<PolicyShareholding> {
    let mut issues = BTreeMap::new();
    for fact in index.facts_by_element(elements.issue_name) {
        let Some(row_number) = row_number(&fact.context_id) else {
            continue;
        };
        let Some(issue_name) = fact.value_as_str() else {
            continue;
        };

        // 銘柄名は原則 CurrentYearInstant でタグ付けされる。提出者拡張等で同じ行が
        // 複数回現れても、当期の値を優先する。
        let priority = if fact.context_id.starts_with("CurrentYearInstant") {
            0
        } else {
            1
        };
        issues
            .entry(row_number)
            .and_modify(|entry: &mut (u8, String)| {
                if priority < entry.0 {
                    *entry = (priority, issue_name.to_owned());
                }
            })
            .or_insert_with(|| (priority, issue_name.to_owned()));
    }

    issues
        .into_iter()
        .map(|(row_number, (_, issue_name))| PolicyShareholding {
            category: elements.category,
            holder_scope: elements.holder_scope,
            holder_name: policy_shareholding_holder_name(index, elements.holder_scope),
            row_number,
            issue_name,
            current: extract_policy_shareholding_period(
                index,
                elements,
                row_number,
                "CurrentYearInstant",
            ),
            prior: extract_policy_shareholding_period(
                index,
                elements,
                row_number,
                "Prior1YearInstant",
            ),
            purpose_of_shareholding: string_for_row(
                index,
                elements.purpose,
                row_number,
                "CurrentYearInstant",
            ),
            business_alliance_overview: string_for_row(
                index,
                elements.business_alliance,
                row_number,
                "CurrentYearInstant",
            ),
            quantitative_effects: string_for_row(
                index,
                elements.quantitative_effects,
                row_number,
                "CurrentYearInstant",
            ),
            reason_for_increase: string_for_row(
                index,
                elements.reason_for_increase,
                row_number,
                "CurrentYearInstant",
            ),
            combined_purpose_and_effects: string_for_row(
                index,
                elements.combined_purpose_and_effects,
                row_number,
                "CurrentYearInstant",
            ),
            issuer_holds_reporting_company_shares: string_for_row(
                index,
                elements.issuer_holds_reporting_company_shares,
                row_number,
                "CurrentYearInstant",
            ),
        })
        .collect()
}

fn policy_shareholding_holder_name(
    index: &XbrlFactIndex,
    holder_scope: PolicyShareholdingHolderScope,
) -> Option<String> {
    match holder_scope {
        PolicyShareholdingHolderScope::Reporting => None,
        PolicyShareholdingHolderScope::Largest => text(index, LARGEST_HOLDING_COMPANY_NAME),
        PolicyShareholdingHolderScope::SecondLargest => {
            text(index, SECOND_LARGEST_HOLDING_COMPANY_NAME)
        }
    }
}

fn extract_policy_shareholding_period(
    index: &XbrlFactIndex,
    elements: &PolicyShareholdingElements,
    row_number: u32,
    period: &str,
) -> PolicyShareholdingPeriod {
    PolicyShareholdingPeriod {
        shares: i64_for_row(index, elements.shares, row_number, period),
        book_value: i64_for_row(index, elements.book_value, row_number, period),
        shares_not_disclosed: fact_exists_for_row(
            index,
            elements.shares_not_disclosed,
            row_number,
            period,
        ),
        book_value_not_disclosed: fact_exists_for_row(
            index,
            elements.book_value_not_disclosed,
            row_number,
            period,
        ),
    }
}

fn row_number(context_id: &str) -> Option<u32> {
    context_id.split('_').find_map(|part| {
        part.strip_prefix("Row")?
            .strip_suffix("Member")?
            .parse()
            .ok()
    })
}

fn fact_exists_for_row(
    index: &XbrlFactIndex,
    element_id: &str,
    row_number: u32,
    period: &str,
) -> bool {
    fact_for_row(index, element_id, row_number, period).is_some()
}

fn i64_for_row(
    index: &XbrlFactIndex,
    element_id: &str,
    row_number: u32,
    period: &str,
) -> Option<i64> {
    fact_for_row(index, element_id, row_number, period)?.parse_i64()
}

fn string_for_row(
    index: &XbrlFactIndex,
    element_id: &str,
    row_number: u32,
    period: &str,
) -> Option<String> {
    fact_for_row(index, element_id, row_number, period)?
        .value_as_str()
        .map(ToOwned::to_owned)
}

fn fact_for_row<'a>(
    index: &'a XbrlFactIndex,
    element_id: &str,
    target_row_number: u32,
    period: &str,
) -> Option<&'a super::super::xbrl_fact::XbrlFact> {
    index.facts_by_element(element_id).find(|fact| {
        fact.context_id.starts_with(period)
            && row_number(&fact.context_id) == Some(target_row_number)
    })
}

fn extract_governance_metrics(index: &XbrlFactIndex) -> GovernanceMetrics {
    GovernanceMetrics {
        ratio_female_directors: f64_value(index, RATIO_FEMALE_DIRECTORS, OFFICER_INSTANT_CONTEXTS),
        total_officer_compensation: total_officer_compensation(index),
        audit_fee: i64_value(index, AUDIT_FEE, CURRENT_YEAR_DURATION_CONTEXTS),
        num_male_directors: i64_value(index, NUM_MALE_DIRECTORS, OFFICER_INSTANT_CONTEXTS),
        num_female_directors: i64_value(index, NUM_FEMALE_DIRECTORS, OFFICER_INSTANT_CONTEXTS),
    }
}

fn text(index: &XbrlFactIndex, element_id: &str) -> Option<String> {
    index
        .first_by_element(element_id)?
        .value_as_str()
        .map(ToOwned::to_owned)
}

fn i64_value(index: &XbrlFactIndex, element_id: &str, contexts: &[&str]) -> Option<i64> {
    index.first_by_contexts(element_id, contexts)?.parse_i64()
}

fn f64_value(index: &XbrlFactIndex, element_id: &str, contexts: &[&str]) -> Option<f64> {
    index.first_by_contexts(element_id, contexts)?.parse_f64()
}

fn total_officer_compensation(index: &XbrlFactIndex) -> Option<i64> {
    let total: i64 = index
        .facts_by_element(TOTAL_OFFICER_COMPENSATION)
        .filter(|fact| {
            // `TotalAmountOfRemunerationEtc...` は「役員報酬総額」という 1 つの経営概念だが、
            // EDINET 上は取締役・監査役・社外役員などの区分 member ごとに分割される。
            // そのため第4 提出会社の状況で人が読む総額に戻すには、当期の役員関連 member を合算する必要がある。
            fact.context_id.starts_with("CurrentYearDuration_")
                && fact.context_id.ends_with("Member")
                && (fact.context_id.contains("Directors")
                    || fact.context_id.contains("Auditors")
                    || fact.context_id.contains("Officers"))
        })
        .filter_map(|fact| fact.parse_i64())
        .sum();

    if total > 0 { Some(total) } else { None }
}

#[cfg(test)]
mod tests {
    use super::super::super::xbrl_fact::XbrlFact;
    use super::*;

    #[test]
    fn extracts_governance_metrics() {
        let index = XbrlFactIndex::new(vec![
            fact(RATIO_FEMALE_DIRECTORS, "FilingDateInstant", "0.25"),
            fact(NUM_MALE_DIRECTORS, "FilingDateInstant", "8"),
            fact(NUM_FEMALE_DIRECTORS, "FilingDateInstant", "2"),
            fact(AUDIT_FEE, "CurrentYearDuration", "120"),
            fact(
                TOTAL_OFFICER_COMPENSATION,
                "CurrentYearDuration_DirectorsExcludingOutsideDirectorsMember",
                "300",
            ),
            fact(
                TOTAL_OFFICER_COMPENSATION,
                "CurrentYearDuration_OutsideDirectorsAndOtherOfficersMember",
                "40",
            ),
            fact(
                CORPORATE_GOVERNANCE,
                "FilingDateInstant",
                "コーポレートガバナンス本文",
            ),
        ]);

        let corporate_information = extract_corporate_information(&index);

        assert_eq!(
            corporate_information.corporate_governance.as_deref(),
            Some("コーポレートガバナンス本文")
        );
        assert_eq!(
            corporate_information
                .governance_metrics
                .ratio_female_directors,
            Some(0.25)
        );
        assert_eq!(
            corporate_information
                .governance_metrics
                .total_officer_compensation,
            Some(340)
        );
        assert_eq!(
            corporate_information.governance_metrics.audit_fee,
            Some(120)
        );
        assert_eq!(
            corporate_information.governance_metrics.num_male_directors,
            Some(8)
        );
        assert_eq!(
            corporate_information
                .governance_metrics
                .num_female_directors,
            Some(2)
        );
    }

    #[test]
    fn extracts_officer_counts_from_current_year_instant() {
        let index = XbrlFactIndex::new(vec![
            fact(RATIO_FEMALE_DIRECTORS, "CurrentYearInstant", "0.364"),
            fact(NUM_MALE_DIRECTORS, "CurrentYearInstant", "7"),
            fact(NUM_FEMALE_DIRECTORS, "CurrentYearInstant", "4"),
        ]);

        let corporate_information = extract_corporate_information(&index);

        assert_eq!(
            corporate_information
                .governance_metrics
                .ratio_female_directors,
            Some(0.364)
        );
        assert_eq!(
            corporate_information.governance_metrics.num_male_directors,
            Some(7)
        );
        assert_eq!(
            corporate_information
                .governance_metrics
                .num_female_directors,
            Some(4)
        );
    }

    #[test]
    fn extracts_shareholdings_text_and_policy_shareholdings_by_row() {
        let specified = &POLICY_SHAREHOLDING_ELEMENTS[2];
        let deemed = &POLICY_SHAREHOLDING_ELEMENTS[3];
        let index = XbrlFactIndex::new(vec![
            fact(SHAREHOLDING, "FilingDateInstant", "株式の保有状況本文"),
            fact(
                LARGEST_HOLDING_COMPANY_NAME,
                "CurrentYearInstant",
                "株式会社最大保有",
            ),
            fact(
                specified.issue_name,
                "CurrentYearInstant_Row61Member",
                "株式会社テスト",
            ),
            fact(specified.shares, "CurrentYearInstant_Row61Member", "100"),
            fact(specified.shares, "Prior1YearInstant_Row61Member", "90"),
            fact(
                specified.book_value,
                "CurrentYearInstant_Row61Member",
                "1200",
            ),
            fact(
                specified.book_value,
                "Prior1YearInstant_Row61Member",
                "1000",
            ),
            fact(
                specified.combined_purpose_and_effects,
                "CurrentYearInstant_Row61Member",
                "取引関係の維持を目的に保有",
            ),
            fact(
                specified.issuer_holds_reporting_company_shares,
                "CurrentYearInstant_Row61Member",
                "有",
            ),
            fact(
                deemed.issue_name,
                "CurrentYearInstant_Row1Member",
                "株式会社みなし",
            ),
            fact(
                deemed.shares_not_disclosed,
                "Prior1YearInstant_Row1Member",
                "記載省略",
            ),
            fact(
                deemed.purpose,
                "CurrentYearInstant_Row1Member",
                "議決権行使の指図権を保有",
            ),
        ]);

        let corporate_information = extract_corporate_information(&index);

        assert_eq!(
            corporate_information.shareholding.as_deref(),
            Some("株式の保有状況本文")
        );
        assert_eq!(corporate_information.policy_shareholdings.len(), 2);

        let specified_holding = &corporate_information.policy_shareholdings[0];
        assert_eq!(
            specified_holding.category,
            PolicyShareholdingCategory::SpecifiedInvestment
        );
        assert_eq!(
            specified_holding.holder_scope,
            PolicyShareholdingHolderScope::Largest
        );
        assert_eq!(specified_holding.row_number, 61);
        assert_eq!(
            specified_holding.holder_name.as_deref(),
            Some("株式会社最大保有")
        );
        assert_eq!(specified_holding.issue_name, "株式会社テスト");
        assert_eq!(specified_holding.current.shares, Some(100));
        assert_eq!(specified_holding.prior.shares, Some(90));
        assert_eq!(specified_holding.current.book_value, Some(1200));
        assert_eq!(specified_holding.prior.book_value, Some(1000));
        assert_eq!(
            specified_holding.combined_purpose_and_effects.as_deref(),
            Some("取引関係の維持を目的に保有")
        );
        assert_eq!(
            specified_holding
                .issuer_holds_reporting_company_shares
                .as_deref(),
            Some("有")
        );

        let deemed_holding = &corporate_information.policy_shareholdings[1];
        assert_eq!(
            deemed_holding.category,
            PolicyShareholdingCategory::DeemedHolding
        );
        assert_eq!(deemed_holding.issue_name, "株式会社みなし");
        assert!(deemed_holding.prior.shares_not_disclosed);
        assert_eq!(
            deemed_holding.purpose_of_shareholding.as_deref(),
            Some("議決権行使の指図権を保有")
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
