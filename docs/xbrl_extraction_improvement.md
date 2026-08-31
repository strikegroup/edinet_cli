# XBRL 抽出項目の改善計画

## 概要

現在の `getter/asr_report.rs` は、有価証券報告書 XBRL CSV から章立てベースの `AsrReport` を組み立てます。
TextBlock と、「主要な経営指標等の推移」に含まれる一部の数値項目を抽出しています。
EDINET が提供する XBRL の CSV には、財務指標・財務諸表・人的資本指標など、構造化された数値データが豊富に含まれています。

本ドキュメントでは、追加で抽出すべき項目とその実装方針を整理します。

現在の抽出基盤は `XbrlFact` / `XbrlFactIndex` を中心に再構成されています。実装構造の詳細は
[`xbrl_fact_extraction_refactor_plan.md`](./xbrl_fact_extraction_refactor_plan.md) を参照してください。

---

## 現在の抽出項目

TextBlock は `XbrlFactIndex::first_by_element` で取得し、数値項目は要素 ID とコンテキスト ID の組み合わせで取得します。

| `AsrReport` フィールド | XBRL 要素 ID | 内容 |
|---|---|---|
| `company_overview.company_history` | `CompanyHistoryTextBlock` | 沿革 |
| `company_overview.employees` | `InformationAboutEmployeesTextBlock` | 従業員の状況 |
| `company_overview.business_results_summary` | 複数の `*SummaryOfBusinessResults` 要素 | 主要な経営指標等の推移 |
| `business_overview.business_description` | `DescriptionOfBusinessTextBlock` | 事業の内容 |
| `business_overview.performance` | `ManagementAnalysisOfFinancialPositionOperatingResultsAndCashFlowsTextBlock` | 経営者による財政状態・経営成績・CF の分析 |
| `business_overview.issues_to_address` | `BusinessPolicyBusinessEnvironmentIssuesToAddressEtcTextBlock` | 経営方針・経営環境・対処すべき課題等 |
| `business_overview.risks` | `BusinessRisksTextBlock` | 事業等のリスク |
| `business_overview.sustainability` | `DisclosureOfSustainabilityRelatedFinancialInformationTextBlock` | サステナビリティに関する考え方及び取組 |
| `business_overview.research_and_development` | `ResearchAndDevelopmentActivitiesTextBlock` | 研究開発活動 |
| `business_overview.critical_contracts` | `CriticalContractsForOperationTextBlock` | 経営上の重要な契約等 |
| `facilities.capital_expenditures` | `OverviewOfCapitalExpendituresEtcTextBlock` | 設備投資等の概要 |
| `facilities.major_facilities` | `MajorFacilitiesTextBlock` | 主要な設備の状況 |
| `facilities.facility_plans` | `PlannedAdditionsRetirementsEtcOfFacilitiesTextBlock` | 設備の新設・除却等の計画 |
| `corporate_information.shareholding` | `ShareholdingByShareholderCategoryTextBlock` | 所有者別状況 |
| `corporate_information.major_shareholders` | `MajorShareholdersTextBlock` | 大株主の状況 |
| `corporate_information.dividend_policy` | `DividendPolicyTextBlock` | 配当政策 |
| `corporate_information.officers` | `InformationAboutOfficersTextBlock` | 役員の状況 |
| `corporate_information.corporate_governance` | `OverviewOfCorporateGovernanceTextBlock` | コーポレートガバナンスの概要 |
| `corporate_information.officer_compensation` | `RemunerationForDirectorsAndOtherOfficersTextBlock` | 役員の報酬等 |
| `financial_information.segment_information` | `NotesSegmentInformationEtcConsolidatedFinancialStatementsTextBlock` | セグメント情報等（連結） |

---

## 技術的な前提知識

### XBRL CSV の構造

CSV は UTF-16LE の TSV 形式で、列は次の通りです。

```
要素ID  項目名  コンテキストID  相対年度  連結・個別  期間・時点  ユニットID  単位  値
```

