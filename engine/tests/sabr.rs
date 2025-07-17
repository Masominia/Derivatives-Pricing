use pricekit::vol::sabr::{calibrate, Sabr};

#[test]
fn lognormal_limit_is_flat() {
    let m = Sabr { alpha: 0.25, beta: 1.0, rho: 0.0, nu: 1e-9 };
    for &k in &[60.0, 100.0, 160.0] {
        assert!((m.vol(100.0, k, 1.0) - 0.25).abs() < 1e-8);
    }
}

#[test]
fn continuous_at_the_money_and_skewed_by_correlation() {
    let m = Sabr { alpha: 0.2, beta: 0.7, rho: -0.4, nu: 0.6 };
    let atm = m.vol(100.0, 100.0, 1.0);
    assert!((m.vol(100.0, 100.0 + 1e-6, 1.0) - atm).abs() < 1e-7);
    assert!(m.vol(100.0, 80.0, 1.0) > atm && atm > m.vol(100.0, 110.0, 1.0));
    let mirrored = Sabr { rho: 0.4, ..m };
    assert!(mirrored.vol(100.0, 125.0, 1.0) > m.vol(100.0, 125.0, 1.0));
}

#[test]
fn alpha_solves_the_at_the_money_volatility() {
    let alpha = Sabr::alpha_from_atm(0.22, 100.0, 2.0, 0.5, -0.3, 0.8);
    let m = Sabr { alpha, beta: 0.5, rho: -0.3, nu: 0.8 };
    assert!((m.vol(100.0, 100.0, 2.0) - 0.22).abs() < 1e-12);
}

#[test]
fn calibration_recovers_the_parameters() {
    let truth = Sabr { alpha: Sabr::alpha_from_atm(0.2, 100.0, 1.0, 0.5, -0.35, 0.7), beta: 0.5, rho: -0.35, nu: 0.7 };
    let strikes: Vec<f64> = (0..11).map(|i| 70.0 + 6.0 * i as f64).collect();
    let vols: Vec<f64> = strikes.iter().map(|&k| truth.vol(100.0, k, 1.0)).collect();
    let (fit, rmse) = calibrate(100.0, 1.0, 0.5, &strikes, &vols, 0.2);
    assert!(rmse < 1e-5 && (fit.rho - truth.rho).abs() < 5e-3 && (fit.nu - truth.nu).abs() < 5e-3, "{fit:?} {rmse}");
}
