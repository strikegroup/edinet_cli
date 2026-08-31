use super::super::asr_report::Facilities;
use super::super::xbrl_fact::XbrlFactIndex;

const CAPITAL_EXPENDITURES: &str = "jpcrp_cor:OverviewOfCapitalExpendituresEtcTextBlock";
const MAJOR_FACILITIES: &str = "jpcrp_cor:MajorFacilitiesTextBlock";
const FACILITY_PLANS: &str = "jpcrp_cor:PlannedAdditionsRetirementsEtcOfFacilitiesTextBlock";

pub(in crate::getter) fn extract_facilities(index: &XbrlFactIndex) -> Facilities {
    Facilities {
        capital_expenditures: text(index, CAPITAL_EXPENDITURES),
        major_facilities: text(index, MAJOR_FACILITIES),
        facility_plans: text(index, FACILITY_PLANS),
    }
}

fn text(index: &XbrlFactIndex, element_id: &str) -> Option<String> {
    index
        .first_by_element(element_id)?
        .value_as_str()
        .map(ToOwned::to_owned)
}
