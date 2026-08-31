#!/usr/bin/env python3
"""Evaluate non-null coverage of `edinet get` JSON outputs."""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any


JsonValue = Any


@dataclass
class Answer:
    label: str
    value: JsonValue


@dataclass
class PathStats:
    total: int = 0
    non_null: int = 0
    null_labels: list[str] = field(default_factory=list)

    @property
    def null_count(self) -> int:
        return self.total - self.non_null

    @property
    def coverage(self) -> float:
        if self.total == 0:
            return 0.0
        return self.non_null / self.total


def main() -> int:
    args = parse_args()

    answers: list[Answer] = []
    answers.extend(load_json_paths(args.inputs))
    answers.extend(load_stdin() if args.stdin else [])

    if not answers:
        print("error: no inputs. pass JSON files or --stdin.", file=sys.stderr)
        return 2

    stats = collect_coverage(answers, args.empty_string_null)
    rows = build_rows(stats, args.threshold, args.limit_null_labels)
    rows.sort(key=sort_key(args.sort))

    if args.format == "json":
        print(json.dumps([row_to_json(row) for row in rows], ensure_ascii=False, indent=2))
    else:
        print_table(rows)

    if args.fail_under is not None:
        failed = [row for row in rows if row[4] < args.fail_under]
        return 1 if failed else 0

    return 0


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Calculate non-null coverage by JSON path for outputs from `edinet get` "
            "and highlight fields likely to have extraction misses."
        )
    )
    parser.add_argument(
        "inputs",
        nargs="*",
        type=Path,
        help="JSON files produced by `edinet get`. Use --stdin for standard input.",
    )
    parser.add_argument(
        "--stdin",
        action="store_true",
        help="Read one JSON output from standard input.",
    )
    parser.add_argument(
        "--threshold",
        type=float,
        default=1.0,
        help="Only show paths whose coverage is below this value. Default: 1.0",
    )
    parser.add_argument(
        "--fail-under",
        type=float,
        help="Exit with status 1 when any displayed path is below this coverage.",
    )
    parser.add_argument(
        "--format",
        choices=("table", "json"),
        default="table",
        help="Output format. Default: table",
    )
    parser.add_argument(
        "--sort",
        choices=("coverage", "path"),
        default="coverage",
        help="Sort order. Default: coverage",
    )
    parser.add_argument(
        "--limit-null-labels",
        type=int,
        default=8,
        help="Maximum number of doc labels shown in null_examples. Default: 8",
    )
    parser.add_argument(
        "--empty-string-null",
        action=argparse.BooleanOptionalAction,
        default=True,
        help="Treat empty strings as null. Default: true",
    )
    return parser.parse_args()


def load_json_paths(paths: list[Path]) -> list[Answer]:
    answers: list[Answer] = []
    for path in paths:
        with path.open(encoding="utf-8") as fh:
            value = json.load(fh)
        answers.append(Answer(label=guess_label(value, path.stem), value=value))
    return answers


def load_stdin() -> list[Answer]:
    value = json.load(sys.stdin)
    return [Answer(label=guess_label(value, "stdin"), value=value)]


def guess_label(value: JsonValue, fallback: str) -> str:
    if isinstance(value, dict):
        metadata = value.get("metadata")
        if isinstance(metadata, dict) and isinstance(metadata.get("doc_id"), str):
            return metadata["doc_id"]
        metadata_ja = value.get("書類情報")
        if isinstance(metadata_ja, dict) and isinstance(metadata_ja.get("書類ID"), str):
            return metadata_ja["書類ID"]
    return fallback


def collect_coverage(answers: list[Answer], empty_string_null: bool) -> dict[str, PathStats]:
    stats: dict[str, PathStats] = {}
    scalar_paths = sorted({path for answer in answers for path in flatten_scalar_paths(answer.value)})

    for path in scalar_paths:
        path_stats = stats.setdefault(path, PathStats())
        for answer in answers:
            value = get_scalar_path(answer.value, path)
            path_stats.total += 1
            if is_non_null(value, empty_string_null):
                path_stats.non_null += 1
            else:
                path_stats.null_labels.append(answer.label)

    for answer in answers:
        for path, value in flatten_array_item_values(answer.value):
            path_stats = stats.setdefault(path, PathStats())
            path_stats.total += 1
            if is_non_null(value, empty_string_null):
                path_stats.non_null += 1
            else:
                path_stats.null_labels.append(answer.label)

        for path, length in flatten_array_lengths(answer.value):
            path_stats = stats.setdefault(path, PathStats())
            path_stats.total += 1
            if length > 0:
                path_stats.non_null += 1
            else:
                path_stats.null_labels.append(answer.label)

    return stats


