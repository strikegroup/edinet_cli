# XBRL CSV ヒューリスティック抽出 実装ガイド

## 概要

EDINET が提供する XBRL の CSV（`type=5` で取得、`XBRL_TO_CSV/` 配下）には、
同一の要素 ID が **コンテキスト ID 別に複数行** 存在します。
「どの行を使うか」を決める判断ロジクが「ヒューリスティック」の核心です。

本ドキュメントは、サンプル CSV（`data/csv/` 配下）と EDINET API 仕様書（`docs/ESE140206.pdf`）の調査結果をもとに、
**フィールドカテゴリ別の抽出ルールを実装レベルで**まとめます。
追加すべきフィールドの全体像は `docs/xbrl_extraction_improvement.md` を参照してください。

現在の抽出基盤は `XbrlCsvRecord`、`XbrlFact`、`XbrlFactIndex` を中心に構成されています。実装全体の方針は
[`xbrl_fact_extraction_refactor_plan.md`](./xbrl_fact_extraction_refactor_plan.md) を参照してください。

---

## CSV ファイルの形式

### エンコーディング・区切り文字

- 文字コード：**UTF-16LE**（BOM なし）
- 区切り文字：**タブ（`\t`）**
- 行末：`\r\n`

現在の `load_local_csv.rs` はこれを正しく処理済みです。

### 列定義

| 列番号 | 列名 | 説明 |
|---|---|---|
| 1 | `要素ID` | XBRL 要素 ID（名前空間プレフィックス付き） |
| 2 | `項目名` | 日本語の項目名 |
| 3 | `コンテキストID` | **絞り込みのキー**（後述） |
| 4 | `相対年度` | 当期・前期・前々期・三期前・四期前 など |
| 5 | `連結・個別` | 連結 / 個別 / その他 |
| 6 | `期間・時点` | 期間（Duration）/ 時点（Instant） |
| 7 | `ユニットID` | `JPY` / `JPYPerShares` / `shares` / `pure` など |
| 8 | `単位` | 「円」など（`pure` の場合は空） |
| 9 | `値` | 数値または HTML テキスト（TextBlock の場合） |

### ファイル命名規則

```
jpcrp030000-asr-001_{EDINETコード}-000_{決算期末日}_01_{提出日}.csv
```

`asr` を含むファイルが有価証券報告書本体のデータです（`load_asr_report.rs` の `find_asr_csv_path` はこれで絞り込んでいます）。

---

## コンテキスト ID の体系

コンテキスト ID は `{期間種別}{年度区分}_{属性...}` の形式で構成されます。

### 期間種別

| パターン | 意味 |
|---|---|
| `CurrentYearDuration` | 当期（期間） |
| `CurrentYearInstant` | 当期末（時点） |
| `Prior1YearDuration` | 前期（期間） |
| `Prior1YearInstant` | 前期末（時点） |
| `Prior2YearDuration` | 前々期（期間） |
| `Prior2YearInstant` | 前々期末（時点） |
| `Prior3YearDuration` | 三期前（期間） |
| `Prior4YearDuration` | 四期前（期間） |
| `FilingDateInstant` | 提出日時点（DEI 情報専用） |

### 属性サフィックス

コンテキスト ID の末尾に `_` で区切られた属性が付加されます。

| サフィックス | 意味 |
|---|---|
| `_NonConsolidatedMember` | 個別（単体）財務諸表 |
| （サフィックスなし） | 連結財務諸表（または個別のみ会社の汎用） |
| `_ConsolidatedAccountingGroupMember` | 連結会計グループ（まれ） |
| `_CapitalStockMember` など | 株主資本変動計算書の内訳 |
| `_ReportableSegmentsMember` | セグメント合計 |
| `_jpcrp030000-asr_{EDINET}-000{セグメント名}...` | 企業固有セグメント |
| `_No{N}MajorShareholdersMember` | 第 N 位大株主 |

### 実サンプルから確認したパターン

| 会社 | 会計基準 | 連結 | 経営指標サマリーのコンテキスト |
|---|---|---|---|
| E03736（日産FS） | Japan GAAP | false | `{Period}_NonConsolidatedMember` のみ |
| E05407（青山財産NW） | Japan GAAP | true | `{Period}`（連結）と `{Period}_NonConsolidatedMember`（個別）両方 |
| E05031（トヨタFS） | Japan GAAP | true | `{Period}`（連結）と `{Period}_NonConsolidatedMember`（個別）両方 |
| E04948（光通信） | IFRS | true | IFRS 要素は `{Period}`、個別は `{Period}_NonConsolidatedMember` |
| E32380（ストライク） | Japan GAAP | false | `{Period}_NonConsolidatedMember` のみ |
| E00424（ダイドーGHD） | Japan GAAP | true | `{Period}`（連結）と `{Period}_NonConsolidatedMember`（個別）両方 |

