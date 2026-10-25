"""Analysis of the wQFM-rs benchmark, as fixed in bench/PREREGISTRATION.md.

Usage (WSL): python3 bench/analyse.py <jobs.tsv> <results.tsv> <wqfm-rs> <out folder>

Writes h1.tsv, h2.tsv, h3.tsv, accuracy.tsv, tests.tsv, summary.md and
fig_h2.png. Every number comes from the raw results; nothing is typed in.
"""

import csv
import os
import statistics
import subprocess
import sys
from collections import defaultdict

from scipy.stats import wilcoxon


def read(path):
    with open(path, newline="") as f:
        return list(csv.DictReader(f, delimiter="\t"))


def rf(rs, a, b):
    """Normalised RF distance between two tree files (None if not comparable)."""
    p = subprocess.run([rs, "compare", a, b], capture_output=True, text=True)
    if p.returncode != 0:
        return None
    return float(p.stdout.split()[1])


def holm(pvalues):
    """Holm-adjusted p-values, in the given order."""
    order = sorted(range(len(pvalues)), key=lambda i: pvalues[i])
    adjusted, running = [0.0] * len(pvalues), 0.0
    for rank, i in enumerate(order):
        running = max(running, (len(pvalues) - rank) * pvalues[i])
        adjusted[i] = min(1.0, running)
    return adjusted


def iqr(values):
    if len(values) < 2:
        return (values[0], values[0]) if values else (None, None)
    q = statistics.quantiles(values, n=4)
    return q[0], q[2]