def flatten_scalar_paths(value: JsonValue, prefix: str = "") -> set[str]:
    paths: set[str] = set()
    if isinstance(value, dict):
        for key, child in value.items():
            child_prefix = join_path(prefix, key)
            if isinstance(child, list):
                continue
            paths.update(flatten_scalar_paths(child, child_prefix))
        return paths
    if isinstance(value, list):
        return paths
    paths.add(prefix)
    return paths


def flatten_array_item_values(value: JsonValue, prefix: str = "") -> list[tuple[str, JsonValue]]:
    rows: list[tuple[str, JsonValue]] = []
    if isinstance(value, dict):
        for key, child in value.items():
            rows.extend(flatten_array_item_values(child, join_path(prefix, key)))
    elif isinstance(value, list):
        array_prefix = f"{prefix}[]"
        for item in value:
            rows.extend(flatten_array_item_leaf_values(item, array_prefix))
    return rows


def flatten_array_item_leaf_values(value: JsonValue, prefix: str) -> list[tuple[str, JsonValue]]:
    rows: list[tuple[str, JsonValue]] = []
    if isinstance(value, dict):
        for key, child in value.items():
            rows.extend(flatten_array_item_leaf_values(child, join_path(prefix, key)))
    elif isinstance(value, list):
        for item in value:
            rows.extend(flatten_array_item_leaf_values(item, f"{prefix}[]"))
    else:
        rows.append((prefix, value))
    return rows


def flatten_array_lengths(value: JsonValue, prefix: str = "") -> list[tuple[str, int]]:
    rows: list[tuple[str, int]] = []
    if isinstance(value, dict):
        for key, child in value.items():
            rows.extend(flatten_array_lengths(child, join_path(prefix, key)))
    elif isinstance(value, list):
        rows.append((f"{prefix}[]", len(value)))
        for item in value:
            rows.extend(flatten_array_lengths(item, f"{prefix}[]"))
    return rows


def get_scalar_path(value: JsonValue, path: str) -> JsonValue:
    current = value
    for part in path.split("."):
        if not isinstance(current, dict) or part not in current:
            return None
        current = current[part]
    return current


def join_path(prefix: str, key: str) -> str:
    if not prefix:
        return key
    return f"{prefix}.{key}"


def is_non_null(value: JsonValue, empty_string_null: bool) -> bool:
    if value is None:
        return False
    if empty_string_null and isinstance(value, str) and value.strip() == "":
        return False
    return True


def build_rows(
    stats: dict[str, PathStats],
    threshold: float,
    limit_null_labels: int,
) -> list[tuple[str, int, int, int, float, str]]:
    rows = []
    for path, path_stats in stats.items():
        if path_stats.coverage >= threshold:
            continue
        rows.append(
            (
                path,
                path_stats.total,
                path_stats.non_null,
                path_stats.null_count,
                path_stats.coverage,
                ", ".join(path_stats.null_labels[:limit_null_labels]),
            )
        )
    return rows


def sort_key(sort: str):
    if sort == "path":
        return lambda row: row[0]
    return lambda row: (row[4], -row[3], row[0])


def print_table(rows: list[tuple[str, int, int, int, float, str]]) -> None:
    headers = ("path", "total", "non_null", "null", "coverage", "null_examples")
    table = [headers]
    table.extend(
        (
            path,
            str(total),
            str(non_null),
            str(null_count),
            f"{coverage:.1%}",
            null_examples,
        )
        for path, total, non_null, null_count, coverage, null_examples in rows
    )
    widths = [max(len(row[index]) for row in table) for index in range(len(headers))]
    for index, row in enumerate(table):
        print("  ".join(cell.ljust(widths[column]) for column, cell in enumerate(row)))
        if index == 0:
            print("  ".join("-" * width for width in widths))


def row_to_json(row: tuple[str, int, int, int, float, str]) -> dict[str, Any]:
    path, total, non_null, null_count, coverage, null_examples = row
    return {
        "path": path,
        "total": total,
        "non_null": non_null,
        "null": null_count,
        "coverage": coverage,
        "null_examples": [value for value in null_examples.split(", ") if value],
    }


if __name__ == "__main__":
    raise SystemExit(main())