**重要な点：同一の要素 ID が複数行に渡って存在する。**
どの行が欲しいデータかは「コンテキスト ID」で判別します。

### コンテキスト ID の命名規則

| コンテキスト ID パターン | 意味 |
|---|---|
| `CurrentYearDuration_NonConsolidatedMember` | 当期（個別） |
| `Prior1YearDuration_NonConsolidatedMember` | 前期（個別） |
| `Prior2YearDuration_NonConsolidatedMember` | 前々期（個別） |
| `Prior3YearDuration_NonConsolidatedMember` | 三期前（個別） |
| `Prior4YearDuration_NonConsolidatedMember` | 四期前（個別） |
| `CurrentYearInstant_NonConsolidatedMember` | 当期末（個別・時点） |
| `Prior1YearInstant_NonConsolidatedMember` | 前期末（個別・時点） |
| `CurrentYearDuration` | 当期（連結 or 単独会社用） |
| `FilingDateInstant` | 提出日時点（DEI 情報等） |

連結財務諸表を作成している企業では `ConsolidatedMember` が使われる場合があります。個別のみの企業（特定金融業等）では `NonConsolidatedMember` のみです。

### 実装上の注意

現在の抽出基盤は `XbrlFactIndex` に全 fact を保持し、TextBlock は要素 ID、数値データは要素 ID とコンテキスト ID で取得します。

- **TextBlock**：各 TextBlock 要素は通常 CSV 中に 1 行だけ現れるため、要素 ID の最初の有効値を使います。
- **数値データ**：同一要素 ID が複数期分・複数コンテキストで複数行存在するため、用途ごとにコンテキスト ID の優先順位を定義します。

---

## 追加項目の提案

### カテゴリ A：テキスト追加候補

2023 年の開示府令改正でサステナビリティ・人的資本の開示が義務化されました。
現在未抽出の追加候補は TextBlock として取得でき、章別 extractor に追加できます。

| 追加フィールド名（提案） | XBRL 要素 ID | 内容 | 備考 |
|---|---|---|---|
| `governance` | `GovernanceTextBlock` | ガバナンス（TCFD 開示含む） | sustainability の下位セクション |
| `strategy` | `StrategyTextBlock` | 戦略 | sustainability の下位セクション |
| `risk_management` | `RiskManagementTextBlock` | リスク管理 | sustainability の下位セクション |
| `metrics_and_targets` | `MetricsAndTargetsTextBlock` | 指標及び目標（CO₂・人的資本 KPI 等） | sustainability の下位セクション |
| `human_resources_policy` | `PolicyOnDevelopmentOfHumanResourcesAndInternalEnvironmentStrategyTextBlock` | 人材育成・社内環境整備方針 | 2023年改正で義務化 |
| `human_capital_metrics_description` | `DescriptionOfMetricsRelatedToPolicyOnDevelopmentOfHumanResourcesAndInternalEnvironmentAndTargetsAndPerformanceUsingSuchMetricsMetricsAndTargetsTextBlock` | 人的資本指標の内容・目標・実績 | 2023年改正で義務化 |

**実装方法：** `AsrReport` の該当章 struct に `Option<String>` フィールドを追加し、章別 extractor で `XbrlFactIndex::first_by_element` から値を取得します。

---

### カテゴリ B：経営指標サマリー 5 期分数値（拡張）

有価証券報告書の「主要な経営指標等の推移」に対応します。
過去 5 期分の時系列データが格納されており、財務トレンド分析の基礎になります。

**データ取得戦略：**
`Prior4YearDuration` 〜 `CurrentYearDuration` の 5 期について、各期ごとに **連結 → 個別 → メンバーなし** の順でコンテキスト候補を探索し、配列として保持します。
現行実装は `CurrentYearDuration` を先に見る箇所がありますが、カテゴリ B に着手する段階でこの優先順位へ統一します。

**提案する出力形式：**