---

## ステップ1：会計基準と連結区分の判定

最初に DEI 情報（`FilingDateInstant` コンテキスト）を読み取り、以降の絞り込みに使います。

### 使用する要素 ID

```
jpdei_cor:AccountingStandardsDEI           → "Japan GAAP" / "IFRS" / "US GAAP" など
jpdei_cor:WhetherConsolidatedFinancialStatementsArePreparedDEI  → "true" / "false"
jpdei_cor:EDINETCodeDEI                    → EDINETコード（例："E00424"）
jpdei_cor:FilerNameInJapaneseDEI           → 会社名（日本語）
jpdei_cor:SecurityCodeDEI                  → 証券コード（例："25900"、上場していない場合は "－"）
jpdei_cor:CurrentFiscalYearStartDateDEI    → 当期開始日（YYYY-MM-DD）
jpdei_cor:CurrentFiscalYearEndDateDEI      → 当期終了日（YYYY-MM-DD）
```

DEI 情報は CSV 中でコンテキスト `FilingDateInstant` の行に存在し、**1 行のみ**です。

### 判定フロー

```
accounting_standard = DEI["AccountingStandardsDEI"]  // "Japan GAAP" | "IFRS" | "US GAAP"
is_consolidated     = DEI["WhetherConsolidatedFinancialStatementsArePreparedDEI"] == "true"

if is_consolidated:
    main_context_suffix = ""              // SummaryOfBusinessResults の連結値
    sub_context_suffix  = "_NonConsolidatedMember"  // 個別値（サブ）
else:
    main_context_suffix = "_NonConsolidatedMember"  // 個別のみ会社
    sub_context_suffix  = None
```

---

## ステップ2：基盤となる共通処理

### 欠損値の扱い

CSV 上で数値が「該当なし」の場合、値は **`"－"`（全角ダッシュ U+FF0D）** です。
空文字・パース失敗と合わせて `None` にします。

```rust
fn parse_numeric(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.is_empty() || s == "\u{FF0D}" { return None; }
    s.parse::<f64>().ok()
}
fn parse_i64(s: &str) -> Option<i64> { parse_numeric(s).map(|f| f as i64) }
```

### `XbrlCsvRecord` と `XbrlFact` に読み込む

```rust
// src/getter/xbrl_fact.rs
#[derive(Debug, Deserialize)]
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
```

### インデックス構築

```rust
fn build_index(records: impl IntoIterator<Item = XbrlCsvRecord>) -> XbrlFactIndex {
    XbrlFactIndex::new(records.into_iter().map(XbrlFact::from).collect())
}
```

### コンテキスト優先順でフォールバック取得

```rust
fn get_first<'a>(
    index: &'a XbrlFactIndex,
    element_id: &str,
    contexts: &[&str],
) -> Option<&'a str> {
    index.first_by_contexts(element_id, contexts)?.value_as_str()
}
```

### 名前空間プレフィックスの種類

| プレフィックス | 内容 |
|---|---|
| `jpdei_cor:` | DEI（提出日時点のメタ情報） |
| `jpcrp_cor:` | 共通様式タクソノミ（大多数の標準項目） |
| `jppfs_cor:` | 財務諸表様式タクソノミ（B/S・P/L・CF の勘定科目） |
| `jpcrp030000-asr_{EDINET}-000:` | **企業固有拡張**（横断比較不可。原則無視） |

---

## カテゴリ A：TextBlock フィールドの抽出

### 抽出方法

TextBlock は同一要素 ID が **CSV 中に 1 行だけ** 存在します。
コンテキスト ID によるフィルタは不要で、`XbrlFactIndex::first_by_element` で最初の有効値を取得します。
`AsrReport` の該当章 struct と extractor にフィールドを追加します。

```rust
// asr_report.rs への追加例
pub sustainability: Option<String>,

pub governance: Option<String>,
```

### 出現頻度（サンプル 6 社での確認）

全 6 社に存在する TextBlock を「全社共通」、特定条件でのみ存在するものを「条件付き」として分類します。

**全社共通（6/6 社）：**

