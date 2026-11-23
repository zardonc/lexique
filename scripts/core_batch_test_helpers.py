"""Shared verification of fixed source captures for core batch tests."""
import json


def verify_archived_source_captures(test_case, captures, sources, resolve_path):
    """Check archived bytes and revision metadata, then return source lines."""
    raw_lines = {}
    for capture in captures:
        api = json.loads(resolve_path(capture["api_path"]).read_text(encoding="utf-8"))
        page = next(p for p in api["query"]["pages"] if p["pageid"] == capture["pageid"])
        rev = next(r for r in page["revisions"] if r["revid"] == capture["revid"])
        raw = resolve_path(capture["raw_path"]).read_bytes()
        test_case.assertEqual(raw, rev["slots"]["main"]["content"].encode("utf-8"))
        test_case.assertEqual(rev["timestamp"], capture["timestamp"])
        test_case.assertEqual(page["title"], capture["title"])
        raw_lines[capture["source_id"]] = raw.decode("utf-8").splitlines()
        test_case.assertEqual(sources[capture["source_id"]]["revid"], rev["revid"])
    return raw_lines