```json
"business_results_summary": [
  {
    "period": "CurrentYear",
    "label": "当期",
    "operating_revenue": 53080000000,
    "ordinary_income": 31669000000,
    "net_income": 22018000000,
    "net_assets": 132537000000,
    "total_assets": 1130436000000,
    "equity_ratio": 0.117,
    "roe": 0.181,
    "per": null,
    "payout_ratio": null,
    "operating_cash_flow": null,
    "investing_cash_flow": null,
    "financing_cash_flow": null,
    "cash_and_equivalents": null,
    "net_assets_per_share": 5725.64,
    "earnings_per_share": 951.18,
    "dividend_per_share": 951.00,
    "total_shareholder_return": null,
    "employees": 1234
  },
  ...
]
```

**取得対象の要素 ID 一覧：**

| 項目 | XBRL 要素 ID（`jpcrp_cor:` プレフィックス） | 単位 |
|---|---|---|
| 営業収益 | `OperatingRevenue1SummaryOfBusinessResults` | 円 |
| 経常利益/損失 | `OrdinaryIncomeLossSummaryOfBusinessResults` | 円 |
| 当期純利益/損失 | `NetIncomeLossSummaryOfBusinessResults` | 円 |
| 純資産額 | `NetAssetsSummaryOfBusinessResults` | 円 |
| 総資産額 | `TotalAssetsSummaryOfBusinessResults` | 円 |
| 資本金 | `CapitalStockSummaryOfBusinessResults` | 円 |
| 発行済株式総数 | `TotalNumberOfIssuedSharesSummaryOfBusinessResults` | 株 |
| 1 株当たり純資産額 | `NetAssetsPerShareSummaryOfBusinessResults` | 円/株 |
| 1 株当たり当期純利益/損失 | `BasicEarningsLossPerShareSummaryOfBusinessResults` | 円/株 |
| 1 株当たり配当額 | `DividendPaidPerShareSummaryOfBusinessResults` | 円/株 |
| 自己資本比率 | `EquityToAssetRatioSummaryOfBusinessResults` | 小数（0.117 = 11.7%） |
| ROE（自己資本利益率） | `RateOfReturnOnEquitySummaryOfBusinessResults` | 小数 |
| PER（株価収益率） | `PriceEarningsRatioSummaryOfBusinessResults` | 倍 |
| 配当性向 | `PayoutRatioSummaryOfBusinessResults` | 小数 |
| 営業活動による CF | `NetCashProvidedByUsedInOperatingActivitiesSummaryOfBusinessResults` | 円 |
| 投資活動による CF | `NetCashProvidedByUsedInInvestingActivitiesSummaryOfBusinessResults` | 円 |
| 財務活動による CF | `NetCashProvidedByUsedInFinancingActivitiesSummaryOfBusinessResults` | 円 |
| 現金及び現金同等物の残高 | `CashAndCashEquivalentsSummaryOfBusinessResults` | 円 |
| 株主総利回り | `TotalShareholderReturn` | 小数 |
| 従業員数 | `NumberOfEmployees` | 人 |

**実装方針：**
フェーズ 2 では、まず現行の `XbrlFactIndex::first_by_contexts` を使う方式に寄せて実装します。
すなわち「要素 ID ごとに、各期の優先コンテキスト候補を順番に試し、最初に見つかった有効値を採用する」方式を共通 helper 化します。
全コンテキストを一括収集してから Period 別に集約する高度化は、必要になった段階で別途検討します。

```rust
struct PeriodContext {
    period: &'static str,
    label: &'static str,
    contexts: &'static [&'static str],
}

const PERIOD_CONTEXTS: &[PeriodContext] = &[
    PeriodContext {
        period: "CurrentYear",
        label: "当期",
        contexts: &[
            "CurrentYearDuration_ConsolidatedMember",
            "CurrentYearDuration_NonConsolidatedMember",
            "CurrentYearDuration",
        ],
    },
    PeriodContext {
        period: "Prior1Year",
        label: "前期",
        contexts: &[
            "Prior1YearDuration_ConsolidatedMember",
            "Prior1YearDuration_NonConsolidatedMember",
            "Prior1YearDuration",
        ],
    },
    // ...
];

fn i64_value(index: &XbrlFactIndex, element_id: &str, contexts: &[&str]) -> Option<i64> {
    index.first_by_contexts(element_id, contexts)?.parse_i64()
}
```

