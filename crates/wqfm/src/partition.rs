//! Sides of a bipartition, the status of a quartet under it, and the
//! partition score: satisfied weight minus beta times violated weight.

use crate::quartet::Quartet;

pub(crate) const LEFT: i8 = -1;
pub(crate) const RIGHT: i8 = 1;
pub(crate) const UNASSIGNED: i8 = 0;

/// How a bipartition treats one quartet.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Status {
    /// The two sister pairs are on different sides.
    Satisfied,
    /// Two taxa on each side, with sisters split.
    Violated,
    /// Three taxa on one side.
    Deferred,
    /// All four taxa on one side.
    Blank,
}

/// The status of a quartet from the sides of its four taxa (l0, l1, r0, r1).
pub(crate) fn status(s: [i8; 4]) -> Status {
    let sum: i8 = s.iter().sum();
    if sum.abs() == 4 {
        Status::Blank
    } else if sum.abs() == 2 {
        Status::Deferred
    } else if s[0] == s[1] && s[2] == s[3] && s[0] != s[2] {
        Status::Satisfied
    } else {
        Status::Violated
    }
}

/// The sides of the four taxa of `q`.
pub(crate) fn sides_of(side: &[i8], q: &Quartet) -> [i8; 4] {
    q.key().map(|t| side[t as usize])
}

/// Satisfied and violated weight and the number of satisfied quartets.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Tally {
    pub(crate) n_satisfied: i64,
    pub(crate) w_satisfied: f64,
    pub(crate) w_violated: f64,
}

impl Tally {
    /// Adds a quartet of weight `w` with status `s`.
    pub(crate) fn add(&mut self, w: f64, s: Status) {
        match s {
            Status::Satisfied => {
                self.n_satisfied += 1;
                self.w_satisfied += w;
            }
            Status::Violated => self.w_violated += w,
            _ => {}
        }
    }

    /// The partition score with violated weight scaled by `beta`.
    pub(crate) fn score(&self, beta: f64) -> f64 {
        self.w_satisfied - beta * self.w_violated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_of_each_case() {
        assert_eq!(status([LEFT, LEFT, RIGHT, RIGHT]), Status::Satisfied);
        assert_eq!(status([LEFT, RIGHT, LEFT, RIGHT]), Status::Violated);
        assert_eq!(status([LEFT, LEFT, LEFT, RIGHT]), Status::Deferred);
        assert_eq!(status([RIGHT; 4]), Status::Blank);
    }

    #[test]
    fn tally_scores_satisfied_minus_violated() {
        let mut t = Tally::default();
        t.add(3.0, Status::Satisfied);
        t.add(1.0, Status::Violated);
        t.add(9.0, Status::Deferred);
        assert_eq!((t.n_satisfied, t.score(1.0), t.score(2.0)), (1, 2.0, 1.0));
    }
}
