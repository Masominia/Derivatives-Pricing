//! Sensitivities of any pricing function by central finite differences.

use crate::market::Market;

#[derive(Clone, Copy, Debug)]
pub struct Bumped {
    pub delta: f64,
    pub gamma: f64,
    pub vega: f64,
    pub rho: f64,
}

/// Delta and gamma from a relative spot bump, vega and rho from absolute bumps.
pub fn bump<F: Fn(&Market) -> f64>(pricer: F, m: &Market) -> Bumped {
    let hs = 1e-3 * m.spot;
    let (up, mid, dn) = (pricer(&m.with_spot(m.spot + hs)), pricer(m), pricer(&m.with_spot(m.spot - hs)));
    let hv = 1e-4;
    let vega = (pricer(&m.with_vol(m.vol + hv)) - pricer(&m.with_vol(m.vol - hv))) / (2.0 * hv);
    let hr = 1e-5;
    let rho = (pricer(&m.with_rate(m.rate + hr)) - pricer(&m.with_rate(m.rate - hr))) / (2.0 * hr);
    Bumped { delta: (up - dn) / (2.0 * hs), gamma: (up - 2.0 * mid + dn) / (hs * hs), vega, rho }
}
