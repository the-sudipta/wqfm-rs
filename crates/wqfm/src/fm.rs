//! Fiduccia–Mattheyses refinement of a bipartition (wQFM v1.4 rules).

use crate::level::Level;
use crate::options::Options;
use crate::partition::{sides_of, status, Status, Tally, LEFT};

/// The tally of every quartet of the level under `side`.
fn whole_tally(level: &Level, side: &[i8]) -> Tally {
    let mut t = Tally::default();
    for q in level.quartets.iter() {
        t.add(q.weight, status(sides_of(side, q)));
    }
    t
}

/// The number of taxa of the level on the left side.
fn count_left(level: &Level, side: &[i8]) -> usize {
    level
        .taxa
        .iter()
        .filter(|&&t| side[t as usize] == LEFT)
        .count()
}

/// True if a side has fewer than two taxa.
#[cfg(test)]
fn is_trivial(level: &Level, side: &[i8]) -> bool {
    let left = count_left(level, side);
    left < 2 || level.taxa.len() - left < 2
}

/// True if `a` and `b` split the taxa of the level the same way.
fn same_bipartition(level: &Level, a: &[i8], b: &[i8]) -> bool {
    let same = level.taxa.iter().all(|&t| a[t as usize] == b[t as usize]);
    let flipped = level.taxa.iter().all(|&t| a[t as usize] == -b[t as usize]);
    same || flipped
}

/// Refines `side` in place. In each pass every free taxon is moved
/// hypothetically (not if a side would keep fewer than two taxa); the one
/// with the highest gain moves and is locked (ties: most satisfied quartets
/// after the move, then the last such taxon). After all taxa are locked,
/// the prefix of moves with the highest cumulative gain is kept. Iterations
/// repeat while that gain is positive and the bipartition changes.
pub(crate) fn fm(level: &Level, side: &mut Vec<i8>, options: &Options) {
    let beta = options.beta;
    let max_id = side.len() - 1;
    // Relevant quartets per taxon, in quartet order.
    let mut relevant: Vec<Vec<u32>> = vec![Vec::new(); max_id + 1];
    for (i, q) in level.quartets.iter().enumerate() {
        for t in q.key() {
            relevant[t as usize].push(i as u32);
        }
    }
    let mut tally = whole_tally(level, side);
    let mut previous_best_gain = 0.0;
    let mut previous_side: Option<Vec<i8>> = None;
    let mut iterations = 0u32;
    loop {
        if iterations > options.max_iterations {
            break;
        }
        iterations += 1;
        let start_side = side.clone();
        let start_tally = tally;
        // One iteration: passes until every taxon is locked.
        let mut locked = vec![false; max_id + 1];
        let mut current = side.clone();
        // Taxa on the left side of `current`, kept up to date with each move.
        let mut n_left = count_left(level, &current);
        let mut current_tally = tally;
        // (gain, side after the move, tally after the move)
        let mut passes: Vec<(f64, Vec<i8>, Tally)> = Vec::new();
        loop {
            let mut best: Option<(f64, i64, u32, Tally)> = None;
            let mut any_candidate = false;
            for &t in &level.taxa {
                if locked[t as usize] {
                    continue;
                }
                // Moving t must leave at least two taxa on each side.
                let left_after = if current[t as usize] == LEFT {
                    n_left - 1
                } else {
                    n_left + 1
                };
                if left_after < 2 || level.taxa.len() - left_after < 2 {
                    continue;
                }
                if relevant[t as usize].is_empty() {
                    locked[t as usize] = true;
                    continue;
                }
                let mut before = Tally::default();
                let mut after = Tally::default();
                for &qi in &relevant[t as usize] {
                    let q = &level.quartets[qi as usize];
                    let s0 = sides_of(&current, q);
                    let st0 = status(s0);
                    before.add(q.weight, st0);
                    if st0 == Status::Deferred {
                        // Moving one taxon of a deferred quartet can give
                        // any status; others become deferred.
                        let s1 = q.key().map(|x| {
                            if x == t {
                                -current[x as usize]
                            } else {
                                current[x as usize]
                            }
                        });
                        after.add(q.weight, status(s1));
                    }
                }
                let gain = after.score(beta) - before.score(beta);
                let whole = Tally {
                    n_satisfied: current_tally.n_satisfied + after.n_satisfied - before.n_satisfied,
                    w_satisfied: current_tally.w_satisfied + after.w_satisfied - before.w_satisfied,
                    w_violated: current_tally.w_violated + after.w_violated - before.w_violated,
                };
                any_candidate = true;
                // Highest gain (Double order of Java's TreeMap, so −0 < +0);
                // ties: most satisfied quartets; then the last taxon.
                let better = match &best {
                    None => true,
                    Some((g, ns, _, _)) => match gain.total_cmp(g) {
                        std::cmp::Ordering::Greater => true,
                        std::cmp::Ordering::Less => false,
                        std::cmp::Ordering::Equal => whole.n_satisfied >= *ns,
                    },
                };
                if better {
                    best = Some((gain, whole.n_satisfied, t, whole));
                }
            }
            if !any_candidate || best.is_none() {
                // Every remaining move would make a side trivial: lock all.
                break;
            }
            let (gain, _, t, whole) = best.expect("checked");
            locked[t as usize] = true;
            current[t as usize] = -current[t as usize];
            if current[t as usize] == LEFT {
                n_left += 1;
            } else {
                n_left -= 1;
            }
            current_tally = whole;
            passes.push((gain, current.clone(), whole));
            if level.taxa.iter().all(|&x| locked[x as usize]) {
                break;
            }
        }
        if passes.is_empty() {
            *side = start_side;
            tally = start_tally;
            break;
        }
        let mut best_cumulative = f64::from(i32::MIN);
        let mut cumulative = 0.0;
        let mut best_index = 0;
        for (i, p) in passes.iter().enumerate() {
            cumulative += p.0;
            if cumulative > best_cumulative {
                best_cumulative = cumulative;
                best_index = i;
            }
        }
        let chosen = &passes[best_index];
        if best_cumulative > 0.0 {
            if best_cumulative == previous_best_gain {
                if let Some(prev) = &previous_side {
                    if same_bipartition(level, &chosen.1, prev) {
                        *side = start_side;
                        tally = start_tally;
                        break;
                    }
                }
            }
            *side = chosen.1.clone();
            tally = chosen.2;
            previous_best_gain = best_cumulative;
            previous_side = Some(side.clone());
        } else {
            *side = start_side;
            tally = start_tally;
            break;
        }
    }
    let _ = tally;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::partition::RIGHT;
    use crate::quartet::Quartet;

    #[test]
    fn fm_fixes_a_bad_start() {
        let level = Level {
            taxa: vec![0, 1, 2, 3, 4],
            quartets: vec![Quartet::new(0, 1, 2, 3, 1.0)].into(),
        };
        let mut side = vec![LEFT, RIGHT, LEFT, RIGHT, RIGHT];
        fm(&level, &mut side, &Options::default());
        let t = whole_tally(&level, &side);
        assert_eq!(t.score(1.0), 1.0);
        assert!(!is_trivial(&level, &side));
    }

    #[test]
    fn mirror_sides_are_the_same_bipartition() {
        let level = Level {
            taxa: vec![0, 1],
            quartets: Vec::new().into(),
        };
        assert!(same_bipartition(&level, &[LEFT, RIGHT], &[RIGHT, LEFT]));
        assert!(!same_bipartition(&level, &[LEFT, RIGHT], &[LEFT, LEFT]));
    }
}
