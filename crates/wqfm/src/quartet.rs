//! Weighted quartets: the input of the amalgamation.

/// A weighted quartet ab|cd on taxon ids, normalised so that `l[0] < l[1]`,
/// `r[0] < r[1]` and `l[0] < r[0]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quartet {
    /// The sister pair with the smallest taxon id.
    pub l: [u32; 2],
    /// The other sister pair.
    pub r: [u32; 2],
    /// The weight (support) of the quartet.
    pub weight: f64,
}

impl Quartet {
    /// The quartet a,b | c,d with the given weight.
    pub fn new(a: u32, b: u32, c: u32, d: u32, weight: f64) -> Self {
        let mut l = [a.min(b), a.max(b)];
        let mut r = [c.min(d), c.max(d)];
        if l[0] > r[0] {
            std::mem::swap(&mut l, &mut r);
        }
        Self { l, r, weight }
    }

    /// The four taxa in the order l0, l1, r0, r1.
    pub(crate) fn key(&self) -> [u32; 4] {
        [self.l[0], self.l[1], self.r[0], self.r[1]]
    }
}

/// The weighted quartets of a quartet a < b < c < d with weights for
/// ab|cd, ac|bd and ad|bc; weights that are not positive are left out.
pub fn quartets_of(q: [usize; 4], weights: [f64; 3]) -> impl Iterator<Item = Quartet> {
    let [a, b, c, d] = q.map(|x| x as u32);
    let tops = [(a, b, c, d), (a, c, b, d), (a, d, b, c)];
    tops.into_iter()
        .zip(weights)
        .filter(|(_, w)| *w > 0.0)
        .map(|((w1, x1, y1, z1), w)| Quartet::new(w1, x1, y1, z1, w))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalises_the_taxon_order() {
        let q = Quartet::new(5, 2, 1, 4, 3.0);
        assert_eq!((q.l, q.r), ([1, 4], [2, 5]));
        assert_eq!(q.key(), [1, 4, 2, 5]);
    }

    #[test]
    fn quartets_of_skips_zero_weights() {
        let qs: Vec<Quartet> = quartets_of([0, 1, 2, 3], [0.5, 0.0, 0.25]).collect();
        assert_eq!(qs.len(), 2);
        assert_eq!((qs[1].l, qs[1].r), ([0, 3], [1, 2]));
    }
}
