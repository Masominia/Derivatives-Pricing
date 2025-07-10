//! Digital options, which pay a fixed cash amount or the asset itself when they finish in the money.

use super::OptionKind;
use crate::market::Market;
use crate::math::normal::cdf;

fn d2(m: &Market, strike: f64, t: f64) -> f64 {
    ((m.spot / strike).ln() + m.log_drift() * t) / (m.vol * t.sqrt())
}

/// Pays one unit of cash if the option finishes in the money.
pub fn cash_or_nothing(m: &Market, kind: OptionKind, strike: f64, t: f64) -> f64 {
    m.discount(t) * cdf(kind.sign() * d2(m, strike, t))
}

/// Pays one unit of the asset if the option finishes in the money.
pub fn asset_or_nothing(m: &Market, kind: OptionKind, strike: f64, t: f64) -> f64 {
    let d1 = d2(m, strike, t) + m.vol * t.sqrt();
    m.spot * (-m.dividend * t).exp() * cdf(kind.sign() * d1)
}