def main():
    jobs_path, results_path, rs, out = sys.argv[1:5]
    os.makedirs(out, exist_ok=True)
    jobs = {j["job"]: j for j in read(jobs_path)}
    results = {r["job"]: r for r in read(results_path)}
    runs = os.path.join(os.path.dirname(results_path), "runs")
    tree = lambda job: os.path.join(runs, job, "tree.nwk")
    ok = lambda job: results.get(job, {}).get("status") == "ok"
    lines = ["# wQFM-rs benchmark: results", ""]

    # H1: identity of wqfm-rs and the jar (quartet inputs, first repeat) and
    # of the two pipelines (gene-tree inputs).
    h1 = []
    for job, j in jobs.items():
        if j["repeat"] != "1":
            continue
        pair = {"wqfm-rs": "wqfm-jar", "wqfm-pipeline-rs": "wqfm-pipeline-java"}.get(j["method"])
        if not pair:
            continue
        other = job.replace(f"__{j['method']}__", f"__{pair}__")
        if not (ok(job) and ok(other)):
            h1.append([job, other, "not both finished", "", ""])
            continue
        d = rf(rs, tree(job), tree(other))
        same_score = results[job]["score"] == results[other]["score"]
        same = d == 0.0 and (same_score or not j["family"].startswith("S"))
        h1.append([job, other, "identical" if same else "DIFFERENT", d, same_score])
    with open(os.path.join(out, "h1.tsv"), "w") as f:
        f.write("rust_job\tjava_job\tresult\tnrf\tsame_score\n")
        for row in h1:
            f.write("\t".join(str(x) for x in row) + "\n")
    compared = [r for r in h1 if r[2] != "not both finished"]
    differ = [r for r in compared if r[2] == "DIFFERENT"]
    lines += [
        "## H1 (identity)",
        f"- compared: {len(compared)}; different: {len(differ)}; "
        f"not both finished: {len(h1) - len(compared)}",
        f"- H1 {'holds' if not differ else 'FAILS'}",
        "",
    ]

    # H2: cost of the amalgamation on quartet inputs with >= 100,000 quartets.
    by_input = defaultdict(lambda: defaultdict(list))
    for job, j in jobs.items():
        if j["method"] in ("wqfm-rs", "wqfm-jar") and ok(job):
            r = results[job]
            by_input[(j["family"], j["input"])][j["method"]].append(
                (float(r["seconds"]), float(r["max_rss_kb"]), int(r["quartets"]))
            )
    h2, time_ratios, mem_ratios, only_rust, only_java = [], [], [], 0, 0
    for key, methods in sorted(by_input.items()):
        rust, java = methods.get("wqfm-rs", []), methods.get("wqfm-jar", [])
        if rust and not java:
            only_rust += 1
        if java and not rust:
            only_java += 1
        if not (rust and java):
            continue
        t_r = statistics.median(x[0] for x in rust)
        t_j = statistics.median(x[0] for x in java)
        m_r = statistics.median(x[1] for x in rust)
        m_j = statistics.median(x[1] for x in java)
        quartets = (rust or java)[0][2]
        row = [key[0], key[1], quartets, t_j, t_r, t_j / t_r, m_j, m_r, m_j / m_r]
        h2.append(row)
        if quartets is not None and quartets >= 100_000:
            time_ratios.append(t_j / t_r)
            mem_ratios.append(m_j / m_r)
    with open(os.path.join(out, "h2.tsv"), "w") as f:
        f.write("family\tinput\tquartets\tjava_s\trust_s\ttime_ratio\tjava_kb\trust_kb\tmemory_ratio\n")
        for row in h2:
            f.write("\t".join(str(x) for x in row) + "\n")
    tm = statistics.median(time_ratios) if time_ratios else None
    mm = statistics.median(mem_ratios) if mem_ratios else None
    passed = (tm or 0) >= 5 or (mm or 0) >= 3 or only_rust > 0
    if not time_ratios and not only_rust:
        verdict = "cannot be evaluated (no input with at least 100,000 quartets finished)"
    else:
        verdict = "holds" if passed else "FAILS"
    lines += [
        "## H2 (amalgamation, inputs with at least 100,000 quartets)",
        f"- inputs: {len(time_ratios)}; median time ratio Java/Rust {tm} (IQR {iqr(time_ratios)}); "
        f"median memory ratio {mm} (IQR {iqr(mem_ratios)})",
        f"- inputs finished only by wQFM-rs: {only_rust}; only by the jar: {only_java}",
        f"- H2 (and the go/no-go gate) {verdict}",
        "",
    ]

    # H3: the pipelines from gene trees.
    h3 = []
    for job, j in jobs.items():
        if j["method"] != "wqfm-pipeline-rs":
            continue
        other = job.replace("__wqfm-pipeline-rs__", "__wqfm-pipeline-java__")
        if ok(job) and ok(other):
            a, b = float(results[other]["seconds"]), float(results[job]["seconds"])
            h3.append([j["family"], job, a, b, a / b])
    with open(os.path.join(out, "h3.tsv"), "w") as f:
        f.write("family\tjob\tjava_pipeline_s\trust_pipeline_s\tratio\n")
        for row in h3:
            f.write("\t".join(str(x) for x in row) + "\n")
    ratios = [r[4] for r in h3]
    lines += [
        "## H3 (pipeline from gene trees)",
        f"- runs: {len(ratios)}; median ratio {statistics.median(ratios) if ratios else None}; "
        f"range {(min(ratios), max(ratios)) if ratios else None}; "
        f"rust faster on all: {all(r > 1 for r in ratios) if ratios else None}",
        "",
    ]

    # D: accuracy (nRF to the true species tree) and tests against wQFM-rs.
    acc = defaultdict(dict)  # (family, condition, replicate) -> method -> nrf
    for job, j in jobs.items():
        if j["truth"] and ok(job):
            fam, cond_rep = j["family"], job.split("__")[1]
            acc[(fam, cond_rep)][j["method"]] = rf(rs, tree(job), j["truth"])
    with open(os.path.join(out, "accuracy.tsv"), "w") as f:
        f.write("family\treplicate\tmethod\tnrf\n")
        for (fam, cr), methods in sorted(acc.items()):
            for m, v in sorted(methods.items()):
                f.write(f"{fam}\t{cr}\t{m}\t{v}\n")
    tests = []
    for fam in sorted({k[0] for k in acc}):
        rows = []
        others = sorted({m for k, v in acc.items() if k[0] == fam for m in v} - {"wqfm-pipeline-rs"})
        for m in others:
            pairs = [
                (v["wqfm-pipeline-rs"], v[m])
                for k, v in acc.items()
                if k[0] == fam and "wqfm-pipeline-rs" in v and m in v
            ]
            diffs = [a - b for a, b in pairs]
            if len(pairs) < 2 or all(d == 0 for d in diffs):
                p = 1.0
            else:
                p = wilcoxon([a for a, _ in pairs], [b for _, b in pairs], zero_method="pratt").pvalue
            rows.append([fam, m, len(pairs), statistics.mean(a for a, _ in pairs) if pairs else None,
                         statistics.mean(b for _, b in pairs) if pairs else None, p])
        for row, adj in zip(rows, holm([r[5] for r in rows])):
            tests.append(row + [adj])
    with open(os.path.join(out, "tests.tsv"), "w") as f:
        f.write("family\tmethod\tpairs\tmean_nrf_wqfm_rs\tmean_nrf_method\tp\tp_holm\n")
        for row in tests:
            f.write("\t".join(str(x) for x in row) + "\n")
    lines += ["## D (accuracy, descriptive)", "- see accuracy.tsv and tests.tsv", ""]

    # Failures by method, family and size.
    fails = defaultdict(int)
    for job, r in results.items():
        if r["status"] != "ok":
            j = jobs[job]
            fails[(j["method"], j["family"], j["size"], r["status"])] += 1
    lines += ["## Runs that did not finish", "", "| method | family | taxa | status | runs |", "|---|---|---|---|---|"]
    for (m, fam, s, st), n in sorted(fails.items()):
        lines.append(f"| {m} | {fam} | {s} | {st} | {n} |")
    with open(os.path.join(out, "summary.md"), "w") as f:
        f.write("\n".join(lines) + "\n")
    plot(h2, os.path.join(out, "fig_h2.png"))
    print("\n".join(lines))


def plot(h2, path):
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    rows = [r for r in h2 if r[2]]
    if not rows:
        return
    fig, ax = plt.subplots(1, 2, figsize=(9, 3.6))
    for i, (j, r, label) in enumerate([(3, 4, "wall time (s)"), (6, 7, "peak memory (KB)")]):
        ax[i].loglog([x[2] for x in rows], [x[j] for x in rows], "o", label="wQFM v1.4 (Java)")
        ax[i].loglog([x[2] for x in rows], [x[r] for x in rows], "s", label="wQFM-rs")
        ax[i].set_xlabel("weighted quartets")
        ax[i].set_ylabel(label)
    ax[0].legend()
    fig.tight_layout()
    fig.savefig(path, dpi=200)


if __name__ == "__main__":
    main()
