# EDINET CLI

EDINET から有価証券報告書を取得し、主要項目を JSON で確認するための CLI ツールです。


## できること

- 企業コードや会社名で有価証券報告書を検索して取得する
- 取得結果を JSON で確認する

## インストール

### Cargo

```bash
cargo install --locked edinet_cli
```

### Homebrew

```bash
brew install strikegroup/tap/edinet
```

## セットアップ

### API キーの登録

最初に EDINET API キーを登録します。

```bash
edinet setup --key <YOUR_EDINET_API_KEY>
```

API キーは、以下の EDINET API 仕様書の 2-3 節の手順で取得してください。

https://disclosure2dl.edinet-fsa.go.jp/guide/static/disclosure/download/ESE140206.pdf

### 検索インデックスを更新

使う前に検索インデックスを更新します。

```bash
edinet update
```

直近一年分の有報を取得できるようになります。


## ユースケース

### 会社名で有報を取得

会社名または提出者名の一部を指定し、該当する最新の有価証券報告書を取得します。

```bash
edinet get --company ストライク
```

```json
{
  "metadata": {
    "file_date": "2025-12-17",
    "doc_id": "S100XAN1",
    "edinet_code": "E32380",
    "sec_code": "61960",
    "filer_name": "株式会社ストライク",
    "period_start": "2024-10-01",
    "period_end": "2025-09-30",
    "submit_date_time": "2025-12-17 12:18",
    "doc_description": "有価証券報告書－第29期(2024/10/01－2025/09/30)"
  },
  "report": {
    "company_overview": {
      "company_history": "２ 【沿革】 年月概要1997年７月東京都足立区において、Ｍ＆Ａ仲介業務を事業目的として設立...",
      "employees": "５ 【従業員の状況】(1) 提出会社の状況2025年９月30日現在...",
      "business_results_summary": [
        {
          "period": "CurrentYear",
          "label": "当期",
          "operating_revenue": 20314153000,
          "ordinary_income": 6341778000,
          "net_income": 4719993000,
          "net_assets": 21474522000,
          "total_assets": 24763151000,
          "employees": 452
        },
        "..."
      ]
    },
    "business_overview": {
      "business_description": "３ 【事業の内容】当社は公認会計士及び税理士が経営主体となり..."
    },
    "facilities": "...",
    "corporate_information": "...",
    "financial_information": "..."
  }
}
```

保存済み CSV キャッシュだけを使う場合は `--offline` を指定します。キャッシュにない書類はダウンロードせずに失敗します。

```bash
edinet get --company ストライク --offline
```

### 会社の営業収益を取得

主要な経営指標から、最新年度の営業収益だけを取得します。

```bash
edinet get --company ストライク | jq '.report.company_overview.business_results_summary[0].operating_revenue'
```

```json
20314153000
```

### 会社の有報の「事業の内容」セクションを取得

有価証券報告書の「事業の内容」を取得します。

```bash
edinet get --company ストライク | jq '.report.business_overview.business_description'
```

```json
"３ 【事業の内容】当社は公認会計士及び税理士が経営主体となり、創業よりＭ＆Ａ(企業合併、企業買収、企業間の資本提携等)の仲介を主たる事業としております。なお、当社はＭ＆Ａ仲介事業の単一セグメントであるため、セグメント情報は記載しておりません。..."
```

### 会社名で候補を探してから有報を取得

会社名の一部から候補を検索し、提出者名または書類IDを指定して有価証券報告書を取得します。

```bash
edinet search トヨタ --limit 3 --json
```

```json
{
  "metadatas": [
    {
      "doc_id": "S100YMDA",
      "edinet_code": "E05031",
      "sec_code": null,
      "jcn": "8010601027383",
      "filer_name": "トヨタファイナンス株式会社",
      "period_start": "2025-04-01",
      "period_end": "2026-03-31",
      "submit_date_time": "2026-06-30 13:07",
      "doc_description": "有価証券報告書－第38期(2025/04/01－2026/03/31)"
    },
    "...",
    {
      "doc_id": "S100Y9AH",
      "edinet_code": "E00540",
      "sec_code": "31160",
      "jcn": "2180301014324",
      "filer_name": "トヨタ紡織株式会社",
      "period_start": "2025-04-01",
      "period_end": "2026-03-31",
      "submit_date_time": "2026-06-09 10:09",
      "doc_description": "有価証券報告書－第101期(2025/04/01－2026/03/31)"
    }
  ]
}
```

