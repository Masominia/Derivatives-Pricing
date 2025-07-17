use pricekit::products::vanilla::black_forward;
use pricekit::vol::implied::{implied_vol, normalised_price, total_vol};
use pricekit::OptionKind;

#[test]
fn round_trip_over_a_wide_range() {
    let mut worst: f64 = 0.0;
    let mut most = 0;
    for i in 0..40 {
        let s = 0.01 * 400f64.powf(i as f64 / 39.0);
        for j in 0..=24 {
            let x = -(j as f64) * 0.25 * s;
            let (got, it) = total_vol(x, normalised_price(x, s)).unwrap();
            worst = worst.max((got / s - 1.0).abs());
            most = most.max(it);
        }
    }
    assert!(worst < 1e-10 && most <= 25, "worst {worst}, iterations {most}");
}

#[test]
fn calls_puts_and_moneyness() {
    for kind in [OptionKind::Call, OptionKind::Put] {
        for &k in &[70.0, 100.0, 140.0] {
            let price = black_forward(100.0, k, 0.3 * 2f64.sqrt(), kind);
            let got = implied_vol(price, 100.0, k, 2.0, kind).unwrap();
            assert!((got - 0.3).abs() < 1e-9, "{kind:?} {k}: {got}");
        }
    }
}

#[test]
fn arbitrage_violations_are_rejected() {
    assert!(implied_vol(0.0, 100.0, 100.0, 1.0, OptionKind::Call).is_none());
    assert!(implied_vol(100.0, 100.0, 100.0, 1.0, OptionKind::Call).is_none());
    assert!(implied_vol(4.0, 100.0, 95.0, 1.0, OptionKind::Call).is_none());
}
