//! Asian options on the continuous average.

use crate::market::Market;
use crate::math::normal::cdf;

/// Call on the continuous geometric average exp((1/T) int ln S dt): the average is lognormal, so the price is a
/// Black-Scholes formula with volatility vol / sqrt(3) and an adjusted carry (Kemna and Vorst).
pub fn geometric_call(m: &Market, strike: f64, t: f64) -> f64 {
    let sg = m.vol / 3f64.sqrt();
    let b = 0.5 * (m.rate - m.dividend - 0.5 * m.vol * m.vol + sg * sg);
    let d1 = ((m.spot / strike).ln() + (b + 0.5 * sg * sg) * t) / (sg * t.sqrt());
    m.spot * ((b - m.rate) * t).exp() * cdf(d1) - strike * m.discount(t) * cdf(d1 - sg * t.sqrt())
}

/// Call on the geometric average over `n` equally spaced dates t/n, 2t/n, ..., t (exact for that monitoring).
pub fn geometric_call_discrete(m: &Market, strike: f64, t: f64, n: usize) -> f64 {
    let nf = n as f64;
    let mean_time = t * (nf + 1.0) / (2.0 * nf);
    let var = m.vol * m.vol * t * (nf + 1.0) * (2.0 * nf + 1.0) / (6.0 * nf * nf);
    let mean = m.spot.ln() + m.log_drift() * mean_time;
    let sd = var.sqrt();
    let d2 = (mean - strike.ln()) / sd;
    m.discount(t) * ((mean + 0.5 * var).exp() * cdf(d2 + sd) - strike * cdf(d2))
}