---

### カテゴリ C：財務諸表 当期・前期値（新規）

`jppfs_cor:` プレフィックスの要素から、貸借対照表・損益計算書の主要科目を取得します。
カテゴリ B のサマリーと重複する項目もありますが、こちらは財務諸表本体から直接取得した値です。

**コンテキスト ID：**
- 時点（B/S）: 各期とも `suffix なし → NonConsolidatedMember` の順で探索
  - 当期: `CurrentYearInstant` → `CurrentYearInstant_NonConsolidatedMember`
  - 前期: `Prior1YearInstant` → `Prior1YearInstant_NonConsolidatedMember`
- 期間（P/L・CF）: 各期とも `suffix なし → NonConsolidatedMember` の順で探索
  - 当期: `CurrentYearDuration` → `CurrentYearDuration_NonConsolidatedMember`
  - 前期: `Prior1YearDuration` → `Prior1YearDuration_NonConsolidatedMember`

**営業収益の取得方針：**
`OperatingRevenue1` のみを前提にせず、一般事業会社で多い `NetSales` を先頭に、`OperatingRevenue1`、`OrdinaryRevenues` の順でフォールバックします。
これにより、業種差分で `null` が増えすぎるのを避けます。

**貸借対照表（B/S）主要科目：**

| 項目 | XBRL 要素 ID（`jppfs_cor:` プレフィックス） |
|---|---|
| 総資産 | `Assets` |
| 流動資産 | `CurrentAssets` |
| 固定資産 | `NoncurrentAssets` |
| 流動負債 | `CurrentLiabilities` |
| 固定負債 | `NoncurrentLiabilities` |
| 純資産 | `NetAssets` |
| 資本金 | `CapitalStock` |
| 資本剰余金 | `CapitalSurplus` |
| 利益剰余金 | `RetainedEarnings` |

**損益計算書（P/L）主要科目：**

| 項目 | XBRL 要素 ID（`jppfs_cor:` プレフィックス） |
|---|---|
| 営業収益 | `NetSales` / `OperatingRevenue1` / `OrdinaryRevenues` |
| 営業費用 | `OperatingExpenses` |
| 営業利益/損失 | `OperatingIncome` |
| 経常利益/損失 | `OrdinaryIncome` |
| 税引前当期純利益/損失 | `IncomeBeforeIncomeTaxes` |
| 当期純利益/損失 | `ProfitLoss` |

---

### カテゴリ D：人的資本・ダイバーシティ数値（新規）

2023 年の開示府令改正で義務化された数値指標です。ESG 評価や投資判断への活用を想定します。

**出力配置方針：**
人的資本数値は `CompanyOverview` に混ぜず、`BusinessOverview` 配下の新しい struct にまとめます。
サステナビリティ開示・人材戦略と同じ章に属する情報として扱い、TextBlock 追加（カテゴリ A）との整合を取りやすくします。

