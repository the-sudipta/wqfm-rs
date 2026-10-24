import random, sys
out = sys.argv[1]
def tree(names, rng, poly, lengths):
    nodes = [n for n in names]
    while len(nodes) > 3:
        k = 3 if (poly and rng.random() < 0.15 and len(nodes) > 4) else 2
        pick = rng.sample(range(len(nodes)), k)
        sub = [nodes[i] for i in pick]
        for i in sorted(pick, reverse=True): nodes.pop(i)
        if lengths:
            sub = [f"{s}:{rng.random():.4f}" for s in sub]
            nodes.append("(" + ",".join(sub) + f"){rng.randint(1,100)}")
        else:
            nodes.append("(" + ",".join(sub) + ")")
    return "(" + ",".join(nodes) + ");"
specs = [("s1_12tax", 12, 40, 0, False, False), ("s2_25tax_missing_poly_len", 25, 60, 4, True, True),
         ("s3_40tax_mixed_names", 40, 20, 6, True, False), ("s4_8tax_many", 8, 500, 1, True, True)]
for name, m, g, drop, poly, lengths in specs:
    rng = random.Random(sum(map(ord, name)))
    base = [str(i) for i in range(1, m // 2)] + [f"tx_{chr(65 + i % 26)}{i}" for i in range(m - m // 2 + 1)]
    with open(f"{out}/{name}.tre", "w", newline="\n") as f:
        for _ in range(g):
            names = rng.sample(base, len(base) - rng.randint(0, drop))
            f.write(tree(names, rng, poly, lengths) + "\n")