| フィールド名（提案） | XBRL 要素 ID |
|---|---|
| `business_description` ✅現行 | `DescriptionOfBusinessTextBlock` |
| `issues_to_address` ✅現行 | `BusinessPolicyBusinessEnvironmentIssuesToAddressEtcTextBlock` |
| `risks` ✅現行 | `BusinessRisksTextBlock` |
| `company_history` ✅現行 | `CompanyHistoryTextBlock` |
| `dividend_policy` ✅現行 | `DividendPolicyTextBlock` |
| `officers` ✅現行 | `InformationAboutOfficersTextBlock` |
| `affiliated_entities` ✅現行 | `OverviewOfAffiliatedEntitiesTextBlock` |
| `capital_expenditures` ✅現行 | `OverviewOfCapitalExpendituresEtcTextBlock` |
| `corporate_governance` ➕追加推奨 | `OverviewOfCorporateGovernanceTextBlock` |
| `officer_compensation` ➕追加推奨 | `RemunerationForDirectorsAndOtherOfficersTextBlock` |
| `major_facilities` ➕追加推奨 | `MajorFacilitiesTextBlock` |
| `facility_plans` ➕追加推奨 | `PlannedAdditionsRetirementsEtcOfFacilitiesTextBlock` |
| `research_and_development` ➕追加推奨 | `ResearchAndDevelopmentActivitiesTextBlock` |
| `human_resources_policy` ➕追加推奨 | `PolicyOnDevelopmentOfHumanResourcesAndInternalEnvironmentStrategyTextBlock` |
| `major_shareholders` ➕追加推奨 | `MajorShareholdersTextBlock` |
| `shareholding` ✅現行 | `ShareholdingByShareholderCategoryTextBlock` |

**条件付き（会計基準・企業規模による）：**

| フィールド名（提案） | XBRL 要素 ID | 条件 |
|---|---|---|
| `sustainability` ➕追加推奨 | `DisclosureOfSustainabilityRelatedFinancialInformationTextBlock` | 2023 年改正以降の書類（義務化）|
| `governance` ➕追加推奨 | `GovernanceTextBlock` | sustainability のサブセクション |
| `strategy` ➕追加推奨 | `StrategyTextBlock` | 同上 |
| `risk_management` ➕追加推奨 | `RiskManagementTextBlock` | 同上 |
| `metrics_and_targets` ➕追加推奨 | `MetricsAndTargetsTextBlock` | 同上 |
| `human_capital_metrics` ➕追加推奨 | `DescriptionOfMetricsRelatedToPolicy...MetricsAndTargetsTextBlock` | 2023 年改正以降 |
| `performance` ✅現行 | `ManagementAnalysisOfFinancialPositionOperatingResultsAndCashFlowsTextBlock` | ほぼ全社 |
| `segment_information` ✅現行 | `NotesSegmentInformationEtcConsolidatedFinancialStatementsTextBlock` | 連結のみ |
| `critical_contracts` ➕追加 | `CriticalContractsForOperationTextBlock` | 重要な契約がある場合 |

---

## カテゴリ B：経営指標サマリー 5 期分（数値）

### コンテキスト ID のパターン（実測）

会社タイプによって使用されるコンテキスト ID が異なります。

| 会社タイプ | SummaryOfBusinessResults のコンテキスト |
|---|---|
| J-GAAP 非連結（E03736, E32380） | `{Period}Duration_NonConsolidatedMember` / `{Period}Instant_NonConsolidatedMember` のみ |
| J-GAAP 連結（E05407, E00424, E05031） | suffix なし（連結値）と `_NonConsolidatedMember`（個別値）の両方 |
| IFRS 連結（E04948） | IFRS 要素 ID は suffix なし、個別財務諸表は `_NonConsolidatedMember` |

**→ 連結の場合は suffix なしを優先、なければ `_NonConsolidatedMember` にフォールバックする。**

### コンテキスト ID の組み立て

```rust
const PERIODS: &[(&str, &str)] = &[
    ("CurrentYear", "当期"),
    ("Prior1Year",  "前期"),
    ("Prior2Year",  "前々期"),
    ("Prior3Year",  "三期前"),
    ("Prior4Year",  "四期前"),
];

fn summary_contexts(period_key: &str, is_consolidated: bool) -> (Vec<String>, Vec<String>) {
    // 連結: suffix なし優先、個別フォールバック
    // 非連結: NonConsolidatedMember のみ
    let dur_ctxs = if is_consolidated {
        vec![
            format!("{}Duration", period_key),
            format!("{}Duration_NonConsolidatedMember", period_key),
        ]
    } else {
        vec![format!("{}Duration_NonConsolidatedMember", period_key)]
    };
    let ins_ctxs = if is_consolidated {
        vec![
            format!("{}Instant", period_key),
            format!("{}Instant_NonConsolidatedMember", period_key),
        ]
    } else {
        vec![format!("{}Instant_NonConsolidatedMember", period_key)]
    };
    (dur_ctxs, ins_ctxs)
}
```

