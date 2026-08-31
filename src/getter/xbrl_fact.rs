use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
/// XBRL CSV の 1 行を読み込むための入力型。
pub struct XbrlCsvRecord {
    #[serde(rename = "要素ID")]
    pub element_id: String,
    #[serde(rename = "項目名")]
    pub item_name: String,
    #[serde(rename = "コンテキストID")]
    pub context_id: String,
    #[serde(rename = "相対年度")]
    pub relative_year: String,
    #[serde(rename = "連結・個別")]
    pub consolidation: String,
    #[serde(rename = "期間・時点")]
    pub period_or_instant: String,
    #[serde(rename = "ユニットID")]
    pub unit_id: String,
    #[serde(rename = "単位")]
    pub unit: String,
    #[serde(rename = "値")]
    pub value: String,
}

#[derive(Debug, Clone)]
/// XBRL CSV 上の 1 fact。
///
/// `element_id` は EDINET タクソノミ上の概念、`context_id` は「当期・前期」「連結・個別」
/// 「どの member に属するか」を表し、この 2 軸の組み合わせで意味が決まる。
#[allow(dead_code)]
pub struct XbrlFact {
    pub element_id: String,
    pub item_name: Option<String>,
    pub context_id: String,
    pub relative_year: Option<String>,
    pub consolidation: Option<String>,
    pub period_or_instant: Option<String>,
    pub unit_id: Option<String>,
    pub unit: Option<String>,
    pub value: String,
}

impl XbrlFact {
    /// EDINET の CSV では欠損値が空文字や全角ダッシュ (`－`) で表現されることがある。
    ///
    /// 特に経営指標サマリーでは「該当なし」を `－` で埋める提出会社があるため、
    /// 数値 zero ではなく欠損として扱う。
    pub fn value_as_str(&self) -> Option<&str> {
        let value = self.value.trim();
        if value.is_empty() || value == "－" {
            return None;
        }
        Some(value)
    }

    pub fn parse_i64(&self) -> Option<i64> {
        self.value_as_str()?.parse::<i64>().ok()
    }

    pub fn parse_f64(&self) -> Option<f64> {
        self.value_as_str()?.parse::<f64>().ok()
    }
}

impl From<XbrlCsvRecord> for XbrlFact {
    fn from(record: XbrlCsvRecord) -> Self {
        Self {
            element_id: record.element_id,
            item_name: empty_to_none(record.item_name),
            context_id: record.context_id,
            relative_year: empty_to_none(record.relative_year),
            consolidation: empty_to_none(record.consolidation),
            period_or_instant: empty_to_none(record.period_or_instant),
            unit_id: empty_to_none(record.unit_id),
            unit: empty_to_none(record.unit),
            value: record.value,
        }
    }
}

#[derive(Debug, Clone)]
/// XBRL fact を「タクソノミ要素」と「context」で引くための index。
///
/// EDINET の抽出では「どの概念か」だけでなく「どの期間・どの連結範囲か」が重要なので、
/// element_id 単独ではなく context を含めて参照できるようにしている。
pub struct XbrlFactIndex {
    facts: Vec<XbrlFact>,
    by_element: HashMap<String, Vec<usize>>,
    by_element_context: HashMap<(String, String), usize>,
}

impl XbrlFactIndex {
    pub fn new(facts: Vec<XbrlFact>) -> Self {
        let mut by_element: HashMap<String, Vec<usize>> = HashMap::new();
        let mut by_element_context: HashMap<(String, String), usize> = HashMap::new();

        for (index, fact) in facts.iter().enumerate() {
            by_element
                .entry(fact.element_id.clone())
                .or_default()
                .push(index);
            by_element_context
                .entry((fact.element_id.clone(), fact.context_id.clone()))
                .or_insert(index);
        }

        Self {
            facts,
            by_element,
            by_element_context,
        }
    }

    pub fn get(&self, element_id: &str, context_id: &str) -> Option<&XbrlFact> {
        let index = self
            .by_element_context
            .get(&(element_id.to_owned(), context_id.to_owned()))?;
        self.facts.get(*index)
    }

    pub fn facts_by_element<'a>(&'a self, element_id: &str) -> impl Iterator<Item = &'a XbrlFact> {
        self.by_element
            .get(element_id)
            .into_iter()
            .flat_map(|indexes| indexes.iter().filter_map(|index| self.facts.get(*index)))
    }

    pub fn first_by_element(&self, element_id: &str) -> Option<&XbrlFact> {
        self.facts_by_element(element_id)
            .find(|fact| fact.value_as_str().is_some())
    }

    /// 同じ taxonomy 要素が複数 context に載る前提で、呼び出し側の業務ルール順に探索する。
    ///
    /// たとえば「連結を優先して、なければ個別、その次に member なし」という
    /// EDINET 実務上の優先順を extractor 側から注入するための helper。
    pub fn first_by_contexts(&self, element_id: &str, contexts: &[&str]) -> Option<&XbrlFact> {
        contexts
            .iter()
            .filter_map(|context_id| self.get(element_id, context_id))
            .find(|fact| fact.value_as_str().is_some())
    }
}

fn empty_to_none(value: String) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexes_fact_by_element_and_context() {
        let index = XbrlFactIndex::new(vec![XbrlFact {
            element_id: "jpcrp_cor:BusinessRisksTextBlock".to_owned(),
            item_name: Some("事業等のリスク".to_owned()),
            context_id: "FilingDateInstant".to_owned(),
            relative_year: None,
            consolidation: None,
            period_or_instant: Some("時点".to_owned()),
            unit_id: None,
            unit: None,
            value: "risk text".to_owned(),
        }]);

        let fact = index
            .get("jpcrp_cor:BusinessRisksTextBlock", "FilingDateInstant")
            .expect("fact should be indexed");
        assert_eq!(fact.value_as_str(), Some("risk text"));
    }

    #[test]
    fn parses_missing_numeric_values_as_none() {
        let fact = XbrlFact {
            element_id: "jpcrp_cor:NetIncomeLossSummaryOfBusinessResults".to_owned(),
            item_name: None,
            context_id: "CurrentYearDuration".to_owned(),
            relative_year: None,
            consolidation: None,
            period_or_instant: None,
            unit_id: Some("JPY".to_owned()),
            unit: Some("円".to_owned()),
            value: "－".to_owned(),
        };

        assert_eq!(fact.value_as_str(), None);
        assert_eq!(fact.parse_i64(), None);
        assert_eq!(fact.parse_f64(), None);
    }
}
