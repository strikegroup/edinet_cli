# XBRL Fact 抽出基盤リファクタリング計画

## 目的

有価証券報告書の XBRL CSV を、TextBlock と数値項目を区別せず扱える抽出基盤へ作り直す。

現在の抽出処理は `要素ID -> 値` の単純な対応を前提としているため、同一要素 ID が複数コンテキストに現れる数値項目を正しく扱いにくい。
XBRL の構造に合わせ、`要素ID + コンテキストID` を基本キーとする fact ベースの抽出へ移行する。

## 基本方針

XBRL CSV の 1 行を XBRL fact として読み込み、検索用 index を構築する。
TextBlock、数値、日付、真偽値はいずれも fact として保持し、用途別の extractor が必要な値を取り出す。

```text
XBRL CSV
  -> XbrlCsvRecord
  -> XbrlFact
  -> XbrlFactIndex
  -> chapter extractors
  -> AsrReport
```

旧来の flat な `AsrSummaryContent` と `serde_json::Map` 経由の deserialize は廃止済みとし、現在の抽出処理では使わない。
出力構造は有価証券報告書の章立てに沿って再設計する。

## 出力構造

内部の CSV 解析結果は `AsrReport` として扱う。
書類メタデータとの結合は CLI/API の出力末端で行い、抽出処理の中核型には含めない。
`AsrDocumentMetadata` は `document_metadatas` 由来の ASR 書類 metadata とし、`AsrReport` は XBRL CSV から抽出した報告内容だけを表す。

```rust
pub struct AsrReport {
    pub company_overview: CompanyOverview,
    pub business_overview: BusinessOverview,
    pub facilities: Facilities,
    pub corporate_information: CorporateInformation,
    pub financial_information: FinancialInformation,
}
```

初期実装では、既存 TextBlock 項目と経営指標サマリーに関係する章を対象にする。

```rust
pub struct CompanyOverview {
    pub company_history: Option<String>,
    pub employees: Option<String>,
    pub business_results_summary: Vec<BusinessResultsPeriod>,
}

pub struct BusinessOverview {
    pub business_description: Option<String>,
    pub performance: Option<String>,
    pub issues_to_address: Option<String>,
    pub risks: Option<String>,
    pub sustainability: Option<String>,
    pub research_and_development: Option<String>,
    pub critical_contracts: Option<String>,
}

pub struct Facilities {
    pub capital_expenditures: Option<String>,
    pub major_facilities: Option<String>,
    pub facility_plans: Option<String>,
}

pub struct CorporateInformation {
    pub shareholding: Option<String>,
    pub major_shareholders: Option<String>,
    pub dividend_policy: Option<String>,
    pub officers: Option<String>,
    pub corporate_governance: Option<String>,
    pub officer_compensation: Option<String>,
}

pub struct FinancialInformation {
    pub segment_information: Option<String>,
}
```

`business_results_summary` は、有価証券報告書の「主要な経営指標等の推移」に対応する。

```rust
pub struct BusinessResultsPeriod {
    pub period: String,
    pub label: String,
    pub operating_revenue: Option<i64>,
    pub ordinary_income: Option<i64>,
    pub net_income: Option<i64>,
    pub net_assets: Option<i64>,
    pub total_assets: Option<i64>,
    pub equity_ratio: Option<f64>,
    pub roe: Option<f64>,
    pub employees: Option<i64>,
}
```

## XBRL Fact 基盤

### `XbrlCsvRecord`

`XbrlCsvRecord` は XBRL CSV deserialize 専用の内部型とする。

```rust
struct XbrlCsvRecord {
    element_id: String,
    item_name: String,
    context_id: String,
    relative_year: String,
    consolidation: String,
    period_or_instant: String,
    unit_id: String,
    unit: String,
    value: String,
}
```

CSV 列との対応は次の通り。

| CSV 列 | フィールド |
|---|---|
| 要素ID | `element_id` |
| 項目名 | `item_name` |
| コンテキストID | `context_id` |
| 相対年度 | `relative_year` |
| 連結・個別 | `consolidation` |
| 期間・時点 | `period_or_instant` |
| ユニットID | `unit_id` |
| 単位 | `unit` |
| 値 | `value` |

### `XbrlFact`

`XbrlFact` は getter 内で扱う抽出単位とする。
空文字の列は `Option<String>` に正規化するが、fact 自体は保持する。

```rust
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
```

型解釈は `XbrlFact` 側に寄せる。

```rust
impl XbrlFact {
    pub fn value_as_str(&self) -> Option<&str>;
    pub fn parse_i64(&self) -> Option<i64>;
    pub fn parse_f64(&self) -> Option<f64>;
}
```

`value_as_str` は空文字と `"－"` を値なしとして扱う。
数値 parse は値だけを変換し、単位変換は行わない。

### `XbrlFactIndex`

`XbrlFactIndex` は検索だけを担当する。