### 売上高（要素 ID の選択）

**同じコンテキストで複数の要素 ID を順番に試す。**

```rust
fn revenue_element_ids(accounting_standard: &str) -> &'static [&'static str] {
    if accounting_standard.contains("IFRS") {
        &["jpcrp_cor:RevenueIFRSSummaryOfBusinessResults"]
    } else {
        &[
            "jpcrp_cor:NetSalesSummaryOfBusinessResults",           // 一般事業（最多）
            "jpcrp_cor:OperatingRevenue1SummaryOfBusinessResults",  // 金融・リース・交通等
            "jpcrp_cor:OrdinaryRevenuesSummaryOfBusinessResults",   // 銀行・保険
        ]
    }
}
```

### 当期純利益（要素 ID の選択）

```rust
fn net_income_element_ids(accounting_standard: &str, is_consolidated: bool) -> &'static [&'static str] {
    match (accounting_standard.contains("IFRS"), is_consolidated) {
        (true, _) => &[
            "jpcrp_cor:ProfitLossAttributableToOwnersOfParentIFRSSummaryOfBusinessResults",
        ],
        (false, true) => &[
            "jpcrp_cor:ProfitLossAttributableToOwnersOfParentSummaryOfBusinessResults",
            "jpcrp_cor:NetIncomeLossSummaryOfBusinessResults",  // 連結でも個別のみの場合に備えたフォールバック
        ],
        (false, false) => &[
            "jpcrp_cor:NetIncomeLossSummaryOfBusinessResults",
        ],
    }
}
```

### 全指標の要素 ID テーブル

| 指標 | J-GAAP 要素 ID（`jpcrp_cor:` 省略） | IFRS 要素 ID（`jpcrp_cor:` 省略） | 種別 |
|---|---|---|---|
| 売上高 | `NetSalesSummaryOfBusinessResults` / `OperatingRevenue1SummaryOfBusinessResults` | `RevenueIFRSSummaryOfBusinessResults` | Duration |
| 経常利益 | `OrdinaryIncomeLossSummaryOfBusinessResults` | — （IFRS は「経常利益」概念なし） | Duration |
| 当期純利益 | `ProfitLossAttributableToOwnersOfParentSummaryOfBusinessResults` / `NetIncomeLossSummaryOfBusinessResults` | `ProfitLossAttributableToOwnersOfParentIFRSSummaryOfBusinessResults` | Duration |
| 包括利益 | `ComprehensiveIncomeSummaryOfBusinessResults` | `ComprehensiveIncomeAttributableToOwnersOfParentIFRSSummaryOfBusinessResults` | Duration |
| 純資産 | `NetAssetsSummaryOfBusinessResults` | `EquityAttributableToOwnersOfParentIFRSSummaryOfBusinessResults` | Instant |
| 総資産 | `TotalAssetsSummaryOfBusinessResults` | `TotalAssetsIFRSSummaryOfBusinessResults` | Instant |
| 資本金 | `CapitalStockSummaryOfBusinessResults` | — | Instant |
| 発行済株式数 | `TotalNumberOfIssuedSharesSummaryOfBusinessResults` | — | Instant |
| 1 株純資産 | `NetAssetsPerShareSummaryOfBusinessResults` | `EquityToAssetRatioIFRSSummaryOfBusinessResults`（1株持分） | Instant |
| 1 株純利益 | `BasicEarningsLossPerShareSummaryOfBusinessResults` | `BasicEarningsLossPerShareIFRSSummaryOfBusinessResults` | Duration |
| 1 株配当 | `DividendPaidPerShareSummaryOfBusinessResults` | — | Duration |
| 自己資本比率 | `EquityToAssetRatioSummaryOfBusinessResults` | `RatioOfOwnersEquityToGrossAssetsIFRSSummaryOfBusinessResults` | Instant |
| ROE | `RateOfReturnOnEquitySummaryOfBusinessResults` | `RateOfReturnOnEquityIFRSSummaryOfBusinessResults` | Duration |
| PER | `PriceEarningsRatioSummaryOfBusinessResults` | `PriceEarningsRatioIFRSSummaryOfBusinessResults` | Duration |
| 配当性向 | `PayoutRatioSummaryOfBusinessResults` | — | Duration |
| 営業 CF | `NetCashProvidedByUsedInOperatingActivitiesSummaryOfBusinessResults` | `CashFlowsFromUsedInOperatingActivitiesIFRSSummaryOfBusinessResults` | Duration |
| 投資 CF | `NetCashProvidedByUsedInInvestingActivitiesSummaryOfBusinessResults` | `CashFlowsFromUsedInInvestingActivitiesIFRSSummaryOfBusinessResults` | Duration |
| 財務 CF | `NetCashProvidedByUsedInFinancingActivitiesSummaryOfBusinessResults` | `CashFlowsFromUsedInFinancingActivitiesIFRSSummaryOfBusinessResults` | Duration |
| 現金残高 | `CashAndCashEquivalentsSummaryOfBusinessResults` | `CashAndCashEquivalentsIFRSSummaryOfBusinessResults` | Instant |
| 株主総利回り | `TotalShareholderReturn` | — | Duration |
| 従業員数 | `NumberOfEmployees` | — | Instant（`_NonConsolidatedMember` を使用） |

