//! wQFM's text format for weighted quartets: one `((a,b),(c,d)); w` per line.

use std::collections::HashMap;
use std::fmt::Write as _;

use crate::gene_trees::QuartetCounts;
use crate::quartet::Quartet;

/// Quartet counts in wQFM's input format exactly as `generate_wqrts.py`
/// writes them: each pair in text order, the pair with the smaller first
/// name first, lines sorted as text, weight = count. With
/// `keep_unresolved`, unresolved 4-sets are written as the script does,
/// `((a,b,c,d)); n` with the names in text order; wQFM v1.4 refuses such
/// lines ("Can't handle polytomy") and stops.
pub fn counts_to_wqfm_input(c: &QuartetCounts, keep_unresolved: bool) -> String {
    let n = |t: u32| c.names[t as usize].as_str();
    let mut lines: Vec<(String, u32)> = Vec::new();
    for (&[a, b, x, y], counts) in &c.counts {
        let tops = [((a, b), (x, y)), ((a, x), (b, y)), ((a, y), (b, x))];
        if keep_unresolved && counts[3] > 0 {
            let mut four = [n(a), n(b), n(x), n(y)];
            four.sort_unstable();
            let key = format!("(({},{},{},{}));", four[0], four[1], four[2], four[3]);
            lines.push((key, counts[3]));
        }
        for (((p, q), (r, s)), &count) in tops.into_iter().zip(counts) {
            if count == 0 {
                continue;
            }
            let pair = |u: u32, v: u32| {
                let (u, v) = (n(u), n(v));
                if u <= v {
                    (u, v)
                } else {
                    (v, u)
                }
            };
            let (mut first, mut second) = (pair(p, q), pair(r, s));
            if second.0 < first.0 {
                std::mem::swap(&mut first, &mut second);
            }
            let key = format!("(({},{}),({},{}));", first.0, first.1, second.0, second.1);
            lines.push((key, count));
        }
    }
    lines.sort_unstable();
    let mut out = String::new();
    for (key, count) in lines {
        let _ = writeln!(out, "{key} {count}");
    }
    out
}

/// Weighted quartets in wQFM's input format.
pub fn to_wqfm_input(names: &[String], quartets: &[Quartet]) -> String {
    let mut out = String::new();
    for q in quartets {
        let n = |t: u32| names[t as usize].as_str();
        let _ = writeln!(
            out,
            "(({},{}),({},{})); {}",
            n(q.l[0]),
            n(q.l[1]),
            n(q.r[0]),
            n(q.r[1]),
            q.weight
        );
    }
    out
}

/// Reads wQFM's input format, mapping names to the indices of `names`
/// (new names are appended). As in wQFM v1.4, a line must have exactly
/// three `(` and three `)` (so `((a,b,c,d)); w` is refused) and reading
/// ends at the first empty line; text after it is an error here, because
/// wQFM would silently ignore it.
pub fn parse_wqfm_input(text: &str, names: &mut Vec<String>) -> Result<Vec<Quartet>, String> {
    let mut index: HashMap<String, u32> = names
        .iter()
        .enumerate()
        .map(|(i, n)| (n.clone(), i as u32))
        .collect();
    let mut out = Vec::new();
    let mut lines = text.lines().enumerate();
    for (n, line) in lines.by_ref() {
        let line = line.trim();
        if line.is_empty() {
            break;
        }
        let open = line.matches('(').count();
        let close = line.matches(')').count();
        if open != 3 || close != 3 {
            return Err(format!(
                "line {}: not a quartet ((a,b),(c,d)); w (an unresolved 4-set?)",
                n + 1
            ));
        }
        let cleaned: String = line
            .chars()
            .map(|c| {
                if c == '(' || c == ')' || c == ';' {
                    ' '
                } else {
                    c
                }
            })
            .collect();
        let parts: Vec<&str> = cleaned
            .split(|c: char| c == ',' || c.is_whitespace())
            .filter(|s| !s.is_empty())
            .collect();
        if parts.len() != 5 {
            return Err(format!("line {}: expected ((a,b),(c,d)); w", n + 1));
        }
        let mut id = |s: &str| -> u32 {
            if let Some(&i) = index.get(s) {
                return i;
            }
            let i = names.len() as u32;
            names.push(s.to_string());
            index.insert(s.to_string(), i);
            i
        };
        let t: Vec<u32> = parts[..4].iter().map(|s| id(s)).collect();
        let w: f64 = parts[4]
            .parse()
            .map_err(|_| format!("line {}: weight {} is not a number", n + 1, parts[4]))?;
        out.push(Quartet::new(t[0], t[1], t[2], t[3], w));
    }
    if let Some((n, _)) = lines.find(|(_, l)| !l.trim().is_empty()) {
        return Err(format!(
            "line {}: quartets after an empty line (wQFM would ignore them)",
            n + 1
        ));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_format_matches_wqfm() {
        let nm: Vec<String> = (0..4).map(|i| format!("t{i}")).collect();
        let text = to_wqfm_input(&nm, &[Quartet::new(0, 1, 2, 3, 34.0)]);
        assert_eq!(text, "((t0,t1),(t2,t3)); 34\n");
    }

    #[test]
    fn counts_are_written_in_script_order() {
        let trees: Vec<crate::newick::Tree> =
            ["((b,10),(2,a));", "((b,10),(2,a));", "((b,2),(10,a));"]
                .iter()
                .map(|t| crate::newick::Tree::parse(t).unwrap())
                .collect();
        let c = crate::gene_trees::count_quartets(&trees).unwrap();
        assert_eq!(
            counts_to_wqfm_input(&c, true),
            "((10,a),(2,b)); 1\n((10,b),(2,a)); 2\n"
        );
        let star = [crate::newick::Tree::parse("(b,10,2,a);").unwrap()];
        let c = crate::gene_trees::count_quartets(&star).unwrap();
        assert_eq!(counts_to_wqfm_input(&c, true), "((10,2,a,b)); 1\n");
        assert_eq!(counts_to_wqfm_input(&c, false), "");
    }

    #[test]
    fn bad_lines_are_errors() {
        assert!(parse_wqfm_input("((a,b),(c)); 1", &mut Vec::new()).is_err());
        assert!(parse_wqfm_input("((a,b),(c,d)); x", &mut Vec::new()).is_err());
        assert!(parse_wqfm_input("((a,b,c,d)); 1", &mut Vec::new()).is_err());
        assert!(parse_wqfm_input(
            "((a,b),(c,d)); 1

((a,c),(b,d)); 1",
            &mut Vec::new()
        )
        .is_err());
        assert_eq!(
            parse_wqfm_input(
                "((a,b),(c,d)); 1

",
                &mut Vec::new()
            )
            .unwrap()
            .len(),
            1
        );
    }
}
