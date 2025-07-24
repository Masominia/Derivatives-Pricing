use pricekit::engines::montecarlo;
use pricekit::products::{asian, barrier, lookback, vanilla};
use pricekit::{Market, OptionKind};

fn market() -> Market {
    Market::new(100.0, 0.05, 0.0, 0.2)
}

#[test]
fn european_within_error_bars() {
    let m = market();
    for kind in [OptionKind::Call, OptionKind::Put] {
        let est = montecarlo::european(&m, kind, 105.0, 1.0, 200_000, 3);
        assert!((est.price - vanilla::price(&m, kind, 105.0, 1.0)).abs() < 4.0 * est.std_error);
        assert!(est.std_error < 0.03 && est.paths == 400_000);
    }
}

#[test]
fn geometric_control_variate_cuts_the_error() {
    let m = market();
    let plain = montecarlo::asian_call(&m, 100.0, 1.0, 50, 50_000, 5, false);
    let ctrl = montecarlo::asian_call(&m, 100.0, 1.0, 50, 50_000, 5, true);
    assert!((plain.price - ctrl.price).abs() < 4.0 * plain.std_error);
    assert!(ctrl.std_error < plain.std_error / 20.0, "{} vs {}", ctrl.std_error, plain.std_error);
    // the arithmetic average is above the geometric one
    assert!(ctrl.price > asian::geometric_call_discrete(&m, 100.0, 1.0, 50));
}

#[test]
fn discrete_geometric_tends_to_continuous() {
    let m = market();
    let cont = asian::geometric_call(&m, 100.0, 1.0);
    let d1 = (asian::geometric_call_discrete(&m, 100.0, 1.0, 100) - cont).abs();
    let d2 = (asian::geometric_call_discrete(&m, 100.0, 1.0, 10_000) - cont).abs();
    assert!(d2 < d1 / 50.0 && d2 < 1e-3);
}

#[test]
fn lookback_bridge_sampling_is_unbiased() {
    let m = market();
    let exact = lookback::floating_strike_call(&m, 1.0);
    assert!((exact - 17.2168).abs() < 1e-3);
    for &steps in &[4usize, 32] {
        let est = montecarlo::lookback_call(&m, 1.0, steps, 200_000, 9);
        assert!((est.price - exact).abs() < 4.0 * est.std_error, "steps {steps}: {} vs {exact}", est.price);
    }
}

#[test]
fn barrier_bridge_correction_is_unbiased() {
    let m = Market::new(100.0, 0.05, 0.02, 0.25);
    let exact = barrier::down_and_out_call(&m, 100.0, 90.0, 1.0);
    let est = montecarlo::down_and_out_call(&m, 100.0, 90.0, 1.0, 10, 400_000, 11);
    assert!((est.price - exact).abs() < 4.0 * est.std_error, "{} vs {exact} +- {}", est.price, est.std_error);
}
