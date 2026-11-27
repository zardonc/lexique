"""Capture fixed noun headers and prepare the sense-bound gender review table."""

import argparse
import json
from pathlib import Path
import urllib.parse
import urllib.request

from cefr_gender_pilot import (
    extract_wiktionary_gender_markers,
    load_lexique_noun_genders,
    recover_full_header_line,
    triage_core_noun_genders,
)

from core_pilot import apply_human_review

ROOT = Path(__file__).resolve().parents[1]

# The selected usage narrows some multi-gender headers; other unchanged spellings allow both.
RESIDUAL_DECISIONS = {
    303: "m", 442: "m", 826: "mf", 1136: "mf", 1239: "mf", 1244: "mf",
    1509: "mf", 1553: "mf", 1639: "mf", 1750: "m", 1770: "mf", 2091: "m",
    2193: "mf", 2217: "mf", 2260: "mf", 2291: "mf", 2992: "mf", 3044: "mf",
    3153: "mf", 3546: "mf", 3634: "mf", 3702: "mf",
}


def selected_header(raw, item, ipa_line):
    definition = item["def_span"]
    verified = recover_full_header_line(raw, line_1_based=definition["line_1_based"],
                                        column_1_based=definition.get("column_1_based"),
                                        raw_source_excerpt=definition["raw_source_excerpt"])
    if verified is not None:
        lines = raw.splitlines()
        for index in range(definition["line_1_based"] - 2, -1, -1):
            line = lines[index]
            if line.startswith("===") or line.startswith("{{S|"):
                break
            if line.startswith("'''" + item["source_headword"] + "'''"):
                return line, index + 1
    return ipa_line, item["ipa_span"]["line_1_based"]


def fetch_missing_revisions(items, sources, capture_dir):
    missing = set()
    for item in items:
        source = sources[item["source_id"]]
        query = dict(urllib.parse.parse_qsl(urllib.parse.urlsplit(source["fixed_url"]).query))
        if str(query.get("oldid")) != str(source["revid"]):
            raise ValueError(f"Fixed URL revision mismatch: {source['id']}")
        if not (capture_dir / f"{source['revid']}.wiki").is_file():
            missing.add(source["revid"])
    revisions = sorted(missing)
    for offset in range(0, len(revisions), 50):
        batch = revisions[offset:offset + 50]
        params = {"action": "query", "prop": "revisions", "revids": "|".join(map(str, batch)),
                  "rvprop": "ids|content", "rvslots": "main", "format": "json",
                  "formatversion": "2", "maxlag": "5"}
        request = urllib.request.Request(
            "https://fr.wiktionary.org/w/api.php?" + urllib.parse.urlencode(params),
            headers={"User-Agent": "Lexique/0.1 (https://github.com/zardonc/lexique) noun-header research"})
        with urllib.request.urlopen(request, timeout=45) as response:
            payload = json.load(response)
        if "error" in payload:
            raise ValueError(f"Wiktionary revision capture failed: {payload['error']}")
        received = {}
        for page in payload["query"]["pages"]:
            for revision in page.get("revisions", []):
                revid = revision["revid"]
                content = revision["slots"]["main"]["content"]
                if revid not in batch or revid in received or not isinstance(content, str) or not content:
                    raise ValueError(f"Unexpected revision capture: {revid}")
                received[revid] = content
        if set(received) != set(batch):
            raise ValueError(f"Missing fixed revisions: {set(batch) - set(received)}")
        for revid, content in received.items():
            (capture_dir / f"{revid}.wiki").write_text(content, encoding="utf-8", newline="\n")


def capture(item, sources, capture_dir):
    source = sources[item["source_id"]]
    span = item["ipa_span"]
    cache = capture_dir / f"{source['revid']}.wiki"
    raw = cache.read_text(encoding="utf-8")
    line = recover_full_header_line(raw, line_1_based=span["line_1_based"],
                                    column_1_based=span.get("column_1_based"),
                                    raw_source_excerpt=span["raw_source_excerpt"])
    if line is None:
        raise ValueError(f"Fixed revision does not replay IPA coordinates: {item['sense_id']}")
    header, number = selected_header(raw, item, line)
    return header, "fixed_oldid_capture", number


