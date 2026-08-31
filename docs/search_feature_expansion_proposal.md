# CLI検索機能の追加実装提案（詳細版）

## 1. 目的

現状CLIは「条件に合う最新1件を解決して取得する」ユースケースに最適化されている。  
一方、実運用では以下が必要になる。

- 候補を比較しながら選ぶ
- 取下げ/不開示/縦覧満了などの状態を判別する
- 証券コード・法人番号・書類種別コードで絞る
- 検索結果を再利用（CSV出力、下流処理）

このギャップを埋めるために、検索体験を「1件解決」から「一覧検索」へ拡張する。

---

## 2. 現状の整理

### 2.1 できること（現状）

- `get --doc-id` で直接取得
- `get --edinet-code/--filer-name/--file-date/--from-date/--to-date` で条件検索
- 条件一致の**最新1件**を内部で解決して取得

### 2.2 制約（現状）

- 複数候補を確認できない（常に1件）
- フィルタ項目が少ない（`sec_code`, `jcn`, `doc_type_code` 等がない）
- 取下げ/不開示/縦覧満了の意味が出力に表れにくい
- 検索結果のCSV出力がない
- AND/OR/NOTや接頭辞を使った検索式がない

---

## 3. 追加実装の全体方針

1. 新しい `search` コマンドを追加（一覧返却専用）
2. フィルタ項目を段階的に拡張
3. 状態フラグ（withdrawal/disclosure/legal）を検索・表示に反映
4. 出力形式（table/json/csv）を追加
5. `get` は後方互換を維持しつつ、`search` との連携を強化

---

## 4. 提案機能（優先度順）

## 4.1 高優先度

### 4.1.1 `search` コマンド新設（一覧返却）

**目的**  
候補一覧を返し、ユーザが選べるようにする。

**CLI案**

```bash
cargo run -- search --filer-name トヨタ --limit 20 --sort submit_date_desc
```

**主要オプション**

- `--limit <N>`（初期値 20、上限 100）
- `--offset <N>`（初期値 0）
- `--sort <submit_date_desc|submit_date_asc|file_date_desc|file_date_asc>`
- `--format <table|json>`（初期値 table）

**返却列（最小）**

- `doc_id`
- `file_date`
- `submit_date_time`
- `filer_name`
- `edinet_code`
- `sec_code`
- `doc_description`
- `legal_status`
- `withdrawal_status`
- `disclosure_status`
- `csv_flag`

---

### 4.1.2 フィルタ拡張

**追加候補**

- `--sec-code`
- `--jcn`
- `--form-code`
- `--doc-type-code`
- `--legal-status`（`0|1|2`）
- `--withdrawal-status`（`0|1|2`）
- `--disclosure-status`（`0|1|2|3`）
- `--has-csv`（`true|false`）
- `--has-pdf`（`true|false`）

**期待効果**

- 対象絞り込み精度を上げる
- 「取れるはずなのに取れない」を減らす

---

### 4.1.3 キーワード検索式（AND/OR/NOT + 接頭辞）

**追加パラメータ案**

- `--query "<式>"`

**式例**

- `トヨタ OR 自動車`
- `s:トヨタ -g:G`
- `e:E00424`
- `c:7203`
- `h:1234567890123`

**解釈ルール（初期実装）**

- 空白区切りをAND
- `OR` を論理和
- 先頭 `-` をNOT
- 接頭辞のマッピング:
  - `e:` -> `edinet_code`
  - `c:` -> `sec_code`
  - `h:` -> `jcn`
  - `s:` -> `filer_name`

---

## 4.2 中優先度

### 4.2.1 期間プリセット

**追加パラメータ案**

- `--period <today|last-3-days|last-week|last-month|last-6-months|last-year|all>`

`--from-date/--to-date` と排他にする。

---

### 4.2.2 CSV出力

**追加パラメータ案**

- `--export-csv <path>`

`search` の結果をそのまま保存できるようにする。  
データ分析や他システム連携をしやすくする。

---

### 4.2.3 状態理由の明示

`search` の結果に、人が理解しやすい補助列を追加する。

- `availability`: `available | withdrawn | expired | undisclosed`
- `availability_reason`: 簡潔な理由文

---

## 4.3 低優先度

### 4.3.1 関連書類ツリー表示

