use pricekit::products::{barrier, vanilla};
use pricekit::{Market, OptionKind};

#[test]
fn in_plus_out_is_vanilla() {
    let m = Market::new(100.0, 0.04, 0.01, 0.3);
    for &h in &[70.0, 85.0, 99.0] {
        let sum = barrier::down_and_in_call(&m, 105.0, h, 0.8) + barrier::down_and_out_call(&m, 105.0, h, 0.8);
        assert!((sum - vanilla::price(&m, OptionKind::Call, 105.0, 0.8)).abs() < 1e-12);
    }
}

#[test]
fn limits_in_the_barrier() {
    let m = Market::new(100.0, 0.04, 0.01, 0.3);
    let vanilla_call = vanilla::price(&m, OptionKind::Call, 105.0, 0.8);
    assert!((barrier::down_and_out_call(&m, 105.0, 1.0, 0.8) - vanilla_call).abs() < 1e-9);
    assert!(barrier::down_and_out_call(&m, 105.0, 99.999, 0.8) < 0.01);
    assert_eq!(barrier::down_and_out_call(&m.with_spot(80.0), 105.0, 85.0, 0.8), 0.0);
    let (lo, hi) = (barrier::down_and_out_call(&m, 105.0, 80.0, 0.8), barrier::down_and_out_call(&m, 105.0, 95.0, 0.8));
    assert!(lo > hi && hi > 0.0);
}
