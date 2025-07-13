//! Down-and-out and down-and-in calls with a continuously monitored barrier below the strike.

use super::vanilla;
use super::OptionKind;
use crate::market::Market;
use crate::math::normal::cdf;

/// Value of the knocked-in part: a call that only exists if the spot has touched `barrier` (barrier <= strike).
pub fn down_and_in_call(m: &Market, strike: f64, barrier: f64, t: f64) -> f64 {
    assert!(barrier <= strike, "closed form implemented for a barrier at or below the strike");
    if m.spot <= barrier {
        return vanilla::price(m, OptionKind::Call, strike, t);
    }
    let sd = m.vol * t.sqrt();
    let lambda = (m.rate - m.dividend + 0.5 * m.vol * m.vol) / (m.vol * m.vol);
    let y = (barrier * barrier / (m.spot * strike)).ln() / sd + lambda * sd;
    let ratio = barrier / m.spot;
    m.spot * (-m.dividend * t).exp() * ratio.powf(2.0 * lambda) * cdf(y) - strike * m.discount(t) * ratio.powf(2.0 * lambda - 2.0) * cdf(y - sd)
}

/// A call that dies if the spot touches `barrier`: the vanilla call minus the knocked-in call.
pub fn down_and_out_call(m: &Market, strike: f64, barrier: f64, t: f64) -> f64 {
    if m.spot <= barrier {
        return 0.0;
    }
    vanilla::price(m, OptionKind::Call, strike, t) - down_and_in_call(m, strike, barrier, t)
}
