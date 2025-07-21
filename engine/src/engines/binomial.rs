//! Cox-Ross-Rubinstein binomial tree.

use super::Exercise;
use crate::market::Market;
use crate::products::OptionKind;

/// Price of a call or put with `steps` time steps.
pub fn price(m: &Market, kind: OptionKind, exercise: Exercise, strike: f64, t: f64, steps: usize) -> f64 {
    let dt = t / steps as f64;
    let up = (m.vol * dt.sqrt()).exp();
    let p = (((m.rate - m.dividend) * dt).exp() - 1.0 / up) / (up - 1.0 / up);
    let disc = (-m.rate * dt).exp();
    let mut s: Vec<f64> = (0..=steps).map(|j| m.spot * up.powi(2 * j as i32 - steps as i32)).collect();
    let mut v: Vec<f64> = s.iter().map(|&x| kind.payoff(x, strike)).collect();
    for n in (0..steps).rev() {
        for j in 0..=n {
            s[j] = s[j + 1] / up;
            let cont = disc * (p * v[j + 1] + (1.0 - p) * v[j]);
            v[j] = match exercise {
                Exercise::European => cont,
                Exercise::American => cont.max(kind.payoff(s[j], strike)),
            };
        }
    }
    v[0]
}

/// Average of n and n + 1 steps (removes the odd-even oscillation), extrapolated between n and 2n.
pub fn price_extrapolated(m: &Market, kind: OptionKind, exercise: Exercise, strike: f64, t: f64, steps: usize) -> f64 {
    let smooth = |n: usize| 0.5 * (price(m, kind, exercise, strike, t, n) + price(m, kind, exercise, strike, t, n + 1));
    2.0 * smooth(2 * steps) - smooth(steps)
}