| 追加フィールド名（提案） | XBRL 要素 ID（`jpcrp_cor:` プレフィックス） | 単位 | 備考 |
|---|---|---|---|
| `employees_count` | `NumberOfEmployees` | 人 | 正規従業員数（コンテキスト別） |
| `average_annual_salary` | `AverageAnnualSalaryInformationAboutReportingCompanyInformationAboutEmployees` | 円 | |
| `average_age_years` | `AverageAgeYearsInformationAboutReportingCompanyInformationAboutEmployees` | 年 | |
| `average_service_years` | `AverageLengthOfServiceYearsInformationAboutReportingCompanyInformationAboutEmployees` | 年 | |
| `ratio_female_managers` | `RatioOfFemaleEmployeesInManagerialPositionsMetricsOfConsolidatedSubsidiaries` | 小数 | 義務開示 |
| `ratio_male_childcare_leave` | `AllEmployeesRatioOfMaleEmployeesTakingChildcareLeaveMetricsOfConsolidatedSubsidiaries` | 小数 | 義務開示 |
| `gender_pay_gap_all` | `AllEmployeesDifferencesInWagesBetweenMaleAndFemaleEmployeesMetricsOfConsolidatedSubsidiaries` | 小数 | 義務開示 |
| `gender_pay_gap_regular` | `RegularEmployeesDifferencesInWagesBetweenMaleAndFemaleEmployeesMetricsOfConsolidatedSubsidiaries` | 小数 | |
| `gender_pay_gap_non_regular` | `NonRegularEmployeesDifferencesInWagesBetweenMaleAndFemaleEmployeesMetricsOfConsolidatedSubsidiaries` | 小数 | |

```rust
pub struct BusinessOverview {
    pub business_description: Option<String>,
    pub performance: Option<String>,
    pub issues_to_address: Option<String>,
    pub risks: Option<String>,
    pub sustainability: Option<String>,
    pub governance: Option<String>,
    pub strategy: Option<String>,
    pub risk_management: Option<String>,
    pub metrics_and_targets: Option<String>,
    pub human_resources_policy: Option<String>,
    pub human_capital_metrics_description: Option<String>,
    pub human_capital_metrics: HumanCapitalMetrics,
    pub research_and_development: Option<String>,
    pub critical_contracts: Option<String>,
}

pub struct HumanCapitalMetrics {
    pub employees_count: Option<i64>,
    pub average_annual_salary: Option<i64>,
    pub average_age_years: Option<f64>,
    pub average_service_years: Option<f64>,
    pub ratio_female_managers: Option<f64>,
    pub ratio_male_childcare_leave: Option<f64>,
    pub gender_pay_gap_all: Option<f64>,
    pub gender_pay_gap_regular: Option<f64>,
    pub gender_pay_gap_non_regular: Option<f64>,
}
```

---

### カテゴリ E：ガバナンス数値（新規）

**出力配置方針：**
ガバナンス数値は `CorporateInformation` 配下の新しい struct にまとめます。
既存の `officers` / `corporate_governance` / `officer_compensation` TextBlock と同じ章に寄せ、フェーズ 3 で平置きフィールドが増えすぎるのを防ぎます。

| 追加フィールド名（提案） | XBRL 要素 ID（`jpcrp_cor:` プレフィックス） | 単位 |
|---|---|---|
| `total_officer_compensation` | `TotalAmountOfRemunerationEtcRemunerationEtcByCategoryOfDirectorsAndOtherOfficers` | 円 |
| `audit_fee` | `AuditFeesReportingCompany` | 円 |
| `ratio_female_directors` | `RatioOfFemaleDirectorsAndOtherOfficers` | 小数 |
| `num_male_directors` | `NumberOfMaleDirectorsAndOtherOfficers` | 人 |
| `num_female_directors` | `NumberOfFemaleDirectorsAndOtherOfficers` | 人 |

```rust
pub struct CorporateInformation {
    pub shareholding: Option<String>,
    pub major_shareholders: Option<String>,
    pub dividend_policy: Option<String>,
    pub officers: Option<String>,
    pub corporate_governance: Option<String>,
    pub officer_compensation: Option<String>,
    pub governance_metrics: GovernanceMetrics,
}

pub struct GovernanceMetrics {
    pub ratio_female_directors: Option<f64>,
    pub total_officer_compensation: Option<i64>,
    pub audit_fee: Option<i64>,
    pub num_male_directors: Option<i64>,
    pub num_female_directors: Option<i64>,
}
```

---

## 優先順位と実装ロードマップ

### フェーズ 1：テキスト追加（低コスト・高価値）

