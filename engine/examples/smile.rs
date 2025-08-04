//! A SABR smile, its calibration back from the quotes, and the implied volatility of the resulting prices.
//!
//!     cargo run --release --example smile

use pricekit::products::vanilla::black_forward;
use pricekit::vol::implied::implied_vol;
use pricekit::vol::sabr::{calibrate, Sabr};
use pricekit::OptionKind;

fn main() {
    let (f, t) = (100.0, 1.0);
    let truth = Sabr { alpha: Sabr::alpha_from_atm(0.2, f, t, 0.5, -0.35, 0.7), beta: 0.5, rho: -0.35, nu: 0.7 };
    let strikes: Vec<f64> = (0..9).map(|i| 70.0 + 7.5 * i as f64).collect();
    let vols: Vec<f64> = strikes.iter().map(|&k| truth.vol(f, k, t)).collect();
    let (fit, rmse) = calibrate(f, t, 0.5, &strikes, &vols, 0.2);
    println!("fitted rho {:.4}, nu {:.4}, error {rmse:.2e}", fit.rho, fit.nu);
    for (&k, &v) in strikes.iter().zip(&vols) {
        let kind = if k < f { OptionKind::Put } else { OptionKind::Call };
        let price = black_forward(f, k, v * t.sqrt(), kind);
        let back = implied_vol(price, f, k, t, kind).unwrap();
        println!("strike {k:>6.1}   vol {:.4}%   price {price:>8.4}   inverted {:.4}%", 100.0 * v, 100.0 * back);
    }
}