### 経営指標サマリー構築の全体コード

```rust
fn build_business_results_summary(
    index: &XbrlFactIndex,
    dei: &DeiInfo,
) -> Vec<BusinessResultsPeriod> {
    PERIODS.iter().map(|(period_key, label)| {
        let (dur_ctxs, ins_ctxs) = summary_contexts(period_key, dei.is_consolidated);
        let dur: Vec<&str> = dur_ctxs.iter().map(|s| s.as_str()).collect();
        let ins: Vec<&str> = ins_ctxs.iter().map(|s| s.as_str()).collect();

        let revenue = revenue_element_ids(&dei.accounting_standard)
            .iter()
            .find_map(|id| get_first(index, id, &dur))
            .and_then(|s| parse_i64(s));

        let net_income = net_income_element_ids(&dei.accounting_standard, dei.is_consolidated)
            .iter()
            .find_map(|id| get_first(index, id, &dur))
            .and_then(|s| parse_i64(s));

        BusinessResultsPeriod {
            period: period_key.to_string(),
            label: label.to_string(),
            revenue,
            ordinary_income:    get_first(index, "jpcrp_cor:OrdinaryIncomeLossSummaryOfBusinessResults", &dur).and_then(|s| parse_i64(s)),
            net_income,
            net_assets:         get_first(index, "jpcrp_cor:NetAssetsSummaryOfBusinessResults", &ins).and_then(|s| parse_i64(s)),
            total_assets:       get_first(index, "jpcrp_cor:TotalAssetsSummaryOfBusinessResults", &ins).and_then(|s| parse_i64(s)),
            equity_ratio:       get_first(index, "jpcrp_cor:EquityToAssetRatioSummaryOfBusinessResults", &ins).and_then(|s| parse_numeric(s)),
            roe:                get_first(index, "jpcrp_cor:RateOfReturnOnEquitySummaryOfBusinessResults", &dur).and_then(|s| parse_numeric(s)),
            per:                get_first(index, "jpcrp_cor:PriceEarningsRatioSummaryOfBusinessResults", &dur).and_then(|s| parse_numeric(s)),
            payout_ratio:       get_first(index, "jpcrp_cor:PayoutRatioSummaryOfBusinessResults", &dur).and_then(|s| parse_numeric(s)),
            eps:                get_first(index, "jpcrp_cor:BasicEarningsLossPerShareSummaryOfBusinessResults", &dur).and_then(|s| parse_numeric(s)),
            bps:                get_first(index, "jpcrp_cor:NetAssetsPerShareSummaryOfBusinessResults", &ins).and_then(|s| parse_numeric(s)),
            dps:                get_first(index, "jpcrp_cor:DividendPaidPerShareSummaryOfBusinessResults", &dur).and_then(|s| parse_numeric(s)),
            operating_cf:       get_first(index, "jpcrp_cor:NetCashProvidedByUsedInOperatingActivitiesSummaryOfBusinessResults", &dur).and_then(|s| parse_i64(s)),
            investing_cf:       get_first(index, "jpcrp_cor:NetCashProvidedByUsedInInvestingActivitiesSummaryOfBusinessResults", &dur).and_then(|s| parse_i64(s)),
            financing_cf:       get_first(index, "jpcrp_cor:NetCashProvidedByUsedInFinancingActivitiesSummaryOfBusinessResults", &dur).and_then(|s| parse_i64(s)),
            cash_and_equivalents: get_first(index, "jpcrp_cor:CashAndCashEquivalentsSummaryOfBusinessResults", &ins).and_then(|s| parse_i64(s)),
            tsr:                get_first(index, "jpcrp_cor:TotalShareholderReturn", &dur).and_then(|s| parse_numeric(s)),
        }
    }).collect()
}
```

---

## カテゴリ D：人的資本・ダイバーシティ数値

### ⚠️ このカテゴリは最も複雑

