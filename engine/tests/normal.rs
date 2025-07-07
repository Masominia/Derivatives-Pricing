use pricekit::math::normal::{cdf, erfc, inverse_cdf, pdf};

#[test]
fn reference_values() {
    assert!((cdf(0.0) - 0.5).abs() < 1e-16);
    assert!((cdf(1.0) - 0.841_344_746_068_542_9).abs() < 1e-15);
    assert!((cdf(-1.96) - 0.024_997_895_148_220_43).abs() < 1e-15);
    assert!((erfc(0.5) - 0.479_500_122_186_953_5).abs() < 1e-15);
    assert!((erfc(3.0) / 2.209_049_699_858_544e-5 - 1.0).abs() < 1e-13);
    assert!((erfc(6.0) / 2.151_973_671_249_891e-17 - 1.0).abs() < 1e-12);
}

#[test]
fn symmetry_and_continuity_at_the_switch() {
    for &x in &[0.1, 0.7, 1.3, 2.2, 3.5, 5.0] {
        assert!((cdf(x) + cdf(-x) - 1.0).abs() < 1e-15);
    }
    let (below, above) = (erfc(2.5 - 1e-9), erfc(2.5 + 1e-9));
    assert!((below / above - 1.0).abs() < 1e-7);
}

#[test]
fn inverse_round_trip() {
    for &p in &[1e-12, 1e-6, 0.01, 0.3, 0.5, 0.9, 0.999_999, 1.0 - 1e-12] {
        assert!((cdf(inverse_cdf(p)) / p - 1.0).abs() < 1e-11, "p = {p}");
    }
    assert!(inverse_cdf(0.5).abs() < 1e-15);
    assert!((inverse_cdf(0.975) - 1.959_963_984_540_054).abs() < 1e-12);
}

#[test]
fn density_is_the_derivative() {
    for &x in &[-2.0, -0.3, 0.0, 1.1, 2.7] {
        let h = 1e-5;
        assert!(((cdf(x + h) - cdf(x - h)) / (2.0 * h) - pdf(x)).abs() < 1e-9);
    }
}
