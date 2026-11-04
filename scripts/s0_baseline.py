"""#25 fixtures and independent raw-state oracle; no product queries or projections."""
from fractions import Fraction
from functools import lru_cache
import json
import math
from pathlib import Path
import random
import subprocess

REPO = Path(__file__).resolve().parents[1]
SEED = 250_001
NOW = 1_700_000_000


@lru_cache(maxsize=4096)
def clock(now, timezone):
    child = subprocess.run(["node", str(REPO / "scripts/s0-clock.mjs")],
        input=json.dumps({"now": now, "timezone": timezone}), text=True,
        capture_output=True, timeout=10, check=True)
    return json.loads(child.stdout)


def seed_rows(connection, words, scenario, now=NOW):
    """Populate only a freshly initialized, empty fixture, retaining two cards/word."""
    if connection.execute("SELECT COUNT(*) FROM vocabulary").fetchone()[0]:
        raise ValueError("Seed requires an empty isolated vocabulary")
    snapshot = json.dumps({"reference": {"dictionary": "synthetic", "lookup_key": "synthetic",
        "version": "1", "attribution": "synthetic", "license": "MIT",
        "source_url": "https://example.org", "license_url": "https://example.org", "sha256": "a" * 64}, "digest": []})
    for word in range(1, words + 1):
        connection.execute("INSERT INTO vocabulary(id,headword,snapshot,notes,created_at,updated_at) VALUES(?,?,?,'',?,?)",
            [word, f"synthetic-{word}", snapshot, now, now])
        for identity, kind in ((word * 2 - 1, "recognition"), (word * 2, "listening")):
            connection.execute("INSERT INTO review_cards VALUES(?,?,?)", [identity, word, kind])
            phase = "new"
            last, due = None, None
            if scenario == "intraday":
                phase, last, due = "learning_first", now - 120, now - 1
            elif scenario == "future":
                phase, last, due = "review", now - 86400, now + 86400
            elif scenario in ("mixed", "backlog"):
                slot = (identity - 1) % 7
                if slot == 1:
                    # Explicit new state with null due, distinct from absent progress.
                    pass
                elif slot in (2, 3):
                    phase = "learning_first" if slot == 2 else "relearning"
                    last, due = now - (120 if slot == 2 else 172800), now - 1
                elif slot in (4, 5, 6):
                    phase = "review"
                    last, due = now - 864000, now + (86400 if slot == 6 and scenario == "mixed" else -slot * 86400)
            if phase != "new" or scenario in ("mixed", "backlog") and identity % 7 == 2:
                state = {"phase": phase, "memory": None if phase == "new" else {"stability": 10.0, "difficulty": 5.0},
                    "last_review_at": last, "due_at": due}
                connection.execute("INSERT INTO review_progress VALUES(?,1,?)", [identity, json.dumps(state)])
        if scenario == "mixed" and word % 11 == 0:
            connection.execute("INSERT INTO review_paused VALUES(?)", [word])
    connection.commit()


def fnv(due_day, identity):
    value = 14695981039346656037
    for byte in due_day.encode() + identity.to_bytes(8, "little", signed=True):
        value = ((value ^ byte) * 1099511628211) & ((1 << 64) - 1)
    return value