**追加パラメータ案**

- `--with-related`

`parent_doc_id` を使って、親子（訂正/取下げ）をまとめて表示する。

---

### 4.3.2 全文検索（将来）

抽出済み本文（または要約）をローカルに保持し、FTSで横断検索する。  
これはデータ設計の影響が大きいため、別フェーズで扱う。

---

## 5. DBスキーマ拡張提案

## 5.1 背景

現在の `document_metadatas` は主要項目は持っているが、検索機能強化に必要な項目が不足している。  
特に関連書類表示や高度検索で不足しやすい。

## 5.2 追加カラム候補

- `fund_code TEXT`
- `issuer_edinet_code TEXT`
- `subject_edinet_code TEXT`
- `subsidiary_edinet_code TEXT`
- `current_report_reason TEXT`
- `parent_doc_id TEXT`
- `ope_date_time TEXT`

## 5.3 インデックス候補

- `idx_document_metadatas_sec_code (sec_code)`
- `idx_document_metadatas_jcn (jcn)`
- `idx_document_metadatas_doc_type_code (doc_type_code)`
- `idx_document_metadatas_form_code (form_code)`
- `idx_document_metadatas_parent_doc_id (parent_doc_id)`
- 複合: `(file_date, ordinance_code, doc_type_code, csv_flag)`

---

## 6. コマンド設計（提案）

## 6.1 `search` シグネチャ案

```bash
cargo run -- search \
  [--edinet-code <CODE>] \
  [--sec-code <CODE>] \
  [--jcn <NUMBER>] \
  [--filer-name <TEXT>] \
  [--doc-type-code <CODE>] \
  [--form-code <CODE>] \
  [--file-date <YYYY-MM-DD>] \
  [--from-date <YYYY-MM-DD>] [--to-date <YYYY-MM-DD>] \
  [--period <PRESET>] \
  [--query <EXPR>] \
  [--legal-status <0|1|2>] \
  [--withdrawal-status <0|1|2>] \
  [--disclosure-status <0|1|2|3>] \
  [--has-csv <true|false>] \
  [--has-pdf <true|false>] \
  [--limit <N>] [--offset <N>] \
  [--sort <...>] \
  [--format <table|json>] \
  [--export-csv <PATH>]
```

## 6.2 `get` との役割分担

- `search`: 候補を探して列挙する
- `get`: 最終的に1件を取得してサマリーを返す

### 連携例

```bash
cargo run -- search --filer-name トヨタ --limit 5
cargo run -- get --doc-id S100XXXX
```

---

## 7. エラーハンドリング方針

- 条件未指定: 明示的エラー（現状維持）
- 日付不正: `YYYY-MM-DD` 形式で明示エラー
- 排他条件違反（`--period` と `--from-date/--to-date` など）: 即時エラー
- 検索0件: 正常終了 + `count: 0`（エラーにしない）

---

## 8. 実装フェーズ案

### Phase 1（最短価値）

- `search` コマンド追加
- `limit/offset/sort`
- 既存カラムだけでフィルタ拡張（`sec_code`, `jcn`, 状態フラグ）
- table/json 出力

### Phase 2（検索品質向上）

- クエリ式（AND/OR/NOT + 接頭辞）
- `--period` プリセット
- `--export-csv`

### Phase 3（構造拡張）

- migrationで追加カラム導入
- 関連書類ツリー表示
- 将来の全文検索基盤の下準備

---

## 9. 互換性と移行

- 既存の `get`, `update`, `status`, `setup` は互換維持
- 新機能は `search` 追加で吸収するため破壊的変更を避けられる
- DB拡張は新規migrationで前方追加し、既存データはNULL許容で移行

---

## 10. 受け入れ基準（抜粋）

- `search` が複数件を安定して返す
- `sec_code/jcn/doc_type_code` で絞り込める
- 取下げ・不開示・縦覧満了を結果で判別できる
- `search` 結果から `get --doc-id` につなげられる
- CSV出力が再現可能（同条件で同順序）

---

## 11. 補足（参照仕様）

- `docs/ESE140133.pdf`: 閲覧サイトの検索体験（簡易/詳細/全文、検索結果出力）
- `docs/ESE140206.pdf`: 書類一覧APIの項目定義、状態遷移（取下げ・不開示・縦覧区分）
