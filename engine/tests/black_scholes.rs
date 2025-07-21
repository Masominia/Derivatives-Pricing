use pricekit::engines::greeks;
use pricekit::products::{digital, vanilla};
use pricekit::{Market, OptionKind};

fn market() -> Market {
    Market::new(100.0, 0.05, 0.02, 0.2)
}

#[test]
fn reference_price() {
    let m = Market::new(100.0, 0.05, 0.0, 0.2);
    assert!((vanilla::price(&m, OptionKind::Call, 100.0, 1.0) - 10.450_583_572_185_565).abs() < 1e-12);
    assert!((vanilla::price(&m, OptionKind::Put, 100.0, 1.0) - 5.573_526_022_256_971).abs() < 1e-12);
}

#[test]
fn put_call_parity() {
    let m = market();
    for &k in &[60.0, 90.0, 100.0, 125.0] {
        for &t in &[0.1, 1.0, 5.0] {
            let lhs = vanilla::price(&m, OptionKind::Call, k, t) - vanilla::price(&m, OptionKind::Put, k, t);
            let rhs = m.spot * (-m.dividend * t).exp() - k * m.discount(t);
            assert!((lhs - rhs).abs() < 1e-12);
        }
    }
}

#[test]
fn analytic_greeks_match_finite_differences() {
    let m = market();
    for kind in [OptionKind::Call, OptionKind::Put] {
        let g = vanilla::greeks(&m, kind, 105.0, 0.75);
        let b = greeks::bump(|mm| vanilla::price(mm, kind, 105.0, 0.75), &m);
        assert!((g.delta - b.delta).abs() < 1e-6 && (g.gamma - b.gamma).abs() < 1e-6);
        assert!((g.vega - b.vega).abs() < 1e-5 && (g.rho - b.rho).abs() < 1e-5);
        let h = 1e-5;
        let theta = (vanilla::price(&m, kind, 105.0, 0.75 - h) - vanilla::price(&m, kind, 105.0, 0.75 + h)) / (2.0 * h);
        assert!((g.theta - theta).abs() < 1e-6);
    }
}

#[test]
fn digitals_decompose_the_vanilla() {
    let m = market();
    let (k, t) = (95.0, 1.3);
    let call = digital::asset_or_nothing(&m, OptionKind::Call, k, t) - k * digital::cash_or_nothing(&m, OptionKind::Call, k, t);
    assert!((call - vanilla::price(&m, OptionKind::Call, k, t)).abs() < 1e-12);
    let both = digital::cash_or_nothing(&m, OptionKind::Call, k, t) + digital::cash_or_nothing(&m, OptionKind::Put, k, t);
    assert!((both - m.discount(t)).abs() < 1e-14);
    let h = 1e-4;
    let slope = -(vanilla::price(&m, OptionKind::Call, k + h, t) - vanilla::price(&m, OptionKind::Call, k - h, t)) / (2.0 * h);
    assert!((slope - digital::cash_or_nothing(&m, OptionKind::Call, k, t)).abs() < 1e-7);
}

#[test]
fn limits() {
    let m = market();
    assert_eq!(vanilla::price(&m, OptionKind::Call, 90.0, 0.0), 10.0);
    let deep = vanilla::price(&m, OptionKind::Call, 1.0, 1.0);
    assert!((deep - (m.spot * (-m.dividend).exp() - m.discount(1.0))).abs() < 1e-10);
    assert!(vanilla::price(&m, OptionKind::Call, 1e4, 1.0) < 1e-20);
}