**対象：** カテゴリ A 全項目  
**工数目安：** 小（章別 struct と extractor への追加）  
**理由：** `XbrlFactIndex::first_by_element` で取得できる TextBlock のため変更箇所が小さい。サステナビリティ開示（2023 年義務化）の取得が最優先。

```rust
// 追加例（asr_report.rs）
pub governance: Option<String>,

pub strategy: Option<String>,
// ...
```

---

### フェーズ 2：経営指標サマリー数値（中コスト・高価値）

**対象：** カテゴリ B  
**工数目安：** 中（出力フィールドと章別 extractor の追加が必要）

現在の XBRL fact 抽出基盤に、未抽出の要素 ID と出力フィールドを追加します。

1. `BusinessResultsPeriod` に追加する数値フィールドを定義する。
2. 各期のコンテキスト候補を **連結 → 個別 → メンバーなし** の順で定義する。
3. `company_overview` extractor に対象要素 ID を追加する。
4. `XbrlFactIndex::first_by_contexts` で優先順位に沿って値を取得する。

```rust
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct BusinessResultsPeriod {
    pub period: String,              // "CurrentYear", "Prior1Year", ...
    pub label: String,               // "当期", "前期", ...
    pub operating_revenue: Option<i64>,
    pub ordinary_income: Option<i64>,
    pub net_income: Option<i64>,
    pub net_assets: Option<i64>,
    pub total_assets: Option<i64>,
    pub equity_ratio: Option<f64>,
    pub roe: Option<f64>,
    // ...
}
```

---

### フェーズ 3：人的資本・ガバナンス数値（低コスト）

**対象：** カテゴリ D・E  
**工数目安：** 小〜中（フェーズ 2 の基盤を使いまわす）  
**前提：** 現在の `XbrlFactIndex` を使って、対象ごとのコンテキスト優先順位を定義する。  
**出力方針：**

1. 人的資本数値は `BusinessOverview.human_capital_metrics` に集約する。
2. ガバナンス数値は `CorporateInformation.governance_metrics` に集約する。
3. 既存の章 struct へ平置きフィールドを増やしすぎず、TextBlock と数値を章単位で整理する。

---

### フェーズ 4：財務諸表詳細（低優先度）

**対象：** カテゴリ C  
**工数目安：** 中  
**備考：** カテゴリ B の経営指標サマリーと大部分が重複します。業種によってスキーマが異なるため（例：金融業は `jppfs_cor:OperatingRevenue1` の代わりに独自要素を使う）、全業種に対応するには追加のハンドリングが必要です。

---

## 注意事項

### 値が「－」の場合

CSV 上で数値が「該当なし」のとき、値は `"－"`（全角ハイフン）が入ります。`None` として扱うよう、パース時に除外処理が必要です。

```rust
let value = fact.value_as_str()?;
```

### 連結 vs 個別

連結財務諸表を作成している企業では `ConsolidatedMember` コンテキストが使われます。
コンテキスト ID による絞り込みロジックでは、連結優先・個別フォールバックの設計が望ましいです。

```
優先順位：
1. CurrentYearDuration_ConsolidatedMember
2. CurrentYearDuration_NonConsolidatedMember
3. CurrentYearDuration（どちらでもない場合）
```

### 業種固有の拡張要素

一部の企業は `jpcrp030000-asr_E{edinetCode}-000:*` という企業固有の要素 ID を使います（例：リース会社がリース収益を独自要素で報告）。これらは標準的な要素 ID では取得できないため、汎用的な抽出では無視して構いません。

---

## 参考資料

- `docs/ESE140133.pdf` — EDINET 書類閲覧操作ガイド
- `docs/ESE140206.pdf` — EDINET API 仕様書（Version 2）
- [EDINET タクソノミ](https://disclosure2dl.edinet-fsa.go.jp/) — 要素 ID の完全な定義はタクソノミドキュメントを参照
- `data/csv/` 配下のサンプル CSV — 実際の要素 ID とコンテキスト ID の確認に使用可能