サンプル 6 社の調査で、**要素 ID のサフィックス** と **コンテキスト ID** の両方が会社タイプで変動することが判明しました。

### 要素 ID サフィックスの 2 パターン

| サフィックス | 使用される状況 | 実測会社 |
|---|---|---|
| `MetricsOfReportingCompany` | 単独会社または自社開示（提出会社単体の数値） | E05407（連結）, E32380（非連結）|
| `MetricsOfConsolidatedSubsidiaries` | 連結子会社ごとに行を分けて開示 | E04948（IFRS 連結）, E00424（J-GAAP 連結）|

両パターンが同一の論理フィールド（例：「管理職女性比率」）を表すため、**両方の要素 ID を試す**必要があります。

### コンテキスト ID の 2 パターン

| コンテキスト | 使用される状況 |
|---|---|
| `CurrentYearInstant_NonConsolidatedMember` | `MetricsOfReportingCompany` を使う会社（提出会社の単一値）|
| `CurrentYearInstant_Row1Member`, `Row2Member`, ... | `MetricsOfConsolidatedSubsidiaries` を使う会社（子会社ごとに 1 行ずつ）|

`RowNMember` パターンは**複数行ある子会社リストの先頭行（Row1）が提出会社自身**である場合が多いですが、保証はありません。
ヒューリスティックとして「最初の有効値（Row1）」を採用します。

### 育休取得率の要素 ID は 3 種類

法律の引用方法の違いで要素 ID が異なります。実際に使われている要素 ID：

```
// 短縮名（ConsolidatedSubsidiaries 系）
jpcrp_cor:AllEmployeesRatioOfMaleEmployeesTakingChildcareLeaveMetricsOfConsolidatedSubsidiaries

// 育児・介護休業法を明示した長名（ReportingCompany 系）
jpcrp_cor:AllEmployeesCalculatedBasedOnProvisionsOfArticle714Item1OfOrdinanceForEnforcementOfActOnChildcareLeaveCaregiverLeaveAndOtherMeasuresForTheWelfareOfWorkersCaringForChildrenOrOtherFamilyMembersRatioOfMaleEmployeesTakingChildcareLeaveMetricsOfReportingCompany

// 女性活躍推進法を明示した長名（ReportingCompany 系）
jpcrp_cor:RegularEmployeesCalculatedBasedOnProvisionsOfActOnPromotionOfWomensActiveEngagementInProfessionalLifeRatioOfMaleEmployeesTakingChildcareLeaveMetricsOfReportingCompany
```

### 各フィールドの抽出ルール

```rust
fn extract_human_capital(
    index: &XbrlFactIndex,
) -> HumanCapitalMetrics {
    // 提出会社単体: NonConsolidatedMember
    let nc_instant = "CurrentYearInstant_NonConsolidatedMember";

    // 連結子会社リスト先頭: Row1Member
    let row1_instant = "CurrentYearInstant_Row1Member";

    // 両パターンを試す汎用取得
    let get_hc = |ids: &[&str]| -> Option<f64> {
        ids.iter().find_map(|id| {
            // 1. ReportingCompany + NonConsolidatedMember
            // 2. ConsolidatedSubsidiaries + Row1Member
            index.get(&(id.to_string(), nc_instant.to_string()))
                .or_else(|| index.get(&(id.to_string(), row1_instant.to_string())))
                .and_then(|s| parse_numeric(s))
        })
    };

    HumanCapitalMetrics {
        // 平均年間給与（提出会社）
        avg_annual_salary: get_first(index,
            "jpcrp_cor:AverageAnnualSalaryInformationAboutReportingCompanyInformationAboutEmployees",
            &[nc_instant]).and_then(|s| parse_i64(s)),

        // 平均年齢・勤続年数
        avg_age_years:     get_first(index,
            "jpcrp_cor:AverageAgeYearsInformationAboutReportingCompanyInformationAboutEmployees",
            &[nc_instant]).and_then(|s| parse_numeric(s)),
        avg_service_years: get_first(index,
            "jpcrp_cor:AverageLengthOfServiceYearsInformationAboutReportingCompanyInformationAboutEmployees",
            &[nc_instant]).and_then(|s| parse_numeric(s)),

        // 管理職女性比率（両サフィックスを試す）
        ratio_female_managers: get_hc(&[
            "jpcrp_cor:RatioOfFemaleEmployeesInManagerialPositionsMetricsOfReportingCompany",
            "jpcrp_cor:RatioOfFemaleEmployeesInManagerialPositionsMetricsOfConsolidatedSubsidiaries",
        ]),

        // 男性育休取得率（3 種の要素 ID を試す）
        ratio_male_childcare_leave: get_hc(&[
            "jpcrp_cor:AllEmployeesRatioOfMaleEmployeesTakingChildcareLeaveMetricsOfConsolidatedSubsidiaries",
            "jpcrp_cor:AllEmployeesCalculatedBasedOnProvisionsOfArticle714Item1OfOrdinanceForEnforcementOfActOnChildcareLeaveCaregiverLeaveAndOtherMeasuresForTheWelfareOfWorkersCaringForChildrenOrOtherFamilyMembersRatioOfMaleEmployeesTakingChildcareLeaveMetricsOfReportingCompany",
            "jpcrp_cor:RegularEmployeesCalculatedBasedOnProvisionsOfActOnPromotionOfWomensActiveEngagementInProfessionalLifeRatioOfMaleEmployeesTakingChildcareLeaveMetricsOfReportingCompany",
        ]),

        // 男女賃金格差（全労働者）
        gender_pay_gap_all: get_hc(&[
            "jpcrp_cor:AllEmployeesDifferencesInWagesBetweenMaleAndFemaleEmployeesMetricsOfReportingCompany",
            "jpcrp_cor:AllEmployeesDifferencesInWagesBetweenMaleAndFemaleEmployeesMetricsOfConsolidatedSubsidiaries",
        ]),

        // 男女賃金格差（正規）
        gender_pay_gap_regular: get_hc(&[
            "jpcrp_cor:RegularEmployeesDifferencesInWagesBetweenMaleAndFemaleEmployeesMetricsOfReportingCompany",
            "jpcrp_cor:RegularEmployeesDifferencesInWagesBetweenMaleAndFemaleEmployeesMetricsOfConsolidatedSubsidiaries",
        ]),

        // 男女賃金格差（非正規）
        gender_pay_gap_non_regular: get_hc(&[
            "jpcrp_cor:NonRegularEmployeesDifferencesInWagesBetweenMaleAndFemaleEmployeesMetricsOfReportingCompany",
            "jpcrp_cor:NonRegularEmployeesDifferencesInWagesBetweenMaleAndFemaleEmployeesMetricsOfConsolidatedSubsidiaries",
        ]),
    }
}
```