検索結果の提出者名を指定して、有価証券報告書を取得します。

```bash
edinet get --company トヨタ紡織株式会社
```

レスポンス形式は「会社名で有報を取得」と同じです。

### EDINET コードで有報を取得

EDINETコードを指定して有価証券報告書を取得します。

```bash
edinet get --edinet-code E00424
```

```json
{
  "metadata": {
    "file_date": "2026-04-14",
    "doc_id": "S100XYDT",
    "edinet_code": "E00424",
    "sec_code": "25900",
    "filer_name": "ダイドーグループホールディングス株式会社",
    "period_start": "2025-01-21",
    "period_end": "2026-01-20",
    "submit_date_time": "2026-04-14 15:33",
    "doc_description": "有価証券報告書－第51期(2025/01/21－2026/01/20)"
  },
  "report": "..."
}
```

### 会社名から最新の有報 PDF をダウンロード

会社名または提出者名の一部を指定し、該当する最新の有価証券報告書を PDF でダウンロードします。

```bash
edinet download pdf ./downloads --company ストライク
```

`./downloads/S100XAN1.pdf` のように、書類 ID をファイル名とした PDF が保存されます。

### 書類 ID を指定して XBRL 変換 CSV をダウンロード

`search` で確認した書類 ID を指定し、XBRL 変換 CSV の ZIP ファイルをダウンロードして展開します。

```bash
edinet download csv ./downloads --doc-id S100XAN1 --extract
```

ZIP ファイルは `./downloads/S100XAN1-csv.zip` に保存され、内容は同名の `./downloads/S100XAN1-csv/` ディレクトリに展開されます。

### 出力 JSON のキーを日本語で出力する場合

`--lang ja` を指定し、JSONのキーを有価証券報告書の日本語項目名で出力します。

```bash
edinet get --company ストライク --lang ja
```

```json
{
  "書類情報": {
    "提出日": "2025-12-17",
    "書類ID": "S100XAN1",
    "EDINETコード": "E32380",
    "証券コード": "61960",
    "提出者名": "株式会社ストライク",
    "事業年度開始日": "2024-10-01",
    "事業年度終了日": "2025-09-30",
    "提出日時": "2025-12-17 12:18",
    "書類概要": "有価証券報告書－第29期(2024/10/01－2025/09/30)"
  },
  "有価証券報告書": {
    "第1 企業の概況": {
      "沿革": "２ 【沿革】 年月概要1997年７月東京都足立区において...",
      "従業員の状況": "５ 【従業員の状況】(1) 提出会社の状況...",
      "主要な経営指標等の推移": ["..."]
    },
    "第2 事業の状況": "...",
    "第3 設備の状況": "...",
    "第4 提出会社の状況": "...",
    "第5 経理の状況": "..."
  }
}
```

### 提出日の範囲で候補を絞り込む場合

提出日時の日付範囲で候補を絞り込み、検索結果の書類IDから有価証券報告書を取得します。

```bash
edinet search トヨタ --from 2026-06-09 --to 2026-06-10 --limit 3 --json
```

```json
{
  "metadatas": [
    {
      "doc_id": "S100Y8NY",
      "edinet_code": "E02144",
      "sec_code": "72030",
      "jcn": "1180301018771",
      "filer_name": "トヨタ自動車株式会社",
      "period_start": "2025-04-01",
      "period_end": "2026-03-31",
      "submit_date_time": "2026-06-10 15:33",
      "doc_description": "有価証券報告書－第122期(2025/04/01－2026/03/31)"
    },
    {
      "doc_id": "S100Y9AH",
      "edinet_code": "E00540",
      "sec_code": "31160",
      "jcn": "2180301014324",
      "filer_name": "トヨタ紡織株式会社",
      "period_start": "2025-04-01",
      "period_end": "2026-03-31",
      "submit_date_time": "2026-06-09 10:09",
      "doc_description": "有価証券報告書－第101期(2025/04/01－2026/03/31)"
    }
  ]
}
```

