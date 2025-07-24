//! Monte Carlo under Black-Scholes with exact lognormal steps, antithetic paths and control variates.

use crate::market::Market;
use crate::math::rng::Rng;
use crate::math::stats::{covariance, Accumulator};
use crate::products::{asian, vanilla, OptionKind};

/// A Monte Carlo estimate with its standard error.
#[derive(Clone, Copy, Debug)]
pub struct Estimate {
    pub price: f64,
    pub std_error: f64,
    pub paths: u64,
}

/// European option from the terminal distribution; each draw is used with both signs.
pub fn european(m: &Market, kind: OptionKind, strike: f64, t: f64, pairs: u64, seed: u64) -> Estimate {
    let mut rng = Rng::new(seed);
    let (drift, sd, disc) = (m.log_drift() * t, m.vol * t.sqrt(), m.discount(t));
    let mut acc = Accumulator::new();
    for _ in 0..pairs {
        let z = rng.normal();
        let a = kind.payoff(m.spot * (drift + sd * z).exp(), strike);
        let b = kind.payoff(m.spot * (drift - sd * z).exp(), strike);
        acc.add(disc * 0.5 * (a + b));
    }
    Estimate { price: acc.mean(), std_error: acc.std_error(), paths: 2 * pairs }
}

/// Arithmetic-average Asian call over `dates` equally spaced dates. With `control` the geometric-average call,
/// whose price is known exactly, is used as a control variate.
pub fn asian_call(m: &Market, strike: f64, t: f64, dates: usize, paths: u64, seed: u64, control: bool) -> Estimate {
    let mut rng = Rng::new(seed);
    let dt = t / dates as f64;
    let (drift, sd, disc) = (m.log_drift() * dt, m.vol * dt.sqrt(), m.discount(t));
    let mut arith = Vec::with_capacity(paths as usize);
    let mut geom = Vec::with_capacity(paths as usize);
    for _ in 0..paths {
        let (mut x, mut sum, mut log_sum) = (m.spot.ln(), 0.0, 0.0);
        for _ in 0..dates {
            x += drift + sd * rng.normal();
            sum += x.exp();
            log_sum += x;
        }
        arith.push(disc * (sum / dates as f64 - strike).max(0.0));
        geom.push(disc * ((log_sum / dates as f64).exp() - strike).max(0.0));
    }
    let mut acc = Accumulator::new();
    if control {
        let exact = asian::geometric_call_discrete(m, strike, t, dates);
        let beta = covariance(&arith, &geom) / covariance(&geom, &geom);
        for (a, g) in arith.iter().zip(&geom) {
            acc.add(a - beta * (g - exact));
        }
    } else {
        for a in &arith {
            acc.add(*a);
        }
    }
    Estimate { price: acc.mean(), std_error: acc.std_error(), paths }
}

/// Floating-strike lookback call. The minimum between two dates is drawn from its exact distribution given the
/// end points (Brownian bridge), so the estimate has no monitoring bias.
pub fn lookback_call(m: &Market, t: f64, steps: usize, paths: u64, seed: u64) -> Estimate {
    let mut rng = Rng::new(seed);
    let dt = t / steps as f64;
    let (drift, var, disc) = (m.log_drift() * dt, m.vol * m.vol * dt, m.discount(t));
    let sd = var.sqrt();
    let mut acc = Accumulator::new();
    for _ in 0..paths {
        let x0 = m.spot.ln();
        let (mut x, mut low) = (x0, x0);
        for _ in 0..steps {
            let next = x + drift + sd * rng.normal();
            let d = next - x;
            let bridge_min = 0.5 * (x + next - (d * d - 2.0 * var * rng.uniform().ln()).sqrt());
            low = low.min(bridge_min);
            x = next;
        }
        acc.add(disc * (x.exp() - low.exp()));
    }
    Estimate { price: acc.mean(), std_error: acc.std_error(), paths }
}

/// Down-and-out call with the barrier checked continuously: a path survives a step with the exact bridge
/// probability, so no barrier shift is needed.
pub fn down_and_out_call(m: &Market, strike: f64, barrier: f64, t: f64, steps: usize, paths: u64, seed: u64) -> Estimate {
    let mut rng = Rng::new(seed);
    let dt = t / steps as f64;
    let (drift, var, disc) = (m.log_drift() * dt, m.vol * m.vol * dt, m.discount(t));
    let (sd, hb) = (var.sqrt(), barrier.ln());
    let mut acc = Accumulator::new();
    for _ in 0..paths {
        let mut x = m.spot.ln();
        let mut alive = 1.0; // probability that the path has not touched the barrier
        for _ in 0..steps {
            let next = x + drift + sd * rng.normal();
            if next <= hb {
                alive = 0.0;
                break;
            }
            alive *= 1.0 - (-2.0 * (x - hb) * (next - hb) / var).exp();
            x = next;
        }
        acc.add(disc * alive * (x.exp() - strike).max(0.0));
    }
    Estimate { price: acc.mean(), std_error: acc.std_error(), paths }
}

/// The closed-form European price, for callers that want the control value.
pub fn european_reference(m: &Market, kind: OptionKind, strike: f64, t: f64) -> f64 {
    vanilla::price(m, kind, strike, t)
}
