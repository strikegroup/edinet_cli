# Development Guide

このドキュメントは、この CLI を開発・保守する人向けのメモです。利用方法だけを知りたい場合は [README](../README.md) を参照してください。

## Get Started（開発者向け）

### 1. 前提ツールを用意する

- Rust（`cargo` が使える状態）
- SQLite（`sqlite3` コマンド。DB の中身を直接確認したい場合のみ）

### 2. リポジトリを取得して移動する

```bash
git clone https://github.com/strikegroup/edinet_cli
cd edinet_cli
```

### 3. 開発用環境変数を設定する

`.env.example` をコピーして `.env` を作成します。

```bash
cp .env.example .env
```

`DATABASE_URL` と `ASRS_CSV_CACHE_DIR` は通常未指定で問題ありません。必要なときだけ上書きします。

```env
# Optional overrides:
# DATABASE_URL=sqlite:///absolute/path/to/asrs.db
# ASRS_CSV_CACHE_DIR=/absolute/path/to/csv-cache
```

API キーは環境変数ではなく、`setup` コマンドで `config.toml` に保存します。

```bash
cargo run -- setup --key <YOUR_EDINET_API_KEY>
```

### 4. まず壊れていない状態を確認する

```bash
cargo check
cargo test
cargo run -- --help
cargo run -- setup --help
cargo run -- update --help
cargo run -- search --help
cargo run -- get --help
cargo run -- clear --help
cargo run -- status --help
```

### 5. 開発中によく使う動作確認

既定 DB の状態確認:

```bash
cargo run -- status
```

当日分だけ更新して `update` の疎通確認:

```bash
cargo run -- update --today
```

特定企業の最新書類を取得して `get` の疎通確認:

```bash
cargo run -- get --edinet-code E00424
```

## プロジェクト構成

このリポジトリは単一 crate 構成です。CLI と内部モジュールを同じ crate 内で管理します。

```text
.
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── store/
│   │   ├── entities/
│   │   └── open_db.rs
│   ├── updater/
│   ├── searcher/
│   └── getter/
└── docs/
```

主な責務は次の通りです。

- `src/main.rs`: CLI のサブコマンド、引数、実行フローを定義します。
- `src/store/open_db.rs`: SQLite 接続とスキーマ初期化を担当します。
- `src/store/entities/`: SeaORM Entity と DB カラム定義を管理します。
- `src/updater/`: EDINET の日次書類メタデータを取得し、検索用 DB に保存します。
- `src/searcher/`: 保存済みメタデータから有価証券報告書候補を検索します。
- `src/getter/`: 有価証券報告書 CSV を取得・展開・読み込み、主要項目を JSON 化します。

## DB 方針

DB アクセスは SeaORM を使います。テーブル定義は `src/store/entities/` の Entity を正とし、アプリ起動時に必要なテーブルと index を作成します。

現在のローカル DB は開発用データとして扱います。スキーマ変更時に既存 DB の互換性は維持しません。必要に応じて `clear` で作り直してください。

```bash
cargo run -- clear
```

`DATABASE_URL` を指定しない場合は OS 標準のアプリデータディレクトリ配下の SQLite DB が使われます。検証用 DB を分けたい場合は `DATABASE_URL` を明示してください。

```bash
DATABASE_URL=sqlite:///tmp/asr.db cargo run -- status
```

## 命名方針

CLI と内部モジュールは、次の操作モデルに揃えています。

- `update`: 検索に使う書類メタデータを更新する
- `search`: 候補を一覧表示する
- `get`: 対象の有価証券報告書本文を取得する
- `status`: 保存済みデータの状態を見る

DB Entity は `store/entities` に置き、外部出力用の型とは分離します。
`AsrDocumentMetadata` は `document_metadatas` 由来の検索結果モデルで、ASR CSV を読み込むための `doc_id` と JSON 出力用の書類情報を保持します。
CSV 解析結果の `AsrReport` とは出力直前まで結合しません。

## 動作確認の観点

変更後は、最低限次を確認します。

```bash
cargo check
cargo test
cargo run -- --help
cargo run -- setup --help
cargo run -- update --help
cargo run -- search --help
cargo run -- get --help
```

DB スキーマや検索条件を触った場合は、一時 DB で `status` や `search` の入口も確認します。

```bash
DATABASE_URL=sqlite:///tmp/asr.db cargo run -- status
DATABASE_URL=sqlite:///tmp/asr.db cargo run -- search トヨタ
```

`update` の引数なし動作を API 呼び出しなしで確認したい場合は、直近 1 年分の `updated_document_metadatas_list` を一時 DB に投入してから実行します。

```bash
sqlite3 /tmp/asr.db "WITH RECURSIVE dates(file_date) AS (SELECT date('now', '-364 days') UNION ALL SELECT date(file_date, '+1 day') FROM dates WHERE file_date < date('now')) INSERT OR REPLACE INTO updated_document_metadatas_list (file_date, updated_at, result_count) SELECT file_date, datetime('now'), 0 FROM dates"
DATABASE_URL=sqlite:///tmp/asr.db cargo run -- update
```

## ローカルデータの削除

保存済みデータを一度まっさらにしたい場合は `clear` を使います。

```bash
cargo run -- clear
```

このコマンドは現在利用している SQLite DB ファイルと CSV キャッシュを削除します。`config.toml` に保存した API キーは残ります。
