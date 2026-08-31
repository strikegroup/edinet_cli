use anyhow::Context;

use super::asr_report::AsrReport;
use super::xbrl_fact::{XbrlCsvRecord, XbrlFact, XbrlFactIndex};

/// 読み込み済み CSV を有価証券報告書の抽出結果に変換する。
pub(in crate::getter) fn extract_asr_report_from_csv(
    reader: &mut csv::Reader<std::io::Cursor<Vec<u8>>>,
) -> anyhow::Result<AsrReport> {
    let records = reader
        .deserialize::<XbrlCsvRecord>()
        .map(|result| result.context("failed to deserialize XBRL CSV record"))
        .collect::<anyhow::Result<Vec<_>>>()?;
    let facts = records.into_iter().map(XbrlFact::from).collect();
    let index = XbrlFactIndex::new(facts);

    Ok(build_report(&index))
}

fn build_report(index: &XbrlFactIndex) -> AsrReport {
    AsrReport {
        company_overview: super::extractors::extract_company_overview(index),
        business_overview: super::extractors::extract_business_overview(index),
        facilities: super::extractors::extract_facilities(index),
        corporate_information: super::extractors::extract_corporate_information(index),
        financial_information: super::extractors::extract_financial_information(index),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_chapter_based_summary_from_xbrl_csv_records() -> anyhow::Result<()> {
        let tsv = [
            "要素ID\t項目名\tコンテキストID\t相対年度\t連結・個別\t期間・時点\tユニットID\t単位\t値",
            "jpcrp_cor:BusinessRisksTextBlock\t事業等のリスク\tFilingDateInstant\t\t\t時点\t\t\tリスク本文",
            "jpcrp_cor:CompanyHistoryTextBlock\t沿革\tFilingDateInstant\t\t\t時点\t\t\t沿革本文",
            "jpcrp_cor:OperatingRevenue1SummaryOfBusinessResults\t営業収益\tCurrentYearDuration\t当期\t連結\t期間\tJPY\t円\t123",
            "jpcrp_cor:GovernanceTextBlock\tガバナンス\tFilingDateInstant\t\t\t時点\t\t\tガバナンス本文",
            "jpcrp_cor:AverageAnnualSalaryInformationAboutReportingCompanyInformationAboutEmployees\t平均年間給与\tCurrentYearInstant_NonConsolidatedMember\t当期\t個別\t時点\tJPY\t円\t8000000",
            "jpcrp_cor:RatioOfFemaleDirectorsAndOtherOfficers\t女性役員比率\tFilingDateInstant\t\t\t時点\t\t\t0.25",
            "jppfs_cor:Assets\t資産\tCurrentYearInstant\t当期\t連結\t時点\tJPY\t円\t1000",
            "jppfs_cor:NetSales\t売上高\tCurrentYearDuration\t当期\t連結\t期間\tJPY\t円\t700",
        ]
        .join("\n");
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .from_reader(std::io::Cursor::new(tsv.into_bytes()));

        let report = extract_asr_report_from_csv(&mut reader)?;

        assert_eq!(
            report.business_overview.risks.as_deref(),
            Some("リスク本文")
        );
        assert_eq!(
            report.company_overview.company_history.as_deref(),
            Some("沿革本文")
        );
        assert_eq!(
            report.company_overview.business_results_summary[0].operating_revenue,
            Some(123)
        );
        assert_eq!(
            report.business_overview.governance.as_deref(),
            Some("ガバナンス本文")
        );
        assert_eq!(
            report
                .business_overview
                .human_capital_metrics
                .average_annual_salary,
            Some(8000000)
        );
        assert_eq!(
            report
                .corporate_information
                .governance_metrics
                .ratio_female_directors,
            Some(0.25)
        );
        assert_eq!(
            report
                .financial_information
                .primary_statements
                .balance_sheet
                .current
                .assets,
            Some(1000)
        );
        assert_eq!(
            report
                .financial_information
                .primary_statements
                .profit_and_loss
                .current
                .operating_revenue,
            Some(700)
        );
        Ok(())
    }
}
