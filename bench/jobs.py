"""Writes the job list of the wQFM-rs benchmark (bench/PREREGISTRATION.md).

Usage (WSL): python3 bench/jobs.py <inputs folder> <jobs.tsv>

One row per run: job id, family (for the stopping rule of Amendment 1),
size (taxa), input (a file, or gen:<kind>:<taxa>:<seed> for S1, made by the
runner), method, repeat, true species tree (gene-tree data only).
"""

import os
import re
import sys

QUARTET_METHODS = ["wqfm-jar", "wqfm-rs"]
GENE_TREE_METHODS = [
    "wqfm-pipeline-java",
    "wqfm-pipeline-rs",
    "wqfm-pipeline-rs-1t",
    "wqfm-tree",
    "astral3",
    "astral4",
    "wastral",
    "tree-qmc",
]
REPEATS = 3  # H2: three timed repeats per quartet input and method
S1 = {"noisy": [25, 50, 75, 100, 125, 150], "sparse": [25, 50, 75, 100, 125, 150, 200]}
SEEDS = [1, 2, 3, 4, 5]


def taxa_in_quartets(path):
    names = set()
    with open(path) as f:
        for line in f:
            names.update(re.findall(r"[^(),;\s]+", line.split(";")[0]))
    return len(names)


def taxa_in_tree(path):
    with open(path) as f:
        return len(re.findall(r"[(,]\s*([^(),:;\s]+)", f.readline()))


def main():
    inputs, out = sys.argv[1], sys.argv[2]
    rows = []

    def add(family, size, source, methods, repeats, truth=""):
        for r in range(1, repeats + 1):
            for m in methods:
                name = os.path.basename(source.rstrip("/")).replace(":", "_")
                job = f"{family}__{name}__{m}__r{r}"
                rows.append([job, family, str(size), source, m, str(r), truth])

    for kind, sizes in S1.items():
        for size in sizes:
            for seed in SEEDS:
                add(f"S1-{kind}", size, f"gen:{kind}:{size}:{seed}", QUARTET_METHODS, REPEATS)
    for part in ["S2", "S3"]:
        folder = os.path.join(inputs, part)
        if os.path.isdir(folder):
            for f in sorted(os.listdir(folder)):
                path = os.path.join(folder, f)
                add(part, taxa_in_quartets(path), path, QUARTET_METHODS, REPEATS)
    for part in ["G1", "G2", "G3"]:
        folder = os.path.join(inputs, part)
        if not os.path.isdir(folder):
            continue
        reps = []
        for cond in sorted(os.listdir(folder)):
            for rep in sorted(os.listdir(os.path.join(folder, cond)), key=lambda s: int(s[1:])):
                d = os.path.join(folder, cond, rep)
                gt = os.path.join(d, "genetrees.tre")
                reps.append((taxa_in_tree(gt), cond, rep, gt, os.path.join(d, "true.tre")))
        # Small to large, so the stopping rule of Amendment 1 applies.
        for size, cond, rep, gt, truth in sorted(reps):
            family = part
            for m in GENE_TREE_METHODS:
                job = f"{part}__{cond}_{rep}__{m}__r1"
                rows.append([job, family, str(size), gt, m, "1", truth])

    with open(out, "w") as f:
        f.write("job\tfamily\tsize\tinput\tmethod\trepeat\ttruth\n")
        for row in rows:
            f.write("\t".join(row) + "\n")
    print(f"{len(rows)} jobs written to {out}")


if __name__ == "__main__":
    main()
