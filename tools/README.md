# Tools

## `evaluate_get_coverage.py`

`edinet get` の JSON 出力を複数件集計し、JSON path ごとの非 null 率を確認する評価ツールです。

保存済みの出力を評価します。

```bash
python3 tools/evaluate_get_coverage.py outputs/*.json
```

標準入力から 1 件の JSON 出力を評価します。

```bash
python3 tools/evaluate_get_coverage.py --stdin < output.json
```

主な列は次の意味です。

- `path`: 集計対象の JSON path。配列要素は `[]` で表します。
- `total`: 評価母数。通常の項目は回答件数、配列内項目は配列要素数です。
- `non_null`: `null` ではなかった件数です。
- `coverage`: `non_null / total` です。
- `null_examples`: `null` が出た回答の doc ID または入力ファイル名です。

CI などで閾値未満を失敗扱いにする場合は `--fail-under` を使います。

```bash
python3 tools/evaluate_get_coverage.py outputs/*.json --threshold 0.8 --fail-under 0.8
```
