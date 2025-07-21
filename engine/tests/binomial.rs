use pricekit::engines::{binomial, Exercise};
use pricekit::products::vanilla;
use pricekit::{Market, OptionKind};

#[test]
fn european_converges_to_the_closed_form() {
    let m = Market::new(100.0, 0.05, 0.01, 0.25);
    let exact = vanilla::price(&m, OptionKind::Put, 110.0, 1.0);
    let e1 = (binomial::price(&m, OptionKind::Put, Exercise::European, 110.0, 1.0, 200) - exact).abs();
    let e2 = (binomial::price(&m, OptionKind::Put, Exercise::European, 110.0, 1.0, 3200) - exact).abs();
    assert!(e1 < 0.03 && e2 < 0.003);
    let extra = binomial::price_extrapolated(&m, OptionKind::Put, Exercise::European, 110.0, 1.0, 400);
    assert!((extra - exact).abs() < 2e-3);
}

#[test]
fn american_put_is_worth_more_and_call_without_dividend_is_not() {
    let m = Market::new(100.0, 0.05, 0.0, 0.2);
    let am = binomial::price(&m, OptionKind::Put, Exercise::American, 100.0, 1.0, 1000);
    let eu = binomial::price(&m, OptionKind::Put, Exercise::European, 100.0, 1.0, 1000);
    assert!(am > eu + 0.4 && (am - 6.0904).abs() < 5e-3);
    let am_call = binomial::price(&m, OptionKind::Call, Exercise::American, 100.0, 1.0, 500);
    let eu_call = binomial::price(&m, OptionKind::Call, Exercise::European, 100.0, 1.0, 500);
    assert!((am_call - eu_call).abs() < 1e-10);
}
