"""Reproduce the #99 candidate handoff from its retained fixed API captures."""

import csv
from datetime import datetime, timezone
import json
from pathlib import Path
import re

from selection import SELECTION

ROOT = Path(__file__).resolve().parents[3]
OUT = Path(__file__).resolve().parent
CAPTURES = ROOT / "target/issue-99"
RESOURCE = ROOT / "resources/core-pilot-79"
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}


def write(name, value):
    (OUT / name).write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


with (RESOURCE / "scope-manifest.csv").open(encoding="utf-8-sig", newline="") as handle:
    scope = list(csv.DictReader(handle))
index = json.loads((CAPTURES / "source-index.json").read_text(encoding="utf-8"))
sources, contexts, lines = {}, {}, {}
contributors = {}
parts = sorted((CAPTURES / "sources").glob("contributors-*.api.json"))
assert parts and not json.loads(parts[-1].read_text(encoding="utf-8")).get("continue")
for path in parts:
    data = json.loads(path.read_text(encoding="utf-8"))
    for page in data["query"]["pages"]:
        contributors.setdefault(page["title"], set()).update(
            person["name"] for person in page.get("contributors", []))

for capture in index:
    title = capture["title"]
    source_id = f"b99_fr_{capture['pageid']}"
    raw = json.loads((ROOT / capture["api_path"]).read_text(encoding="utf-8-sig"))
    page = next(page for page in raw["query"]["pages"] if page["title"] == title)
    revision = next(rev for rev in page["revisions"] if rev["revid"] == capture["revid"])
    body = revision["slots"]["main"]["content"]
    assert body == (ROOT / capture["raw_path"]).read_text(encoding="utf-8"), title
    assert revision["timestamp"] == capture["timestamp"]
    lines[title] = body.splitlines()
    frame = ("Current multi-title API query: actual returned revision retained."
             if "batch-current-fixed" in capture["api_path"] else
             "Existing API capture copied byte-identically; original path retained."
             if capture["capture"].startswith("byte_identical") else
             "First-party API revision selected at 2026-10-01T00:00:00Z cutoff.")
    sources[title] = dict(id=source_id, edition="fr", entry_language="fr", title=title,
                         pageid=capture["pageid"], revid=capture["revid"], timestamp=capture["timestamp"],
                         fixed_url=capture["fixed_url"], history_url=capture["history_url"],
                         attribution="Wiktionnaire contributors (full linked article history); Lexique contributors for adaptations and new explanatory translations.",
                         content_license="CC-BY-SA-4.0", license_url="https://creativecommons.org/licenses/by-sa/4.0/legalcode.en",
                         source_frame=frame, selected_text_review="pending_independent_selected_span_review")
    notices = [{"line_1_based": number, "raw_marker": line,
                "declared_source": "Académie française, Dictionnaire de l’Académie française, eighth edition (1932–1935).",
                "selected_text_authorship_status": "unresolved_at_import_boundary"}
               for number, line in enumerate(lines[title], 1) if "{{Import:DAF8" in line]
    contexts[title] = dict(id=source_id, title=title, current_content_revid=capture["revid"],
                          fixed_url=capture["fixed_url"], history_url=capture["history_url"],
                          status="author_source_span_validation_passed_independent_review_pending",
                          selected_locations=[], import_notices=notices,
                          contributor_capture_parts=[str(path.relative_to(ROOT)) for path in parts],
                          current_registry_names=len(contributors.get(title, set())),
                          attribution_boundary="Paginated current contributor registry and full history credit are retained; they do not establish selected-sentence authorship.",
                          selected_text_rights_status="pending_independent_import_and_selected_text_boundary_audit")


def binding(title, line_number):
    source = sources[title]
    return dict(source_id=source["id"], revid=source["revid"], url=source["fixed_url"],
                history_url=source["history_url"], line_1_based=line_number,
                reviewed_excerpt=lines[title][line_number - 1])


