"""Shared verification of fixed source captures for core batch tests."""
import json
from urllib.parse import parse_qs, urlparse


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


def verify_archived_contributor_captures(test_case, captures, resolve_path):
    """Aggregate contributor credits and verify complete archived pagination."""
    names, anonymous, chains = {}, {}, {}
    for capture in captures:
        path = resolve_path(capture)
        data = json.loads(path.read_bytes())
        prefix, part = path.name.removesuffix(".api.json").rsplit("-", 1)
        chains.setdefault(prefix, {})[int(part)] = (path, data)
        for page in data["query"]["pages"]:
            names.setdefault(page["title"], set()).update(p["name"] for p in page.get("contributors", []))
            if "anoncontributors" in page:
                anonymous[page["title"]] = page["anoncontributors"]
    for parts in chains.values():
        test_case.assertEqual(sorted(parts), list(range(1, len(parts) + 1)))
        for part, (_, data) in parts.items():
            continuation = data.get("continue")
            if part == len(parts):
                test_case.assertFalse(continuation)
            else:
                test_case.assertTrue(continuation)
                request = resolve_path(str(parts[part + 1][0].with_suffix(".request.txt"))).read_text(encoding="utf-8")
                query = parse_qs(urlparse(request).query)
                for key, value in continuation.items():
                    test_case.assertEqual(query[key], [str(value)])
    return names, anonymous, chains
