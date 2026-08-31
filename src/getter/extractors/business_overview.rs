use super::super::asr_report::{BusinessOverview, HumanCapitalMetrics};
use super::super::xbrl_fact::XbrlFactIndex;

/// 「事業の内容」本文。
const BUSINESS_DESCRIPTION: &str = "jpcrp_cor:DescriptionOfBusinessTextBlock";
/// MD&A に相当する「経営者による財政状態、経営成績及びキャッシュ・フローの状況の分析」。
const PERFORMANCE: &str =
    "jpcrp_cor:ManagementAnalysisOfFinancialPositionOperatingResultsAndCashFlowsTextBlock";
/// 「経営方針、経営環境及び対処すべき課題等」本文。
const ISSUES_TO_ADDRESS: &str =
    "jpcrp_cor:BusinessPolicyBusinessEnvironmentIssuesToAddressEtcTextBlock";
/// 「事業等のリスク」本文。
const RISKS: &str = "jpcrp_cor:BusinessRisksTextBlock";
/// サステナビリティ関連の統合 TextBlock。
const SUSTAINABILITY: &str =
    "jpcrp_cor:DisclosureOfSustainabilityRelatedFinancialInformationTextBlock";
/// サステナビリティ注記内の「ガバナンス」節。
const GOVERNANCE: &str = "jpcrp_cor:GovernanceTextBlock";
/// サステナビリティ注記内の「戦略」節。
const STRATEGY: &str = "jpcrp_cor:StrategyTextBlock";
/// サステナビリティ注記内の「リスク管理」節。
const RISK_MANAGEMENT: &str = "jpcrp_cor:RiskManagementTextBlock";
/// サステナビリティ注記内の「指標及び目標」節。
const METRICS_AND_TARGETS: &str = "jpcrp_cor:MetricsAndTargetsTextBlock";
/// 「人材の育成及び社内環境整備に関する方針」本文。
const HUMAN_RESOURCES_POLICY: &str =
    "jpcrp_cor:PolicyOnDevelopmentOfHumanResourcesAndInternalEnvironmentStrategyTextBlock";
/// 人的資本方針に紐づく「指標・目標・実績」の説明本文。
const HUMAN_CAPITAL_METRICS_DESCRIPTION: &str = "jpcrp_cor:DescriptionOfMetricsRelatedToPolicyOnDevelopmentOfHumanResourcesAndInternalEnvironmentAndTargetsAndPerformanceUsingSuchMetricsMetricsAndTargetsTextBlock";
/// 「研究開発活動」本文。
const RESEARCH_AND_DEVELOPMENT: &str = "jpcrp_cor:ResearchAndDevelopmentActivitiesTextBlock";
/// 「重要な契約等」本文。
const CRITICAL_CONTRACT_IDS: &[&str] = &[
    "jpcrp_cor:CriticalContractsTextBlock",
    "jpcrp_cor:CriticalContractsForOperationTextBlock",
];

/// 提出会社または対象範囲の従業員数。
const NUMBER_OF_EMPLOYEES: &str = "jpcrp_cor:NumberOfEmployees";
/// 提出会社の平均年間給与。
const AVERAGE_ANNUAL_SALARY: &str =
    "jpcrp_cor:AverageAnnualSalaryInformationAboutReportingCompanyInformationAboutEmployees";
/// 提出会社の平均年齢。
const AVERAGE_AGE_YEARS: &str =
    "jpcrp_cor:AverageAgeYearsInformationAboutReportingCompanyInformationAboutEmployees";
