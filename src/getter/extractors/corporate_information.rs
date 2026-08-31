// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use super::super::asr_report::{CorporateInformation, GovernanceMetrics};
use super::super::xbrl_fact::XbrlFactIndex;

/// 「株式の保有状況」本文。
const SHAREHOLDING: &str = "jpcrp_cor:ShareholdingByShareholderCategoryTextBlock";
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

pub(in crate::getter) fn extract_corporate_information(
    index: &XbrlFactIndex,
) -> CorporateInformation {
    CorporateInformation {
        shareholding: text(index, SHAREHOLDING),
        major_shareholders: text(index, MAJOR_SHAREHOLDERS),
        dividend_policy: text(index, DIVIDEND_POLICY),
        officers: text(index, OFFICERS),
        corporate_governance: text(index, CORPORATE_GOVERNANCE),
        officer_compensation: text(index, OFFICER_COMPENSATION),
        governance_metrics: extract_governance_metrics(index),
    }
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