def oracle(connection, now, timezone, health, skipped=()):
    """Read original cards/JSON/history/settings/positions, never a candidate API."""
    day = clock(now, timezone)
    classify = lambda state, zone: (
        "new" if state["phase"] == "new" else "review" if state["phase"] == "review"
        else "intraday" if clock(state["last_review_at"], zone)["day"] == day["day"] else "interday")
    paused = {row[0] for row in connection.execute("SELECT vocabulary_id FROM review_paused")}
    progress = {identity: json.loads(state) for identity, state in connection.execute("SELECT card_id,state FROM review_progress")}
    settings = connection.execute("SELECT new_limit,review_limit FROM review_settings WHERE id=1").fetchone()
    ignore = connection.execute("SELECT value FROM app_metadata WHERE key='new-cards-ignore-review-limit'").fetchone()
    ignore = bool(ignore and ignore[0] == "true")
    used = {"new": 0, "review": 0}
    latest = connection.execute("SELECT MAX(reviewed_at) FROM review_ratings WHERE undo_operation IS NULL").fetchone()[0]
    for quota, before, request in connection.execute("SELECT quota,before_state,request FROM review_ratings WHERE undo_operation IS NULL AND learning_day=? AND reviewed_at<=?", [day["day"], now]):
        if quota == "new":
            used["new"] += 1
        elif classify(json.loads(before), json.loads(request)["timezone"]) in ("review", "interday"):
            used["review"] += 1
    remaining_review = max(0, settings[1] - used["review"] - (0 if ignore else used["new"]))
    remaining_new = max(0, settings[0] - used["new"])
    groups = {name: [] for name in ("new", "review", "interday", "intraday")}
    future = []
    unknown = 0
    states = {}
    for identity, word, kind in connection.execute("SELECT id,vocabulary_id,kind FROM review_cards ORDER BY id"):
        if word in paused or identity in skipped:
            continue
        state = progress.get(identity, {"phase": "new", "due_at": None, "last_review_at": None})
        states[identity] = state
        if kind == "listening":
            status = health.get(word, "missing")
            unknown += status == "unknown"
            if status != "healthy":
                continue
        if state["due_at"] is not None and state["due_at"] > now:
            future.append(state["due_at"])
            continue
        groups[classify(state, timezone)].append(identity)
    if latest is not None and now < latest:
        return {"ids": [], "reason": "clock", "available_at": latest, "learning_day": day["day"],
            "next_day_at": day["next"], "used": used, "unknown_audio": unknown, "classes": {k: 0 for k in groups}}
    before_count = sum(map(len, groups.values()))
    groups["review"].sort(key=lambda identity: (clock(states[identity]["due_at"], timezone)["day"],
        fnv(clock(states[identity]["due_at"], timezone)["day"], identity), identity))
    groups["interday"] = groups["interday"][:remaining_review]
    groups["review"] = groups["review"][:max(0, remaining_review - len(groups["interday"]))]
    new_limit = remaining_new if ignore else min(remaining_new, max(0, remaining_review - len(groups["interday"]) - len(groups["review"])))
    groups["new"] = groups["new"][:new_limit]
    # Merge rational midpoints, independently of the production two-pointer loop.
    mixed = sorted((Fraction(2 * index + 1, 2 * len(groups[kind])), kind != "new", identity)
        for kind in ("new", "review") for index, identity in enumerate(groups[kind]))
    admission = [identity for _, _, identity in mixed] + groups["interday"] + groups["intraday"]
    queue_day = connection.execute("SELECT learning_day FROM review_queue_state WHERE id=1").fetchone()
    saved = [row[0] for row in connection.execute("SELECT card_id FROM review_queue_entries ORDER BY position")]
    if not queue_day or queue_day[0] != day["day"]:
        saved = []
    # Preserve invisible saved anchors; insert newly admitted IDs before the next anchor.
    next_identity = None
    for identity in reversed(admission):
        if identity not in saved:
            saved.insert(len(saved) if next_identity is None else saved.index(next_identity), identity)
        next_identity = identity
    admitted = set(admission)
    ids = sorted((identity for identity in saved if identity in admitted),
        key=lambda identity: (states[identity]["phase"] in ("new", "review"), saved.index(identity)))
    quota_blocked = len(admission) < before_count
    wake = min(future + ([day["next"]] if quota_blocked else []), default=None)
    return {"ids": ids, "reason": "quota" if quota_blocked and (not future or day["next"] < min(future)) else "due" if future else "quota" if quota_blocked else None,
        "available_at": wake, "learning_day": day["day"], "next_day_at": day["next"], "used": used,
        "unknown_audio": unknown, "classes": {k: len(v) for k, v in groups.items()}}


def operations(count=100):
    randomizer = random.Random(SEED)
    kinds = ("rate", "undo", "pause", "resume", "limits", "timezone", "audio", "reopen")
    return [{"step": i, "kind": kinds[i % len(kinds)], "word": randomizer.randint(1, 40),
        "value": randomizer.randint(0, 50)} for i in range(count)]


def summarize(rows):
    if len(rows) < 100:
        raise ValueError("Steady-state percentiles require at least 100 attempts")
    values = sorted(row["ms"] for row in rows)
    failures = sum(row["status"] != "passed" for row in rows)
    return {"attempts": len(rows), "failures": failures, "failure_rate": failures / len(rows),
        "quantile": "nearest rank over all attempts; timeout=60000ms",
        "p50_ms": values[math.ceil(len(values) * .5) - 1], "p95_ms": values[math.ceil(len(values) * .95) - 1],
        "max_ms": values[-1], "accepted": failures == 0}
