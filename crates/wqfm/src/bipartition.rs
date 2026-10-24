//! The greedy initial bipartition of a level (wQFM v1.4).

use crate::level::Level;
use crate::partition::{LEFT, RIGHT, UNASSIGNED};

/// Quartets in descending weight (input order on ties) assign their
/// unassigned taxa, keeping sisters together and the two sister pairs
/// apart; taxa left over are spread to balance the sides. The result is
/// indexed by taxon id up to `max_id`.
pub(crate) fn initial_bipartition(level: &Level, max_id: usize) -> Vec<i8> {
    let mut side = vec![UNASSIGNED; max_id + 1];
    let mut order: Vec<usize> = (0..level.quartets.len()).collect();
    // Descending weight; equal weights keep their order (stable sort).
    order.sort_by(|&a, &b| {
        level.quartets[b]
            .weight
            .total_cmp(&level.quartets[a].weight)
    });
    let (mut n_left, mut n_right) = (0i64, 0i64);
    let put = |side: &mut Vec<i8>, t: u32, v: i8, n_left: &mut i64, n_right: &mut i64| {
        side[t as usize] = v;
        if v == LEFT {
            *n_left += 1;
        } else {
            *n_right += 1;
        }
    };
    for i in order {
        let q = &level.quartets[i];
        let [q1, q2, q3, q4] = q.key();
        let mut s = [q1, q2, q3, q4].map(|t| side[t as usize]);
        if s.iter().all(|&v| v == UNASSIGNED) {
            put(&mut side, q1, LEFT, &mut n_left, &mut n_right);
            put(&mut side, q2, LEFT, &mut n_left, &mut n_right);
            put(&mut side, q3, RIGHT, &mut n_left, &mut n_right);
            put(&mut side, q4, RIGHT, &mut n_left, &mut n_right);
            continue;
        }
        if s[0] == UNASSIGNED {
            let v = if s[1] != UNASSIGNED {
                s[1]
            } else if s[2] != UNASSIGNED {
                -s[2]
            } else if s[3] != UNASSIGNED {
                -s[3]
            } else {
                UNASSIGNED
            };
            if v != UNASSIGNED {
                put(&mut side, q1, v, &mut n_left, &mut n_right);
                s[0] = v;
            }
        }
        if s[1] == UNASSIGNED {
            let v = if s[0] == LEFT { LEFT } else { RIGHT };
            put(&mut side, q2, v, &mut n_left, &mut n_right);
            s[1] = v;
        }
        if s[2] == UNASSIGNED {
            let v = if s[3] != UNASSIGNED {
                s[3]
            } else if s[0] == RIGHT {
                LEFT
            } else {
                RIGHT
            };
            put(&mut side, q3, v, &mut n_left, &mut n_right);
            s[2] = v;
        }
        if s[3] == UNASSIGNED {
            let v = if s[2] == LEFT { LEFT } else { RIGHT };
            put(&mut side, q4, v, &mut n_left, &mut n_right);
        }
    }
    // Leftover taxa, balancing the sides.
    let mut flag = 0i64;
    for &t in &level.taxa {
        if side[t as usize] != UNASSIGNED {
            continue;
        }
        if n_left < n_right {
            flag = 2;
        } else if n_left > n_right {
            flag = 1;
        } else {
            flag += 1;
        }
        if flag % 2 == 0 {
            put(&mut side, t, LEFT, &mut n_left, &mut n_right);
        } else {
            put(&mut side, t, RIGHT, &mut n_left, &mut n_right);
        }
    }
    side
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quartet::Quartet;

    #[test]
    fn heaviest_quartet_splits_its_pairs() {
        let level = Level {
            taxa: vec![0, 1, 2, 3, 4],
            quartets: vec![Quartet::new(0, 2, 1, 3, 1.0), Quartet::new(0, 1, 2, 3, 5.0)],
        };
        let side = initial_bipartition(&level, 4);
        assert_eq!(&side[..4], &[LEFT, LEFT, RIGHT, RIGHT]);
        assert_ne!(side[4], UNASSIGNED);
    }
}