---

## カテゴリ E：ガバナンス数値

### 役員数・女性役員比率

コンテキストは `FilingDateInstant`（提出日時点）の固定です。

```rust
fn extract_governance_officers(
    index: &XbrlFactIndex,
) -> OfficerInfo {
    let ctx = "FilingDateInstant";
    OfficerInfo {
        num_male_directors:   get_first(index, "jpcrp_cor:NumberOfMaleDirectorsAndOtherOfficers", &[ctx]).and_then(|s| parse_i64(s)),
        num_female_directors: get_first(index, "jpcrp_cor:NumberOfFemaleDirectorsAndOtherOfficers", &[ctx]).and_then(|s| parse_i64(s)),
        ratio_female_directors: get_first(index, "jpcrp_cor:RatioOfFemaleDirectorsAndOtherOfficers", &[ctx]).and_then(|s| parse_numeric(s)),
    }
}
```

### 監査報酬

コンテキストは `CurrentYearDuration`（メンバーサフィックスなし）。
当期と前期の両方が存在するため、当期（`CurrentYearDuration`）を取得します。

```rust
fn extract_audit_fees(index: &XbrlFactIndex) -> AuditFees {
    let ctx = &["CurrentYearDuration"];
    AuditFees {
        // 提出会社の監査証明業務報酬
        audit_fee_reporting_company: get_first(index, "jpcrp_cor:AuditFeesReportingCompany", ctx).and_then(|s| parse_i64(s)),
        // 連結子会社の監査証明業務報酬
        audit_fee_subsidiaries:      get_first(index, "jpcrp_cor:AuditFeesConsolidatedSubsidiaries", ctx).and_then(|s| parse_i64(s)),
        // 合計
        audit_fee_total:             get_first(index, "jpcrp_cor:AuditFeesTotal", ctx).and_then(|s| parse_i64(s)),
    }
}
```

### 役員報酬総額

役員区分（取締役/社外取締役/監査役等）ごとに別のコンテキスト行が存在します。
「全体の合計」を一発で取得できる単一コンテキストはなく、**区分ごとの値を個別に取得する**設計が現実的です。

実測したコンテキスト ID のパターン：