/// 提出会社の平均勤続年数。
const AVERAGE_SERVICE_YEARS: &str = "jpcrp_cor:AverageLengthOfServiceYearsInformationAboutReportingCompanyInformationAboutEmployees";
/// 女性管理職比率。提出会社版と連結子会社版の両命名を吸収する。
const RATIO_FEMALE_MANAGERS_IDS: &[&str] = &[
    "jpcrp_cor:RatioOfFemaleEmployeesInManagerialPositionsMetricsOfReportingCompany",
    "jpcrp_cor:RatioOfFemaleEmployeesInManagerialPositionsMetricsOfConsolidatedSubsidiaries",
];
/// 男性育児休業取得率。法令準拠の正式名が長く、報告会社版と連結子会社版で命名が揺れる。
const RATIO_MALE_CHILDCARE_LEAVE_IDS: &[&str] = &[
    "jpcrp_cor:AllEmployeesRatioOfMaleEmployeesTakingChildcareLeaveMetricsOfConsolidatedSubsidiaries",
    "jpcrp_cor:AllEmployeesCalculatedBasedOnProvisionsOfArticle714Item1OfOrdinanceForEnforcementOfActOnChildcareLeaveCaregiverLeaveAndOtherMeasuresForTheWelfareOfWorkersCaringForChildrenOrOtherFamilyMembersRatioOfMaleEmployeesTakingChildcareLeaveMetricsOfReportingCompany",
    "jpcrp_cor:AllEmployeesCalculatedBasedOnProvisionsOfArticle714Item2OfOrdinanceForEnforcementOfActOnChildcareLeaveCaregiverLeaveAndOtherMeasuresForTheWelfareOfWorkersCaringForChildrenOrOtherFamilyMembersRatioOfMaleEmployeesTakingChildcareLeaveMetricsOfReportingCompany",
    "jpcrp_cor:AllEmployeesCalculatedBasedOnProvisionsOfActOnPromotionOfWomensActiveEngagementInProfessionalLifeRatioOfMaleEmployeesTakingChildcareLeaveMetricsOfReportingCompany",
    "jpcrp_cor:RegularEmployeesCalculatedBasedOnProvisionsOfActOnPromotionOfWomensActiveEngagementInProfessionalLifeRatioOfMaleEmployeesTakingChildcareLeaveMetricsOfReportingCompany",
];
/// 男女賃金差異（全労働者）。
const GENDER_PAY_GAP_ALL_IDS: &[&str] = &[
    "jpcrp_cor:AllEmployeesDifferencesInWagesBetweenMaleAndFemaleEmployeesMetricsOfReportingCompany",
    "jpcrp_cor:AllEmployeesDifferencesInWagesBetweenMaleAndFemaleEmployeesMetricsOfConsolidatedSubsidiaries",
];
/// 男女賃金差異（正規雇用労働者）。
const GENDER_PAY_GAP_REGULAR_IDS: &[&str] = &[
    "jpcrp_cor:RegularEmployeesDifferencesInWagesBetweenMaleAndFemaleEmployeesMetricsOfReportingCompany",
    "jpcrp_cor:RegularEmployeesDifferencesInWagesBetweenMaleAndFemaleEmployeesMetricsOfConsolidatedSubsidiaries",
];
/// 男女賃金差異（非正規雇用労働者）。
const GENDER_PAY_GAP_NON_REGULAR_IDS: &[&str] = &[
    "jpcrp_cor:NonRegularEmployeesDifferencesInWagesBetweenMaleAndFemaleEmployeesMetricsOfReportingCompany",
    "jpcrp_cor:NonRegularEmployeesDifferencesInWagesBetweenMaleAndFemaleEmployeesMetricsOfConsolidatedSubsidiaries",
];

/// 第2 事業の状況の人的資本指標は、提出会社単体の雇用データなら NonConsolidatedMember、
/// 連結子会社ベースの内訳行なら Row1Member に載ることがある。
///
/// 「サステナビリティ関連財務情報の注記」の実務上、同じ KPI でも context の切り方が揺れるため優先順を持つ。
const HUMAN_CAPITAL_INSTANT_CONTEXTS: &[&str] = &[
    "CurrentYearInstant_NonConsolidatedMember",
    "CurrentYearInstant_Row1Member",
    "CurrentYearInstant_ConsolidatedMember",
    "CurrentYearInstant",
];

