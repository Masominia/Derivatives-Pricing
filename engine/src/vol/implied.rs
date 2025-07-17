//! Implied volatility by Halley iterations on the logarithm of the normalised Black price, with a bracket.

use crate::math::normal::{cdf, inverse_cdf};
use crate::products::OptionKind;

const INV_SQRT_2PI: f64 = 0.398_942_280_401_432_7;

/// Normalised out-of-the-money price for x = ln(F / K) <= 0 and total volatility s.
pub fn normalised_price(x: f64, s: f64) -> f64 {
    let (h, t) = (x / s, 0.5 * s);
    (0.5 * x).exp() * cdf(h + t) - (-0.5 * x).exp() * cdf(h - t)
}

fn normalised_vega(x: f64, s: f64) -> f64 {
    let (h, t) = (x / s, 0.5 * s);
    INV_SQRT_2PI * (-0.5 * (h * h + t * t)).exp()
}

/// Total volatility s with normalised_price(x, s) = beta, for x <= 0. None when beta is outside (0, exp(x/2)).
pub fn total_vol(x: f64, beta: f64) -> Option<(f64, u32)> {
    if !(beta > 0.0 && beta < (0.5 * x).exp()) {
        return None;
    }
    if x == 0.0 {
        return Some((2.0 * inverse_cdf(0.5 * (1.0 + beta)), 0));
    }
    let target = beta.ln();
    // leading order: -ln b = x^2 / (2 s^2) + s^2 / 8, one root on each side of the inflection point
    let sc = (2.0 * x.abs()).sqrt();
    let root = (4.0 * target * target - x * x).max(0.0).sqrt();
    let mut s = if beta < normalised_price(x, sc) { (-4.0 * target - 2.0 * root).sqrt() } else { (-4.0 * target + 2.0 * root).sqrt() };
    if !(s > 0.0) {
        s = sc;
    }
    let (mut lo, mut hi, mut previous, mut iterations) = (0.0, f64::INFINITY, f64::INFINITY, 0u32);
    while iterations < 60 {
        iterations += 1;
        let b = normalised_price(x, s);
        if !(b > 0.0) {
            lo = s;
            s = if hi.is_finite() { 0.5 * (lo + hi) } else { 2.0 * s };
            continue;
        }
        let f = b.ln() - target;
        if f < 0.0 {
            lo = s;
        } else {
            hi = s;
        }
        let f1 = normalised_vega(x, s) / b;
        let f2 = f1 * (x * x / (s * s * s) - 0.25 * s) - f1 * f1;
        let newton = -f / f1;
        let mut step = newton / (1.0 + 0.5 * newton * f2 / f1).max(0.5);
        let mut next = s + step;
        if !(next >= lo && next <= hi && next > 0.0) {
            next = if hi.is_finite() { 0.5 * (lo + hi) } else { 2.0 * s };
            step = next - s;
        }
        s = next;
        let size = step.abs();
        if size <= 1e-14 * s || (size >= previous && previous <= 1e-9 * s) {
            break;
        }
        previous = size;
    }
    Some((s, iterations))
}

/// Implied volatility from an undiscounted option price on a forward.
pub fn implied_vol(price: f64, forward: f64, strike: f64, t: f64, kind: OptionKind) -> Option<f64> {
    let x = (forward / strike).ln();
    let intrinsic = (kind.sign() * (forward - strike)).max(0.0);
    let beta = (price - intrinsic) / (forward * strike).sqrt();
    total_vol(-x.abs(), beta).map(|(s, _)| s / t.sqrt())
}
