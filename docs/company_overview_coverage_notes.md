# company_overview coverage notes

## 調査対象

- `temp/outputs` 配下の `get` 出力を `tools/evaluate_get_coverage.py` で集計した。
- `download` コマンドで代表書類の PDF と CSV を取得し、PDF 表示上の記載と XBRL CSV の要素を比較した。

## 改善した項目

### `business_results_summary[].operating_revenue`

- 対象例: `S100YKV6`
- PDF では主要な経営指標等の推移に売上収益が記載されている。
- XBRL CSV では `jpcrp_cor:RevenueKeyFinancialData` として出力されていた。
- 既存の営業収益候補にこの要素 ID が含まれていなかったため、`operating_revenue` の候補に追加した。

追加調査で、残りの `operating_revenue` null には次の要素 ID 不足が含まれていた。

- 対象例: `S100YJV2`, `S100YL35`
  - PDF/CSV では営業収入が `jpcrp_cor:OperatingRevenue2SummaryOfBusinessResults` として出力されていた。
- 対象例: `S100YJZJ`, `S100YD3N`, `S100YEQG`
  - PDF/CSV では金融機関等の経常収益が `jpcrp_cor:OrdinaryIncomeSummaryOfBusinessResults` として出力されていた。

どちらも「主要な経営指標等の推移」の売上高・営業収益相当として扱えるため、`operating_revenue` の候補に追加した。

全 CSV キャッシュに対して、既存 `temp/outputs` で `operating_revenue` が `null` のセルを走査した結果、追加した 2 要素で 90 書類・448 セルが補完可能だった。

- `jpcrp_cor:OrdinaryIncomeSummaryOfBusinessResults`: 379 セル
- `jpcrp_cor:OperatingRevenue2SummaryOfBusinessResults`: 69 セル

一方で、`S100X793`, `S100XCGP` などは設立直後・上場直後・表示対象期間不足により一部過年度が PDF/CSV 上も存在しない。これらは要素 ID 追加では改善できない。

## null が妥当と判断した項目

### `S100XVW2`: `per`, `payout_ratio`, `roe`, `total_shareholder_return`

- PDF と XBRL CSV の主要な経営指標等の推移で、対象値が `－` として開示されている。
- 同社は対象期で損失や債務超過があり、PER、ROE、配当性向などが算定対象外として表示されている。
- 数値ではなく非該当を表す開示なので、`null` のままが妥当。

### `S100XCGP`: `Prior3Year`, `Prior4Year` の主要指標

- PDF 上も会社設立後の期間に対応する第1期から第3期までが主な表示対象で、古い期間は存在しない。
- XBRL CSV でも該当する過年度コンテキストの値が存在しない。
- 事業実績が存在しない期間なので、`null` のままが妥当。

### `S100XV05`: `company_history`, `employees`, `business_results_summary`

- PDF には「第2 企業の概況」として沿革、従業員の状況、主要な経営指標等の推移が存在する。
- 取得した CSV は `jpcrp080000-asr` 形式で、通常の `jpcrp030000` の `CompanyHistoryTextBlock`、`InformationAboutEmployeesTextBlock`、`*SummaryOfBusinessResults` 系要素が見当たらない。
- 現在の `get` は XBRL CSV を入力にした抽出であり、このケースを補うには PDF 本文抽出や `jpcrp080000` 専用の表抽出が必要になる。
- CSV 入力だけでは取得できないため、現状の `null` は妥当。ただし PDF 抽出を導入する場合は改善対象になる。

### PER、配当性向、ROE、総株主還元率の多くの null

- PER、配当性向、ROE は、損失、債務超過、配当なし、EPS が非正値の場合に PDF/CSV 上で `－` として開示されることが多い。
- 総株主還元率は、上場直後や比較対象期間不足の場合に PDF/CSV 上で `－` として開示されることが多い。
- これらは数値抽出漏れではなく非該当を表すため、包括的に数値補完するパッチは当てない。