def review_report(rows, review=None):
    residual = [row for row in rows if row["queue"] != "auto_accepted"]
    extra = [row for row in rows if row["queue"] == "auto_accepted" and row["review_status"] != "deterministic_agreement"]
    accepted = sum(row["gender"] is not None for row in rows)
    status = (f"Reviewer: {review['reviewer']}; reviewed on {review['reviewed_on']}. Policy: use Lexique when nonnull, otherwise Candidate; omit when both are null. All {len(review['decisions'])} scoped decisions are recorded in the evidence and replayed during regeneration."
              if review else f"Independent human review is pending for {len(residual) + len(extra)} rows.")
    lines = ["# Core noun gender residual review", "", status, "",
             f"The 187 original residuals and {len(extra)} additional exceptions are retained below. {accepted} of {len(rows)} noun senses have admitted gender.", "",
             "Evidence: [noun-genders.json](../../resources/core-pilot-79/noun-genders.json), CC BY-SA 4.0. Each header link pins its revision and wikitext line. Reviewed Lexique decisions cite [Lexique 3.83](../../resources/core-pilot-79/lexique-provenance.json); Candidate decisions retain the fixed-source header and selected usage. Full captures remain in `target/core-gender-completion/fixed-revisions/`.", ""]
    for title, group in [("Original 187 residuals", residual), ("Additional seven exceptions", extra)]:
        lines.extend([f"## {title}", "", "| Frozen row / sense | Headword | Selected definition / usage | Fixed header / line | Lexique | Candidate | Admitted / reviewer | Rationale / status |",
                      "| --- | --- | --- | --- | --- | --- | --- | --- |"])
        for row in group:
            cells = [f"{row['frozen_row']} / {row['sense_id']}", row["headword"],
                     row["definition_fr"] + " / " + row["usage"],
                     f"[{row['revid']}, L{row['line_1_based']}]({row['fixed_url']}) {row['raw_header']}",
                     row["lexique_gender"] or "null", row["candidate_gender"] or "null",
                     (row["gender"] or "omitted") + " / " + (review["reviewer"] if row["review_status"] == "human_reviewed" and review else "pending"),
                     row["rationale"] + " Status: " + row["review_status"] + "."]
            lines.append("| " + " | ".join(str(cell).replace("|", "\\|").replace("\n", " ") for cell in cells) + " |")
        lines.append("")
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    resource = ROOT / "resources/core-pilot-79"
    curated = json.loads((resource / "curated-inputs.json").read_text(encoding="utf-8"))
    sources = {source["id"]: source for source in curated["sources"]}
    triage = triage_core_noun_genders(curated, load_lexique_noun_genders(ROOT / "resources/lexique-3.83/Lexique383.tsv.gz"))
    args.output.mkdir(parents=True, exist_ok=True)
    capture_dir = args.output / "fixed-revisions"
    capture_dir.mkdir(exist_ok=True)
    items = sorted(triage["auto_accepted"] + triage["residuals"], key=lambda item: item["frozen_row"])
    trimmed = [item for item in items if item["wikt_gender"] is None and item["headword"] != "gens"]
    captured = {}
    failures = []
    try:
        fetch_missing_revisions(trimmed, sources, capture_dir)
    except Exception as error:
        failures.append({"phase": "revision_capture", "error": str(error)})
    for item in trimmed:
        try:
            captured[item["sense_id"]] = capture(item, sources, capture_dir)
        except Exception as error:
            failures.append({"sense_id": item["sense_id"], "error": str(error)})
    rows = []
    for item in items:
        source = sources[item["source_id"]]
        line, mode, number = captured.get(item["sense_id"], (item["ipa_span"]["raw_source_excerpt"], "existing_fixed_excerpt", item["ipa_span"]["line_1_based"]))
        marker, uncertain, _ = extract_wiktionary_gender_markers(line, item["def_span"]["raw_source_excerpt"])
        candidate = None if uncertain else marker
        if marker == "mf" and item["use_hint"] in {"m", "f"}:
            candidate = item["use_hint"]
        if item["frozen_row"] in RESIDUAL_DECISIONS:
            candidate = RESIDUAL_DECISIONS[item["frozen_row"]]
        automatic = item["queue"] == "auto_accepted"
        verified_gender = automatic and marker == item["accepted_gender"]
        rows.append({
            "frozen_row": item["frozen_row"], "headword": item["headword"], "sense_id": item["sense_id"],
            "source_id": source["id"], "fixed_url": source["fixed_url"], "revid": source["revid"],
            "line_1_based": number, "raw_header": line, "capture_mode": mode,
            "queue": item["queue"], "lexique_gender": item["lexique_gender"], "definition_fr": item["definition_fr"],
            "usage": item["use"], "candidate_gender": candidate,
            "gender": item["accepted_gender"] if verified_gender else None,
            "review_status": "deterministic_agreement" if verified_gender else "pending_independent_human_review",
            "rationale": ("Explicit fixed-source gender agrees with the existing single-gender cross-check."
                          if verified_gender else "Selected spelling and sense: " + item["use"] +
                          (" Source does not establish one simple gender; retain null." if candidate is None else
                           " Candidate follows the fixed header within this usage; independent review is pending.")),
        })
    evidence_path = resource / "noun-genders.json"
    review = json.loads(evidence_path.read_text(encoding="utf-8")).get("human_review") if evidence_path.is_file() else None
    apply_human_review(rows, review)
    payload = {"schema": "lexique-core-noun-genders-1", "content_license": "CC-BY-SA-4.0", "rows": rows}
    if review:
        payload["human_review"] = review
    (args.output / "noun-genders.json").write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")
    residual = [row for row in rows if row["queue"] != "auto_accepted"]
    assert len(rows) == 2134 and len(residual) == 187
    (args.output / "residual-table.json").write_text(json.dumps(residual, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")
    (args.output / "noun-gender-residual-review.md").write_text(review_report(rows, review), encoding="utf-8", newline="\n")
    (args.output / "capture-failures.json").write_text(json.dumps(failures, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps({"nouns": len(rows), "residuals": len(residual), "trimmed_headers": len(trimmed), "captured": len(captured), "failures": len(failures)}))
    if failures:
        raise SystemExit("Capture failures remain; inspect capture-failures.json")


if __name__ == "__main__":
    main()
