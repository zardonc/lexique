"""#25 CPU resource comparisons using the existing admitted model probe."""
import argparse
import hashlib
import json
from pathlib import Path
import statistics
import sys

from measure_preflight_tts import trial
from measure_s0 import execute, new_directory
from measure_tts_generation import finish, namespace, parse_config, provider
from s0_baseline import REPO


def large_audio(resources, root):
    space = namespace()
    config = parse_config("shared,1,2,8,on")
    model = space["create_model"](str(resources / "assets"), 2, provider(config, root))
    text = "Aujourd’hui, nous apprenons le français ensemble. Les enfants vont à l’école avec leurs amis, puis ils rentrent à la maison pour raconter leur journée et préparer tranquillement le dîner en famille."
    return finish(space, model, {"id": 0, "text": text, "voice": "female", "seed": 42}, config, resources, root)


def synthesize(request):
    """Called only by the bounded subprocess; never reuse an output directory."""
    background = request["background"]
    if background is not None and (type(background) is not int or background not in (0, 1, 2, 4)):
        raise ValueError("Invalid background concurrency")
    root = new_directory(Path(request["output"]))
    resources = Path(request["resources"]).resolve()
    return large_audio(resources, root) if background is None else trial(resources, root, background, "on", "fixed")


def run_case(resources, root, background=None):
    command = [str(resources / "runtime/python.exe"), "-I", "-B", "-X", "utf8", "-c",
        f"import sys,json;sys.path.insert(0,{str(REPO / 'scripts')!r});"
        "from measure_s0_resources import synthesize;print(json.dumps(synthesize(json.load(sys.stdin))))"]
    outcome = execute(command, root / "stdout.json", timeout=60,
        request={"resources": str(resources), "output": str(root / "audio"), "background": background})
    outcome["audio_directory"] = "audio"
    if outcome["status"] == "passed":
        outcome["result"] = json.loads((root / "stdout.json").read_text(encoding="utf8"))
    return outcome


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("resources", type=Path)
    parser.add_argument("output", type=Path)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--child", type=int, choices=(0, 1, 2, 4))
    mode.add_argument("--large", action="store_true")
    args = parser.parse_args()
    resources = args.resources.resolve()
    root = new_directory(args.output)
    if args.child is not None or args.large:
        outcome = run_case(resources, root, args.child)
        (root / "results.json").write_text(json.dumps(outcome, indent=2), encoding="utf8")
        print(json.dumps(outcome))
        if outcome["status"] != "passed":
            sys.exit(1)
        return
    budgets = json.loads((REPO / "docs/evidence/issue-25/budgets.json").read_text())
    report = {"budgets_sha256": hashlib.sha256((REPO / "docs/evidence/issue-25/budgets.json").read_bytes()).hexdigest(),
        "provider": "cpu", "steps": 8, "bitrate": 48000, "trials": [], "boundary": "model only; actual product P0/IPC/playback separately recorded"}
    for repeat in range(3):
        for background in (0, 1, 2, 4):
            directory = new_directory(root / f"repeat-{repeat}-background-{background}")
            outcome = run_case(resources, directory, background)
            report["trials"].append({"repeat": repeat, "background": background, "case": directory.name, **outcome})
            (root / "results.json").write_text(json.dumps(report, indent=2), encoding="utf8")
            print(f"{directory.name}: {outcome['status']}", flush=True)
    successful = [item for item in report["trials"] if item["status"] == "passed"]
    solo = [item["result"]["p0"]["click_to_complete_ms"] for item in successful if item["background"] == 0]
    baseline = statistics.median(solo) if solo else None
    limits = budgets["model_only_interaction"]
    report["comparisons"] = []
    for background in (0, 1, 2, 4):
        rows = [item["result"] for item in successful if item["background"] == background]
        if not rows or baseline is None:
            report["comparisons"].append({"background": background, "accepted": False,
                "reason": "no successful samples" if not rows else "no successful solo baseline"})
            continue
        p0 = [row["p0"]["click_to_complete_ms"] for row in rows]
        report["comparisons"].append({"background": background, "total": max(1, background + 1), "attempts": 3, "successes": len(rows),
            "p0_median_ms": statistics.median(p0), "p0_max_ms": max(p0), "relative": statistics.median(p0) / baseline,
            "private_max": max(row["peak_private_bytes"] for row in rows), "working_set_max": max(row["peak_working_set_bytes"] for row in rows),
            "accepted": len(rows) == 3 and statistics.median(p0) <= baseline * limits["p0_median_relative_max"] and max(p0) <= limits["p0_max_ms"]
                and all(row["peak_private_bytes"] <= limits["private_bytes_max"] and row["peak_working_set_bytes"] <= limits["working_set_bytes_max"] for row in rows),
            "percentiles": "not claimed"})
    directory = new_directory(root / "large-audio")
    large = run_case(resources, directory)
    report["large_audio"] = large
    (root / "results.json").write_text(json.dumps(report, indent=2), encoding="utf8")
    if any(item["status"] != "passed" for item in report["trials"]) or large["status"] != "passed":
        sys.exit(1)


if __name__ == "__main__":
    main()