検索結果の書類IDを指定して、有価証券報告書を取得します。

```bash
edinet get --doc-id S100Y8NY
```

書類IDを直接指定した場合は保存済み書類メタデータを経由しないため、`metadata` は `null` です。

```json
{
  "metadata": null,
  "report": {
    "company_overview": {
      "company_history": "２ 【沿革】 年月概要1933年９月㈱豊田自動織機製作所自動車部を分離独立..."
    },
    "business_overview": "...",
    "facilities": "...",
    "corporate_information": "...",
    "financial_information": "..."
  }
}
```

## よくあるエラー

`failed to read config file ...`

API キーがまだ登録されていません。`edinet setup --key <YOUR_EDINET_API_KEY>` を実行するか、`update` / `get` / `download` に `--key` を付けて実行してください。

`ASR document not found for the given query`

条件に一致する書類が検索用データにない可能性があります。先に `edinet update` を実行してください。

## コマンド

| コマンド | 短縮形 | 概要 |
|---|---|---|
| `setup` | `init` | API キーを登録します |
| `update` | `u` | 検索インデックスを更新します |
| `search` | `s` | 有価証券報告書の候補を検索します |
| `get` | `g` | 有価証券報告書を取得します |
| `download` | `d` | 書類データをダウンロードします |
| `clear` | `c` | ローカルデータを削除します |
| `status` | `st` | 保存済みデータの状態を表示します |

各オプションの短縮形は `edinet <COMMAND> --help` で確認できます。

### `setup`

EDINET API キーを登録します。

```bash
edinet setup --key <YOUR_EDINET_API_KEY>
```

### `update`

書類検索に使う書類メタデータを更新します。

```bash
# 直近1年の未更新日を更新
edinet update

# この実行だけ別のAPIキーを使う
edinet update --key <YOUR_EDINET_API_KEY>

# 直近3年の未更新日を更新
edinet update --years 3

# 最大8件の日次API取得を並列実行
edinet update --years 3 --concurrency 8

# 日次API取得を直列実行
edinet update --years 3 --sequential

# 更新済み日付も含めて直近3年を再取得
edinet update --years 3 --force

# 指定期間を更新
edinet update --from 2026-04-01 --to 2026-04-14

# 今日の分だけ更新
edinet update --today
```

日次 API の取得は既定で最大4件を並列実行します。`--concurrency`（`-p`）で並列数を変更でき、`--sequential`（`-s`）を指定すると直列実行します。`--concurrency 1` でも同じ動作になります。HTTP 429 が返された場合は、待機時間を延ばしながら最大3回再試行します。

引数なしまたは `--years` を指定した更新では、通常は未更新日のみ取得します。`--force`（`-F`）を指定すると、既存インデックスの更新状態を無視して対象期間の全日を再取得します。`--today` と `--from` / `--to` は元から指定期間を再取得します。

### `search`

保存済みメタデータから、有価証券報告書の候補一覧を表示します。

```bash
# 提出者名、EDINETコード、証券コード、法人番号をまとめて検索
edinet search トヨタ

# 証券コードで検索
edinet search --sec-code 7203

# 2025年に提出された有報を検索
edinet search --company ストライク --year 2025

# 提出日時の日付範囲で絞り込み
edinet search トヨタ --from 2026-04-01 --to 2026-04-14

# 2ページ目を表示
edinet search トヨタ --limit 20 --page 2

# JSON 形式で出力
edinet search トヨタ --json
```

`--limit` 未指定時と `--limit 0` 指定時は全件表示します。ページ指定は `--limit` が 1 以上のときに有効です。

検索対象は、CSV を取得できる有価証券報告書に限定されます。日付条件は、書類一覧 API のファイル日付ではなく、書類ごとの提出日時を基準にします。
`--year` は4桁の西暦を指定し、その年の1月1日から12月31日までに提出された有価証券報告書を対象にします。`--date`、`--from`、`--to` とは併用できません。

