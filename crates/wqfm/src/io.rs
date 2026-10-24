//! wQFM's text format for weighted quartets: one `((a,b),(c,d)); w` per line.

use std::collections::HashMap;
use std::fmt::Write as _;

use crate::quartet::Quartet;

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
/// (new names are appended).
pub fn parse_wqfm_input(text: &str, names: &mut Vec<String>) -> Result<Vec<Quartet>, String> {
    let mut index: HashMap<String, u32> = names
        .iter()
        .enumerate()
        .map(|(i, n)| (n.clone(), i as u32))
        .collect();
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
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
    fn bad_lines_are_errors() {
        assert!(parse_wqfm_input("((a,b),(c)); 1", &mut Vec::new()).is_err());
        assert!(parse_wqfm_input("((a,b),(c,d)); x", &mut Vec::new()).is_err());
    }
}
