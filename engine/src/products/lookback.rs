//! Floating-strike lookback call: pays the final spot minus the lowest spot seen (continuous monitoring).

use crate::market::Market;
use crate::math::normal::cdf;

/// Price at inception, when the running minimum equals the spot (Goldman, Sosin and Gatto). Requires a non-zero
/// carry rate - dividend.
pub fn floating_strike_call(m: &Market, t: f64) -> f64 {
    let b = m.rate - m.dividend;
    assert!(b.abs() > 1e-12, "the closed form needs a non-zero carry");
    let sd = m.vol * t.sqrt();
    let a1 = (b + 0.5 * m.vol * m.vol) * t / sd;
    let a2 = a1 - sd;
    let a3 = (-b + 0.5 * m.vol * m.vol) * t / sd;
    let g = m.vol * m.vol / (2.0 * b);
    let dq = (-m.dividend * t).exp();
    let dr = m.discount(t);
    m.spot * dq * cdf(a1) - m.spot * dq * g * cdf(-a1) - m.spot * dr * (cdf(a2) - g * cdf(-a3))
}
