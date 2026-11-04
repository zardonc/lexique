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


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("resources", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--child", type=int, choices=(0, 1, 2, 4))
    parser.add_argument("--large", action="store_true")
    args = parser.parse_args()
    resources = args.resources.resolve()
    if args.child is not None:
        print(json.dumps(trial(resources, args.output.resolve(), args.child, "on", "fixed")))
        return
    if args.large:
        print(json.dumps(large_audio(resources, args.output.resolve())))
        return
    root = new_directory(args.output)
    budgets = json.loads((REPO / "docs/evidence/issue-25/budgets.json").read_text())
    report = {"budgets_sha256": hashlib.sha256((REPO / "docs/evidence/issue-25/budgets.json").read_bytes()).hexdigest(),
        "provider": "cpu", "steps": 8, "bitrate": 48000, "trials": [], "boundary": "model only; actual product P0/IPC/playback separately recorded"}
    command = [str(resources / "runtime/python.exe"), "-I", "-B", "-X", "utf8", "-c",
        f"import sys,runpy;sys.path.insert(0,{str(REPO / 'scripts')!r});runpy.run_path({str(Path(__file__).resolve())!r},run_name='__main__')"]
    for repeat in range(3):
        for background in (0, 1, 2, 4):
            directory = new_directory(root / f"repeat-{repeat}-background-{background}")
            outcome = execute(command + [str(resources), str(directory), "--child", str(background)], directory / "stdout.json")
            if outcome["status"] == "passed":
                outcome["result"] = json.loads((directory / "stdout.json").read_text(encoding="utf8"))
            report["trials"].append({"repeat": repeat, "background": background, "case": directory.name, **outcome})
            (root / "results.json").write_text(json.dumps(report, indent=2), encoding="utf8")
            print(f"{directory.name}: {outcome['status']}", flush=True)
    successful = [item for item in report["trials"] if item["status"] == "passed"]
    baseline = statistics.median(item["result"]["p0"]["click_to_complete_ms"] for item in successful if item["background"] == 0)
    limits = budgets["model_only_interaction"]
    report["comparisons"] = []
    for background in (0, 1, 2, 4):
        rows = [item["result"] for item in successful if item["background"] == background]
        if not rows:
            report["comparisons"].append({"background": background, "accepted": False, "reason": "no successful samples"})
            continue
        p0 = [row["p0"]["click_to_complete_ms"] for row in rows]
        report["comparisons"].append({"background": background, "total": max(1, background + 1), "attempts": 3, "successes": len(rows),
            "p0_median_ms": statistics.median(p0), "p0_max_ms": max(p0), "relative": statistics.median(p0) / baseline,
            "private_max": max(row["peak_private_bytes"] for row in rows), "working_set_max": max(row["peak_working_set_bytes"] for row in rows),
            "accepted": len(rows) == 3 and statistics.median(p0) <= baseline * limits["p0_median_relative_max"] and max(p0) <= limits["p0_max_ms"]
                and all(row["peak_private_bytes"] <= limits["private_bytes_max"] and row["peak_working_set_bytes"] <= limits["working_set_bytes_max"] for row in rows),
            "percentiles": "not claimed"})
    directory = new_directory(root / "large-audio")
    large = execute(command + [str(resources), str(directory), "--large"], directory / "stdout.json")
    if large["status"] == "passed":
        large["result"] = json.loads((directory / "stdout.json").read_text(encoding="utf8"))
    report["large_audio"] = large
    (root / "results.json").write_text(json.dumps(report, indent=2), encoding="utf8")
    if any(item["status"] != "passed" for item in report["trials"]) or large["status"] != "passed":
        sys.exit(1)


if __name__ == "__main__":
    main()