```
// 監査等委員会設置会社（E04948/光通信）
CurrentYearDuration_DirectorsExcludingAuditAndSupervisoryCommitteeMembersAndOutsideDirectorsMember
CurrentYearDuration_DirectorsAppointedAsAuditAndSupervisoryCommitteeMembersExcludingOutsideDirectorsMember
CurrentYearDuration_OutsideDirectorsAndOtherOfficersMember

// 監査役会設置会社（E05407/青山財産NW）
CurrentYearDuration_DirectorsExcludingOutsideDirectorsMember
CurrentYearDuration_CorporateAuditorsExcludingOutsideCorporateAuditorsMember
CurrentYearDuration_OutsideDirectorsAndOtherOfficersMember
```

```rust
// 全区分の報酬を集約する場合
fn total_officer_compensation(index: &XbrlFactIndex) -> Option<i64> {
    // コンテキスト ID に "CurrentYearDuration_" で始まり "Member" で終わる、
    // かつ "Directors" or "Auditors" or "Officers" を含む行を全部合算する
    let element_id = "jpcrp_cor:TotalAmountOfRemunerationEtcRemunerationEtcByCategoryOfDirectorsAndOtherOfficers";
    let total: i64 = index
        .facts_by_element(element_id)
        .filter(|fact| {
            fact.context_id.starts_with("CurrentYearDuration_")
                && fact.context_id.ends_with("Member")
        })
        .filter_map(|fact| fact.parse_i64())
        .sum();
    if total > 0 { Some(total) } else { None }
}
```

---

## 全体的な抽出フロー

```
入力: CSV 全行

1. 全行を `XbrlCsvRecord` にデシリアライズ
2. `XbrlFact` に変換し、`XbrlFactIndex` を構築

3. DEI フェーズ（FilingDateInstant コンテキスト）
   → accounting_standard / is_consolidated を確定

4. TextBlock 抽出（カテゴリ A）
   → `first_by_element` で最初の有効値を取得（コンテキスト不問）

5. 経営指標サマリー構築（カテゴリ B）
   → summary_contexts() でコンテキストリスト生成
   → 売上高・純利益は会計基準で要素 ID を分岐
   → 5 期分ループ

6. 人的資本指標抽出（カテゴリ D）
   → MetricsOfReportingCompany + NonConsolidatedMember を試す
   → なければ MetricsOfConsolidatedSubsidiaries + Row1Member を試す

7. ガバナンス数値抽出（カテゴリ E）
   → 役員数: FilingDateInstant
   → 監査報酬: CurrentYearDuration
   → 役員報酬: CurrentYearDuration_*Member を全件合算
```

---

## エラーケースと対処方針

| ケース | 対処 |
|---|---|
| 要素 ID が存在しない | `Option<T>` で `None`、出力は `null` |
| 値が `"－"`（全角ダッシュ） | `parse_numeric` / `parse_i64` で `None` |
| 値に `"※1"` 等の注釈 | パース失敗 → `None` |
| TextBlock が複数行（稀） | 先頭行を採用（現行動作を維持） |
| 人的資本指標が Row1〜RowN の複数行 | Row1 のみ採用（提出会社自体の値が先頭に来ることが多い） |
| 役員報酬のコンテキストが会社機関設計で変わる | 全 `CurrentYearDuration_*Member` を合算 |
| 企業固有拡張要素（`jpcrp030000-asr_E…`）が混入 | コンテキスト・要素 ID とも横断比較不可のため無視 |

---

## 動作確認の観点（サンプル 6 社）

| 会社 | EDINET | 基準 | 連結 | 確認ポイント |
|---|---|---|---|---|
| 日産FS | E03736 | J-GAAP | false | コンテキストは `_NonConsolidatedMember` のみ |
| ストライク | E32380 | J-GAAP | false | 同上、人的資本は `MetricsOfReportingCompany` |
| 青山財産NW | E05407 | J-GAAP | true | suffix なし優先、人的資本は `MetricsOfReportingCompany`、役員報酬は監査役会型コンテキスト |
| ダイドーGHD | E00424 | J-GAAP | true | 売上は `NetSales`、人的資本は `MetricsOfConsolidatedSubsidiaries + Row1Member` |
| トヨタFS | E05031 | J-GAAP | true | 売上は `OperatingRevenue1` |
| 光通信 | E04948 | IFRS | true | IFRS 要素 ID、役員報酬は監査等委員会型コンテキスト、人的資本は `MetricsOfConsolidatedSubsidiaries + Row1Member` |

---

## 参考

- `docs/ESE140206.pdf` — EDINET API 仕様書（書類取得 API の `type=5` が CSV）
- `docs/ESE140133.pdf` — EDINET 書類閲覧操作ガイド
- `docs/xbrl_extraction_improvement.md` — 抽出項目の改善計画（追加すべきフィールド一覧）
- [EDINET タクソノミ](https://disclosure2dl.edinet-fsa.go.jp/) — 要素 ID の完全定義