pub(in crate::getter) fn extract_business_overview(index: &XbrlFactIndex) -> BusinessOverview {
    BusinessOverview {
        business_description: text(index, BUSINESS_DESCRIPTION),
        performance: text(index, PERFORMANCE),
        issues_to_address: text(index, ISSUES_TO_ADDRESS),
        risks: text(index, RISKS),
        sustainability: text(index, SUSTAINABILITY),
        governance: text(index, GOVERNANCE),
        strategy: text(index, STRATEGY),
        risk_management: text(index, RISK_MANAGEMENT),
        metrics_and_targets: text(index, METRICS_AND_TARGETS),
        human_resources_policy: text(index, HUMAN_RESOURCES_POLICY),
        human_capital_metrics_description: text(index, HUMAN_CAPITAL_METRICS_DESCRIPTION),
        human_capital_metrics: extract_human_capital_metrics(index),
        research_and_development: text(index, RESEARCH_AND_DEVELOPMENT),
        critical_contracts: text_any(index, CRITICAL_CONTRACT_IDS),
    }
}

fn extract_human_capital_metrics(index: &XbrlFactIndex) -> HumanCapitalMetrics {
    HumanCapitalMetrics {
        employees_count: i64_value(
            index,
            &[NUMBER_OF_EMPLOYEES],
            HUMAN_CAPITAL_INSTANT_CONTEXTS,
        ),
        average_annual_salary: i64_value(
            index,
            &[AVERAGE_ANNUAL_SALARY],
            HUMAN_CAPITAL_INSTANT_CONTEXTS,
        ),
        average_age_years: f64_value(index, &[AVERAGE_AGE_YEARS], HUMAN_CAPITAL_INSTANT_CONTEXTS),
        average_service_years: f64_value(
            index,
            &[AVERAGE_SERVICE_YEARS],
            HUMAN_CAPITAL_INSTANT_CONTEXTS,
        ),
        ratio_female_managers: f64_value(
            index,
            RATIO_FEMALE_MANAGERS_IDS,
            HUMAN_CAPITAL_INSTANT_CONTEXTS,
        ),
        ratio_male_childcare_leave: f64_value(
            index,
            RATIO_MALE_CHILDCARE_LEAVE_IDS,
            HUMAN_CAPITAL_INSTANT_CONTEXTS,
        ),
        gender_pay_gap_all: f64_value(
            index,
            GENDER_PAY_GAP_ALL_IDS,
            HUMAN_CAPITAL_INSTANT_CONTEXTS,
        ),
        gender_pay_gap_regular: f64_value(
            index,
            GENDER_PAY_GAP_REGULAR_IDS,
            HUMAN_CAPITAL_INSTANT_CONTEXTS,
        ),
        gender_pay_gap_non_regular: f64_value(
            index,
            GENDER_PAY_GAP_NON_REGULAR_IDS,
            HUMAN_CAPITAL_INSTANT_CONTEXTS,
        ),
    }
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

/// 人的資本 taxonomy は同じ意味の KPI が `...ReportingCompany` と
/// `...ConsolidatedSubsidiaries` の 2 系統で定義されていることがある。
/// そのため「女性管理職比率」などの業務概念から複数 element_id を束ねて探す。
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
    fn extracts_business_overview_texts_and_human_capital_metrics() {
        let index = XbrlFactIndex::new(vec![
            fact(GOVERNANCE, "FilingDateInstant", "ガバナンス本文"),
            fact(STRATEGY, "FilingDateInstant", "戦略本文"),
            fact(
                "jpcrp_cor:CriticalContractsTextBlock",
                "FilingDateInstant",
                "重要な契約等本文",
            ),
            fact(HUMAN_RESOURCES_POLICY, "FilingDateInstant", "人材方針本文"),
            fact(
                HUMAN_CAPITAL_METRICS_DESCRIPTION,
                "FilingDateInstant",
                "人的資本指標本文",
            ),
            fact(
                NUMBER_OF_EMPLOYEES,
                "CurrentYearInstant_NonConsolidatedMember",
                "100",
            ),
            fact(
                AVERAGE_ANNUAL_SALARY,
                "CurrentYearInstant_NonConsolidatedMember",
                "8000000",
            ),
            fact(
                AVERAGE_AGE_YEARS,
                "CurrentYearInstant_NonConsolidatedMember",
                "41.2",
            ),
            fact(
                AVERAGE_SERVICE_YEARS,
                "CurrentYearInstant_NonConsolidatedMember",
                "12.5",
            ),
            fact(
                "jpcrp_cor:RatioOfFemaleEmployeesInManagerialPositionsMetricsOfReportingCompany",
                "CurrentYearInstant_NonConsolidatedMember",
                "0.31",
            ),
            fact(
                "jpcrp_cor:AllEmployeesRatioOfMaleEmployeesTakingChildcareLeaveMetricsOfConsolidatedSubsidiaries",
                "CurrentYearInstant_Row1Member",
                "0.85",
            ),
            fact(
                "jpcrp_cor:AllEmployeesDifferencesInWagesBetweenMaleAndFemaleEmployeesMetricsOfReportingCompany",
                "CurrentYearInstant_NonConsolidatedMember",
                "0.78",
            ),
            fact(
                "jpcrp_cor:RegularEmployeesDifferencesInWagesBetweenMaleAndFemaleEmployeesMetricsOfConsolidatedSubsidiaries",
                "CurrentYearInstant_Row1Member",
                "0.81",
            ),
            fact(
                "jpcrp_cor:NonRegularEmployeesDifferencesInWagesBetweenMaleAndFemaleEmployeesMetricsOfReportingCompany",
                "CurrentYearInstant_NonConsolidatedMember",
                "0.92",
            ),
        ]);

        let overview = extract_business_overview(&index);

        assert_eq!(overview.governance.as_deref(), Some("ガバナンス本文"));
        assert_eq!(overview.strategy.as_deref(), Some("戦略本文"));
        assert_eq!(
            overview.human_resources_policy.as_deref(),
            Some("人材方針本文")
        );
        assert_eq!(
            overview.human_capital_metrics_description.as_deref(),
            Some("人的資本指標本文")
        );
        assert_eq!(
            overview.critical_contracts.as_deref(),
            Some("重要な契約等本文")
        );
        assert_eq!(overview.human_capital_metrics.employees_count, Some(100));
        assert_eq!(
            overview.human_capital_metrics.average_annual_salary,
            Some(8000000)
        );
        assert_eq!(overview.human_capital_metrics.average_age_years, Some(41.2));
        assert_eq!(
            overview.human_capital_metrics.average_service_years,
            Some(12.5)
        );
        assert_eq!(
            overview.human_capital_metrics.ratio_female_managers,
            Some(0.31)
        );
        assert_eq!(
            overview.human_capital_metrics.ratio_male_childcare_leave,
            Some(0.85)
        );
        assert_eq!(
            overview.human_capital_metrics.gender_pay_gap_all,
            Some(0.78)
        );
        assert_eq!(
            overview.human_capital_metrics.gender_pay_gap_regular,
            Some(0.81)
        );
        assert_eq!(
            overview.human_capital_metrics.gender_pay_gap_non_regular,
            Some(0.92)
        );
    }

    #[test]
    fn extracts_additional_male_childcare_leave_elements() {
        let index = XbrlFactIndex::new(vec![fact(
            "jpcrp_cor:AllEmployeesCalculatedBasedOnProvisionsOfArticle714Item2OfOrdinanceForEnforcementOfActOnChildcareLeaveCaregiverLeaveAndOtherMeasuresForTheWelfareOfWorkersCaringForChildrenOrOtherFamilyMembersRatioOfMaleEmployeesTakingChildcareLeaveMetricsOfReportingCompany",
            "CurrentYearInstant_NonConsolidatedMember",
            "0.846",
        )]);

        let overview = extract_business_overview(&index);

        assert_eq!(
            overview.human_capital_metrics.ratio_male_childcare_leave,
            Some(0.846)
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
