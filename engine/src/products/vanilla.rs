//! European calls and puts: the Black-Scholes formula and its sensitivities.

use super::OptionKind;
use crate::market::Market;
use crate::math::normal::{cdf, pdf};

fn d1_d2(m: &Market, strike: f64, t: f64) -> (f64, f64) {
    let sd = m.vol * t.sqrt();
    let d1 = ((m.spot / strike).ln() + (m.rate - m.dividend + 0.5 * m.vol * m.vol) * t) / sd;
    (d1, d1 - sd)
}

pub fn price(m: &Market, kind: OptionKind, strike: f64, t: f64) -> f64 {
    if t <= 0.0 {
        return kind.payoff(m.spot, strike);
    }
    let (d1, d2) = d1_d2(m, strike, t);
    let w = kind.sign();
    w * (m.spot * (-m.dividend * t).exp() * cdf(w * d1) - strike * m.discount(t) * cdf(w * d2))
}

/// Sensitivities of a European option.
#[derive(Clone, Copy, Debug)]
pub struct Greeks {
    pub delta: f64,
    pub gamma: f64,
    pub vega: f64,
    pub theta: f64,
    pub rho: f64,
}

pub fn greeks(m: &Market, kind: OptionKind, strike: f64, t: f64) -> Greeks {
    let (d1, d2) = d1_d2(m, strike, t);
    let w = kind.sign();
    let (dq, dr) = ((-m.dividend * t).exp(), m.discount(t));
    let delta = w * dq * cdf(w * d1);
    let gamma = dq * pdf(d1) / (m.spot * m.vol * t.sqrt());
    let vega = m.spot * dq * pdf(d1) * t.sqrt();
    let theta = -m.spot * dq * pdf(d1) * m.vol / (2.0 * t.sqrt()) - w * m.rate * strike * dr * cdf(w * d2) + w * m.dividend * m.spot * dq * cdf(w * d1);
    let rho = w * strike * t * dr * cdf(w * d2);
    Greeks { delta, gamma, vega, theta, rho }
}

/// Undiscounted Black price on a forward, per unit of total volatility `s = vol * sqrt(t)`.
pub fn black_forward(forward: f64, strike: f64, s: f64, kind: OptionKind) -> f64 {
    let d1 = (forward / strike).ln() / s + 0.5 * s;
    let w = kind.sign();
    w * (forward * cdf(w * d1) - strike * cdf(w * (d1 - s)))
}