```rust
pub struct XbrlFactIndex {
    // internal indexes
}

impl XbrlFactIndex {
    pub fn get(&self, element_id: &str, context_id: &str) -> Option<&XbrlFact>;
    pub fn facts_by_element(&self, element_id: &str) -> impl Iterator<Item = &XbrlFact>;
    pub fn first_by_element(&self, element_id: &str) -> Option<&XbrlFact>;
    pub fn first_by_contexts(&self, element_id: &str, contexts: &[&str]) -> Option<&XbrlFact>;
}
```

`get` は `element_id + context_id` の完全一致で fact を返す。
`first_by_element` は TextBlock など、同一要素 ID の代表値を取得する用途で使う。
`first_by_contexts` は数値抽出時に、コンテキスト優先順で fact を取得する用途で使う。

## Extractor 構成

抽出処理は有価証券報告書の章単位に分割する。

```text
extract_company_overview
extract_business_overview
extract_facilities
extract_corporate_information
extract_financial_information
```

`extract_asr_report_from_csv` は次の責務だけを持つ。

1. CSV を `XbrlCsvRecord` として読む
2. `XbrlFactIndex` を作る
3. 各章 extractor を呼び出して `AsrReport` を組み立てる

## 経営指標サマリー

`business_results_summary` は 5 期分を抽出する。

```text
CurrentYear
Prior1Year
Prior2Year
Prior3Year
Prior4Year
```

各期の context 候補は、連結優先・個別フォールバックとする。

```text
{Period}Duration_ConsolidatedMember
{Period}Duration_NonConsolidatedMember
{Period}Duration
```

初期対象項目は次の通り。

| フィールド | XBRL 要素 ID |
|---|---|
| `operating_revenue` | `jpcrp_cor:OperatingRevenue1SummaryOfBusinessResults` |
| `ordinary_income` | `jpcrp_cor:OrdinaryIncomeLossSummaryOfBusinessResults` |
| `net_income` | `jpcrp_cor:NetIncomeLossSummaryOfBusinessResults` |
| `net_assets` | `jpcrp_cor:NetAssetsSummaryOfBusinessResults` |
| `total_assets` | `jpcrp_cor:TotalAssetsSummaryOfBusinessResults` |
| `equity_ratio` | `jpcrp_cor:EquityToAssetRatioSummaryOfBusinessResults` |
| `roe` | `jpcrp_cor:RateOfReturnOnEquitySummaryOfBusinessResults` |
| `employees` | `jpcrp_cor:NumberOfEmployees` |

## 変更対象

- `src/getter/asr_report.rs`
  - 章立てベースの `AsrReport` を定義する。
- `src/getter/extract_asr_report.rs`
  - `serde_json::Map` 経由の組み立てを廃止し、`XbrlFactIndex` 経由の抽出へ変更する。
- `src/getter/xbrl_fact.rs`
  - `XbrlCsvRecord`、`XbrlFact`、`XbrlFactIndex` を追加する。
- `src/getter/extractors/company_overview.rs`
  - 第 1 章「企業の概況」を抽出する。
- `src/getter/extractors/business_overview.rs`
  - 第 2 章「事業の状況」を抽出する。
- `src/getter/extractors/facilities.rs`
  - 第 3 章「設備の状況」を抽出する。
- `src/getter/extractors/corporate_information.rs`
  - 第 4 章「提出会社の状況」を抽出する。
- `src/getter/extractors/financial_information.rs`
  - 第 5 章「経理の状況」を抽出する。
- `src/getter.rs`
  - 外部 API と内部モジュールの公開範囲を定義する。

## 実装手順

1. `xbrl_fact.rs` を追加し、XBRL CSV 9 列を `XbrlFactIndex` に変換する。
2. `AsrReport` と章別 struct を定義する。
3. 既存 TextBlock 抽出を章別 extractor に移す。
4. `BusinessResultsPeriod` と 5 期分抽出を `company_overview` に追加する。
5. `extract_asr_report_from_csv` を新フローに差し替える。
6. 既存テストを新構造に合わせて更新し、fact index の単体テストを追加する。
7. `cargo check` と `cargo test` で確認する。
8. ローカル CSV キャッシュまたは `get --doc-id` で JSON 形状を確認する。

## テスト方針

- XBRL CSV 9 列を読めること。
- `element_id + context_id` で fact を取得できること。
- 空文字 fact が保持されること。
- `value_as_str` が空文字と `"－"` を値なしとして扱うこと。
- `parse_i64` / `parse_f64` が数値を変換できること。
- 既存 TextBlock が章別 struct に入ること。
- 5 期分の `business_results_summary` が context 優先順で抽出されること。

## リスク

- JSON 形状は破壊的に変わる。
- README の出力項目説明は後続で更新が必要になる。
- 連結・個別 context は実データ差があるため、サンプル CSV で調整が必要になる可能性がある。
- 単位変換は行わないため、単位付き出力が必要な場合は別途 `XbrlFact` の単位情報を出力へ反映する設計が必要になる。