groups, excerpts, reviews, usages, supporting = [], [], [], [], []
for row, definition_line, ipa_line, ipa, french, english, chinese, use, excluded in SELECTION:
    frozen = scope[row - 1]
    title = frozen["word"]
    identity = f"b99-{row}-{title}-{frozen['lexique_cgram'].lower().replace(':', '-')}-selected"
    assert lines[title][definition_line - 1].startswith("# "), (row, definition_line)
    assert ipa.strip("/") in lines[title][ipa_line - 1], (row, ipa_line)
    french_heading = max(number for number, line in enumerate(lines[title][:definition_line], 1) if line.startswith("== "))
    assert lines[title][french_heading - 1] == "== {{langue|fr}} ==", row
    supplementary = {904: [(title, 31)], 913: [("certain", 41), ("certain", 42)],
                     914: [("certain", 139), ("certain", 141), ("certain", 142)],
                     935: [(title, 49), (title, 51)], 944: [(title, 16)],
                     949: [(title, 85), (title, 87)]}.get(row, [])
    source_support = [binding(word, number) for word, number in supplementary]
    for record in source_support:
        supporting.append(dict(sense_id=identity, role="grammar_spelling_or_semantic_corroboration", **record))
    fields = {}
    for name, value in zip(LANGUAGES, (french, ipa, english, chinese), strict=True):
        exact = binding(title, ipa_line if name == "ipa" else definition_line)
        modification = ("Selected one recorded IPA variant; notation changed to slashes. No audio or media copied."
                        if name == "ipa" else
                        "Lexique scoped adaptation of selected French semantics; wiki markup removed."
                        if name == "french_definition" else
                        "New Lexique explanatory translation of the selected French use, not an upstream bilingual quotation.")
        if row in (913, 914) and name != "ipa":
            modification += " Exact plural-form entry combined with the separately fixed certain indefinite-use evidence."
        if row == 949 and name != "ipa":
            modification += " Abstract nominalization supported separately by TLFi A.1 grammar evidence; no TLFi, Robert or Capital prose copied."
        fields[name] = dict(language=LANGUAGES[name], text=value, source_id=exact["source_id"],
                            source_span=dict(line_1_based=exact["line_1_based"], raw_source_excerpt=exact["reviewed_excerpt"]),
                            content_license="CC-BY-SA-4.0", modification=modification,
                            author_validation="passed_fixed_source_span_and_scoped_draft_check",
                            independent_review="pending", rights_eligibility_status="pending_selected_span_audit")
        excerpts.append(dict(sense_id=identity, field=name, **exact))
        reviews.append(dict(sense_id=identity, frozen_row=row, headword=title, pos=frozen["lexique_cgram"],
                            field=name, language=LANGUAGES[name], proposed_text=value, exact_source=exact,
                            supporting_sources=source_support, author_validation="passed_source_binding_check",
                            accepted_text_quality_status="pending", rights_eligibility_status="pending_selected_span_audit", reviewer=None))
        contexts[title]["selected_locations"].append(dict(sense_id=identity, field=name, line_1_based=exact["line_1_based"]))
    sense = dict(sense_id=identity, source_headword=title, use=use, use_language="en", excluded_uses=excluded,
                 independent_review="pending", resource_admission="pending_independent_quality_and_rights_review",
                 cefr=None, fields=fields)
    groups.append(dict(frozen_row=row, headword=title, lexique_cgram=frozen["lexique_cgram"],
                       subtitle_lemma_per_million=frozen["subtitle_lemma_per_million"],
                       subtitle_pos_per_million=frozen["subtitle_pos_per_million"], senses=[sense]))
    usages.append(dict(sense_id=identity, frozen_row=row, headword=title, pos=frozen["lexique_cgram"],
                       proposed_usage=use, language="en", source_headword=title,
                       author_validation="selected_use_checked_against_fixed_source_and_recorded_restrictions",
                       accepted_usage_quality_status="pending", everyday_sense_status="pending_independent_review",
                       special_review_focus="Abstract neutral noun: verify POS mapping and modern everyday fit." if row == 949 else None,
                       reviewer=None))

row = 934
frozen = scope[row - 1]
reason = "Fixed Wiki revision 39772403 places lowercase Roman one in translingual lines 41–43. Its French section at 48 has letter nouns and colloquial pronouns, not a numeral adjective. Current Robert labels i a noun/symbol and Roman one as uppercase I. An eligible modern everyday French lowercase i/ADJ:num use and its pronunciation/spelling/POS bridge remain unproved."
groups.append(dict(frozen_row=row, headword=frozen["word"], lexique_cgram=frozen["lexique_cgram"],
                   subtitle_lemma_per_million=frozen["subtitle_lemma_per_million"],
                   subtitle_pos_per_million=frozen["subtitle_pos_per_million"], senses=[], unresolved_reason=reason))
