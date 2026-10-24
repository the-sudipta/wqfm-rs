//! Division of a level into two sub-problems with a shared dummy taxon.

use std::collections::HashMap;

use crate::level::Level;
use crate::partition::{sides_of, status, Status, LEFT, RIGHT};
use crate::quartet::Quartet;

/// Each side gets the dummy for the other side. Blank quartets go to their
/// side; deferred quartets go to the side of their three taxa with the
/// fourth replaced by the dummy, and equal new quartets are merged with the
/// mean weight. Satisfied and violated quartets are dropped.
pub(crate) fn divide(level: Level, side: &[i8], dummy: u32) -> (Level, Level) {
    let mut left = Level {
        taxa: Vec::new(),
        quartets: Vec::new(),
    };
    let mut right = Level {
        taxa: Vec::new(),
        quartets: Vec::new(),
    };
    for &t in &level.taxa {
        if side[t as usize] == LEFT {
            left.taxa.push(t);
        } else {
            right.taxa.push(t);
        }
    }
    left.taxa.push(dummy);
    right.taxa.push(dummy);
    // New dummy quartets in order of first appearance: key -> (index, count).
    let mut seen: HashMap<[u32; 4], (bool, usize, u32)> = HashMap::new();
    let mut dummy_quartets: Vec<(bool, Quartet)> = Vec::new();
    for q in &level.quartets {
        let s = sides_of(side, q);
        match status(s) {
            Status::Blank => {
                if s[0] == LEFT {
                    left.quartets.push(*q);
                } else {
                    right.quartets.push(*q);
                }
            }
            Status::Deferred => {
                let sum: i8 = s.iter().sum();
                let common = if sum < 0 { LEFT } else { RIGHT };
                let mut k = q.key();
                // The (last) taxon on the other side becomes the dummy.
                let odd = (0..4).rev().find(|&i| s[i] != common).expect("deferred");
                k[odd] = dummy;
                let nq = Quartet::new(k[0], k[1], k[2], k[3], q.weight);
                let to_left = common == LEFT;
                match seen.get_mut(&nq.key()) {
                    None => {
                        seen.insert(nq.key(), (to_left, dummy_quartets.len(), 1));
                        dummy_quartets.push((to_left, nq));
                    }
                    Some((_, idx, count)) => {
                        let old = dummy_quartets[*idx].1.weight;
                        let c = *count as f64;
                        dummy_quartets[*idx].1.weight = (c * old + q.weight) / (c + 1.0);
                        *count += 1;
                    }
                }
            }
            _ => {}
        }
    }
    for (to_left, q) in dummy_quartets {
        if to_left {
            left.quartets.push(q);
        } else {
            right.quartets.push(q);
        }
    }
    (left, right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deferred_quartets_merge_with_the_mean_weight() {
        // Sides: 0,1,2 left; 3,4 right. Both quartets become 0,1|2,X.
        let level = Level {
            taxa: vec![0, 1, 2, 3, 4],
            quartets: vec![
                Quartet::new(0, 1, 2, 3, 2.0),
                Quartet::new(0, 1, 2, 4, 4.0),
                Quartet::new(0, 2, 3, 4, 9.0),
            ],
        };
        let side = [LEFT, LEFT, LEFT, RIGHT, RIGHT];
        let (l, r) = divide(level, &side, 9);
        assert_eq!(l.taxa, vec![0, 1, 2, 9]);
        assert_eq!(r.taxa, vec![3, 4, 9]);
        assert_eq!(l.quartets.len(), 1);
        assert_eq!(l.quartets[0].key(), [0, 1, 2, 9]);
        assert_eq!(l.quartets[0].weight, 3.0);
        assert!(r.quartets.is_empty());
    }
}
