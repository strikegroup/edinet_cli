# Development Guide

このドキュメントは、この CLI を開発・保守する人向けのメモです。利用方法だけを知りたい場合は [README](../README.md) を参照してください。

## Get Started（開発者向け）

### 1. 前提ツールを用意する

- mise（Rust、Node.js、Python と開発用 CLI の管理）
- SQLite（`sqlite3` コマンド。DB の中身を直接確認したい場合のみ）

Rust、Node.js 24、Python、GitHub CLI、cargo-dist、release-plz、cargo-about は `mise.toml` で管理します。

```bash
mise install
```

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
SQLX_OFFLINE=true
# EDINET_API_KEY=your-api-key
# Optional overrides:
# DATABASE_URL=sqlite:///absolute/path/to/asrs.db
# ASRS_CSV_CACHE_DIR=/absolute/path/to/csv-cache
```

API キーは `setup` コマンドで `config.toml` に保存するか、`EDINET_API_KEY` 環境変数で指定します。コマンドラインの `--key`、`EDINET_API_KEY`、`config.toml` の順に優先されます。

```bash
cargo run -- setup --key <YOUR_EDINET_API_KEY>
```

### 4. まず壊れていない状態を確認する

```bash
cargo fmt --check
cargo check
cargo test
cargo run -- --help
cargo run -- setup --help
cargo run -- update --help
cargo run -- search --help
cargo run -- get --help
cargo run -- download --help
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
│   ├── app_config.rs
│   ├── app_paths.rs
│   ├── document_id.rs
│   ├── main.rs
│   ├── zip_archive.rs
│   ├── downloader/
│   ├── getter/
│   ├── searcher/
│   ├── store/
│   │   ├── entities/
│   │   └── open_db.rs
│   ├── updater/
│   └── ...
├── docs/
│   └── development.md
└── tools/
```

主な責務は次の通りです。

- `src/main.rs`: CLI のサブコマンド、引数、実行フローを定義します。
- `src/app_config.rs`、`src/app_paths.rs`: API キーとローカルデータの保存先を管理します。
- `src/document_id.rs`: EDINET 書類 ID の形式を検証します。
- `src/zip_archive.rs`: ZIP のパスと展開量を検証しながら安全に展開します。
- `src/store/open_db.rs`: SQLite 接続とスキーマ初期化を担当します。
- `src/store/entities/`: SeaORM Entity と DB カラム定義を管理します。
- `src/updater/`: EDINET の日次書類メタデータを取得し、検索用 DB に保存します。
- `src/searcher/`: 保存済みメタデータから有価証券報告書候補を検索します。
- `src/getter/`: 有価証券報告書 CSV を取得・展開・読み込み、主要項目を JSON 化します。
- `src/downloader/`: PDF、XBRL、XBRL 変換 CSV のダウンロードを担当します。
- `tools/`: 出力結果の評価など、開発補助用スクリプトを管理します。

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
cargo fmt --check
cargo check
cargo test
cargo run -- --help
cargo run -- setup --help
cargo run -- update --help
cargo run -- search --help
cargo run -- get --help
cargo run -- download --help
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

## リリース

通常のリリースは、`main` への push を契機に release-plz が作成・更新する Release PR を使います。手動でバージョンタグを作成したり、Release PR のマージ前に `cargo publish` したりしないでください。

Release PR を確認します。

```bash
gh pr list --state open --search 'head:release-plz-'
```

release-plz が提案したバージョンでよければ、Release PR のバージョンは変更しません。`0.1.0` など、意図したバージョンへ変更する場合は PR のブランチで `release-plz set-version` を実行します。`Cargo.toml`、`Cargo.lock`、`CHANGELOG.md` を個別に手修正する必要はありません。

```bash
gh pr checkout <RELEASE_PR_NUMBER>
mise exec -- release-plz set-version 0.1.0
git diff -- Cargo.toml Cargo.lock CHANGELOG.md
```

変更後は公開前チェックを実行し、Release PR のブランチへ push します。

```bash
cargo fmt --check
cargo test --locked
cargo publish --dry-run --allow-dirty
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "chore: release v0.1.0"
git push
gh pr edit <RELEASE_PR_NUMBER> --title "chore: release v0.1.0"
```

上の `0.1.0` は実際に公開するバージョンに読み替えます。PR の Actions が `action_required` の場合は GitHub 上で実行を承認します。権限がある場合は GitHub CLI からも承認できます。

```bash
gh api --method POST \
  repos/strikegroup/edinet_cli/actions/runs/<RUN_ID>/approve
```

CI 成功後に Release PR を `main` へマージすると、release-plz が crates.io への公開と draft GitHub Release の作成を行います。続けて cargo-dist が各 OS 向けバイナリを追加し、GitHub Release、Homebrew、npm、APT へ公開します。

公開ワークフローと各配布先のバージョンを確認します。

```bash
gh run list --limit 10
gh run watch <RUN_ID> --exit-status
gh release view v0.1.0
cargo search edinet_cli --limit 1
npm view edinet-cli version
brew update
brew info strikegroup/tap/edinet
curl --fail --head \
  https://strikegroup.github.io/edinet_cli/apt/dists/stable/InRelease
```

npm と APT の初期設定や障害対応は、`publish-npm.md` と `publish-apt.md` を参照してください。

## ライセンス更新

依存クレートのライセンス文書を更新するには、`mise.toml` で管理している cargo-about を使って生成コマンドを実行します。生成された `THIRD_PARTY_LICENSES.html` もリポジトリへコミットしてください。

```bash
mise exec -- cargo about generate --locked --fail --output-file THIRD_PARTY_LICENSES.html about.hbs
```
