//! SABR smile: Hagan's asymptotic formula for the lognormal implied volatility, and a small calibrator.

/// SABR parameters: level alpha, backbone beta, correlation rho, volatility of volatility nu.
#[derive(Clone, Copy, Debug)]
pub struct Sabr {
    pub alpha: f64,
    pub beta: f64,
    pub rho: f64,
    pub nu: f64,
}

impl Sabr {
    /// Lognormal implied volatility for a forward, strike and expiry.
    pub fn vol(&self, forward: f64, strike: f64, t: f64) -> f64 {
        let Sabr { alpha, beta, rho, nu } = *self;
        let one = 1.0 - beta;
        let fk = (forward * strike).powf(0.5 * one);
        let correction = 1.0 + (one * one / 24.0 * alpha * alpha / (fk * fk) + 0.25 * rho * beta * nu * alpha / fk + (2.0 - 3.0 * rho * rho) / 24.0 * nu * nu) * t;
        let log = (forward / strike).ln();
        if log.abs() < 1e-10 {
            return alpha / forward.powf(one) * correction;
        }
        let denom = fk * (1.0 + one * one / 24.0 * log * log + one.powi(4) / 1920.0 * log.powi(4));
        let z = nu / alpha * fk * log;
        let ratio = if z.abs() < 1e-8 {
            1.0 - 0.5 * rho * z
        } else {
            z / (((1.0 - 2.0 * rho * z + z * z).sqrt() + z - rho) / (1.0 - rho)).ln()
        };
        alpha / denom * ratio * correction
    }

    /// Alpha that reproduces a given at-the-money volatility, by fixed-point iteration on the at-the-money formula.
    pub fn alpha_from_atm(atm_vol: f64, forward: f64, t: f64, beta: f64, rho: f64, nu: f64) -> f64 {
        let mut alpha = atm_vol * forward.powf(1.0 - beta);
        for _ in 0..50 {
            let trial = Sabr { alpha, beta, rho, nu }.vol(forward, forward, t);
            alpha *= atm_vol / trial;
        }
        alpha
    }
}

/// Least-squares fit of (rho, nu) to a smile with beta fixed and alpha tied to the at-the-money volatility.
/// A coarse grid search refined three times; robust and derivative-free.
pub fn calibrate(forward: f64, t: f64, beta: f64, strikes: &[f64], vols: &[f64], atm_vol: f64) -> (Sabr, f64) {
    let error = |rho: f64, nu: f64| -> (Sabr, f64) {
        let model = Sabr { alpha: Sabr::alpha_from_atm(atm_vol, forward, t, beta, rho, nu), beta, rho, nu };
        let sse: f64 = strikes.iter().zip(vols).map(|(&k, &v)| (model.vol(forward, k, t) - v).powi(2)).sum();
        (model, (sse / strikes.len() as f64).sqrt())
    };
    let (mut rho_c, mut nu_c, mut rho_w, mut nu_w) = (0.0, 1.0, 0.95, 0.95);
    let mut best = error(rho_c, nu_c);
    for _ in 0..4 {
        for i in 0..=20 {
            for j in 0..=20 {
                let rho = (rho_c + rho_w * (i as f64 / 10.0 - 1.0)).clamp(-0.99, 0.99);
                let nu = (nu_c + nu_w * (j as f64 / 10.0 - 1.0)).max(1e-3);
                let trial = error(rho, nu);
                if trial.1 < best.1 {
                    best = trial;
                }
            }
        }
        rho_c = best.0.rho;
        nu_c = best.0.nu;
        rho_w *= 0.15;
        nu_w *= 0.15;
    }
    best
}