### `get`

有価証券報告書を取得し、JSON で表示します。

```bash
# 企業コードで取得
edinet get --edinet-code E00424

# 会社名で取得
edinet get --company トヨタ

# 2025年に提出された最新の有報を取得
edinet get --company トヨタ --year 2025

# 書類IDを指定して取得
edinet get --doc-id S100XYDT

# 日本語キーで出力
edinet get --edinet-code E00424 --lang ja

# この実行だけ別のAPIキーを使う
edinet get --company トヨタ --key <YOUR_EDINET_API_KEY>
```

`--doc-id` は `--edinet-code`、`--company`、`--year` と併用できません。条件に一致する書類が複数ある場合は、その年に提出された最新の有価証券報告書を取得します。
`--lang ja` を指定したときだけ、JSON のフィールド名を日本語ラベルで出力します。既定は `--lang en` です。

### `download`

有価証券報告書の生データを PDF、XBRL、XBRL 変換 CSV のいずれかでダウンロードします。

```text
edinet download <xbrl|pdf|csv> [PATH] [OPTIONS]
```

`PATH` を省略した場合は、カレントディレクトリに保存します。ディレクトリを指定すると、書類 ID に応じて次のファイル名が使われます。

- `xbrl`: `<DOC_ID>-xbrl.zip`
- `pdf`: `<DOC_ID>.pdf`
- `csv`: `<DOC_ID>-csv.zip`

```bash
# 書類 ID を指定して PDF を取得
edinet download pdf ./downloads --doc-id S100XAN1

# 会社名に一致する最新の書類を XBRL 形式で取得
edinet download xbrl ./downloads --company ストライク

# 2025年に提出された最新の書類を PDF で取得
edinet download pdf ./downloads --company ストライク --year 2025

# ZIP をダウンロード後に展開
edinet download csv ./downloads --doc-id S100XAN1 --extract

# この実行だけ別の API キーを使う
edinet download pdf ./downloads --doc-id S100XAN1 --key <YOUR_EDINET_API_KEY>
```

書類 ID を直接指定する `--doc-id` は、`--company`、`--edinet-code`、`--year` と併用できません。書類 ID を指定しない場合は、いずれかの検索条件が必要です。`--year` に4桁の西暦を指定すると、その年に提出された書類だけを対象にします。複数件に一致した場合は、提出日時が最新の書類を取得します。

`--extract` は ZIP 形式で取得する `xbrl` または `csv` で使用できます。PDF では使用できません。ZIP ファイルは残したまま、拡張子を除いた同名ディレクトリへ展開します。

### `clear`

ローカルの SQLite DB と CSV キャッシュを削除します。`config.toml` に保存した API キーは削除しません。

```bash
edinet clear
```

### `status`

保存済みデータ、CSV キャッシュ、API キーの登録状態を表示します。DB が未作成の場合でも、新しい DB は作成しません。

```bash
edinet status
```

## 出力される主な項目

`get` の結果は、保存済み書類メタデータ由来の `metadata` と、XBRL CSV から抽出した `report` を返します。
`--doc-id` 直接指定時は保存済み書類メタデータを経由しないため、`metadata` は `null` です。
`report` は有価証券報告書の章立てに沿って出力されます。

- `company_overview`
  - `company_history`
  - `employees`
  - `business_results_summary`
- `business_overview`
  - `business_description`
  - `performance`
  - `issues_to_address`
  - `risks`
  - `sustainability`
  - `research_and_development`
  - `critical_contracts`
- `facilities`
  - `capital_expenditures`
  - `major_facilities`
  - `facility_plans`
- `corporate_information`
  - `shareholding`
  - `major_shareholders`
  - `dividend_policy`
  - `officers`
  - `corporate_governance`
  - `officer_compensation`
- `financial_information`
  - `segment_information`

## 開発者向け情報

実装構成、ローカル DB、開発時の手順は [Development Guide](docs/development.md) を参照してください。

## ライセンス

このソフトウェアは [Apache License 2.0](LICENSE) で提供します。
