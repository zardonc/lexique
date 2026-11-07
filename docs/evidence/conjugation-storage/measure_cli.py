"""Measure first and subsequent public CLI queries, including source validation."""

import json
from pathlib import Path
import platform
import statistics
import subprocess
import sys
import time

resource, output = map(Path, sys.argv[1:])
root = Path(__file__).resolve().parents[3]
query = "elles se sont abstenues"
samples = []
for _ in range(6):
    started = time.perf_counter()
    result = subprocess.run([sys.executable, "-X", "utf8", str(root / "scripts/conjugation_sample.py"),
                             "query", str(resource), query], capture_output=True, text=True, encoding="utf-8", timeout=60)
    assert result.returncode == 0, result.stderr
    response = json.loads(result.stdout)
    assert response["status"] == "found" and [table["lemma"] for table in response["tables"]] == ["s’abstenir"]
    samples.append(round((time.perf_counter() - started) * 1000, 3))
report = {"query": query, "python": sys.version.split()[0], "platform": platform.system(), "architecture": platform.machine(),
          "samples_ms": samples, "first_ms": samples[0], "warm_median_ms": statistics.median(samples[1:]),
          "warm_max_ms": max(samples[1:]), "budget_ms": 1700, "within_budget": max(samples) <= 1700,
          "boundary": "six fresh CLI processes including source validation and response serialization; no OS cache flush"}
output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report))