groups.sort(key=lambda group: group["frozen_row"])
assert [group["frozen_row"] for group in groups] == list(range(901, 951))
assert len(excerpts) == len(reviews) == 196 and len(usages) == 49
assert len(sources) == 44

write("review-candidates.json", dict(schema="lexique-d2-content-candidate-handoff-1", ticket=99,
      status="author_validated_candidates_independent_review_pending", scope_denominator_groups=4087,
      groups=groups))
write("sources.json", dict(sources=list(sources.values())))
corroboration = []
for row, filename, url, section, note in [
    (934, "i-robert", "https://dictionnaire.lerobert.com/definition/i", "Modern definition: noun/symbol and uppercase Roman I.",
     "Does not establish the frozen lowercase numeral-adjective use."),
    (935, "i-robert", "https://dictionnaire.lerobert.com/definition/i", "Modern definition: nom masculin invariable; Latin letter I, i.",
     "Corroborates the French letter-name noun/POS mapping without adding an alias."),
    (949, "incroyable-tlfi", "https://www.cnrtl.fr/definition/incroyable", "A.1: emploi substantif masculin singulier à valeur de neutre.",
     "Grammar fact supports abstract noun l’incroyable; quoted Valéry example is dated 1931. No TLFi prose is included in fields."),
    (949, "incroyable-robert", "https://dictionnaire.lerobert.com/definition/incroyable", "Examples: le plus incroyable; Capital, 04/09/2020, hydrogen article.",
     "Modern usage lead; Robert explicitly says partner examples are automatically selected and not reviewed by its editors. Noun definition itself is the historical Directoire sense. No Robert/Capital prose is included in fields."),
]:
    raw_path = CAPTURES / "sources" / (filename + ".html")
    corroboration.append(dict(frozen_row=row, url=url, section=section, assessment=note,
        captured_at_utc=datetime.fromtimestamp(raw_path.stat().st_mtime, timezone.utc).isoformat(),
        fixed_local_capture=str(raw_path.relative_to(ROOT)),
        text_capture=str(raw_path.with_suffix(".txt").relative_to(ROOT)),
        role="read_only_grammar_or_usage_corroboration", independent_review="pending"))
write("source-context.json", dict(current_content=list(contexts.values()),
      rights_review="Pending independent selected-text/import-boundary audit; article licence or Import:DAF8 alone does not establish eligible original prose.",
      grammar_corroboration=corroboration))
write("source-excerpts.json", dict(fields=excerpts, supporting_evidence=supporting))
write("field-review.json", dict(schema="lexique-d2-field-review-draft-1", kind="author_prepared_for_separate_reviewer",
      independent_acceptance=False, reviewer=None, fields=reviews, usage=usages,
      unresolved_fields=[dict(frozen_row=934, headword="i", pos="ADJ:num", field=name, reason=reason) for name in LANGUAGES]))
write("reuse.json", dict(reused_groups=[], note="No ticket rows were admitted at the confirmed integration baseline; all previous records remain unchanged."))
write("source-captures.json", dict(sources=index,
      contributor_registry=dict(parts=[str(path.relative_to(ROOT)) for path in parts],
                                pagination_complete=True, names=sum(len(names) for names in contributors.values()),
                                selected_sentence_authorship_proved=False)))
write("admission.json", dict(ticket=99, frozen_groups=50, proposed_groups=49, proposed_fields=196,
      independently_accepted_new_groups=0, independently_accepted_new_fields=0, unresolved_rows=[934],
      special_independent_usage_review_rows=[949], resource_admission="incomplete", native_checks="parent_owned_not_run"))
print(json.dumps(dict(frozen_groups=50, proposed_groups=49, proposed_fields=196, fixed_sources=len(sources),
      selected_spans_validated=len(excerpts), unresolved_rows=[934], independent_acceptance=False), ensure_ascii=False))
