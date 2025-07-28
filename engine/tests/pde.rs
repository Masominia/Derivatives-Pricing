use pricekit::engines::pde::{solve, solve_extrapolated, Spec};
use pricekit::engines::{binomial, Exercise};
use pricekit::products::{barrier, vanilla};
use pricekit::{Market, OptionKind};

fn spec(kind: OptionKind, exercise: Exercise, lower_barrier: Option<f64>) -> Spec {
    Spec { kind, exercise, strike: 100.0, maturity: 1.0, lower_barrier }
}

#[test]
fn european_second_order() {
    let m = Market::new(100.0, 0.05, 0.0, 0.2);
    for kind in [OptionKind::Call, OptionKind::Put] {
        let exact = vanilla::price(&m, kind, 100.0, 1.0);
        let s = spec(kind, Exercise::European, None);
        let e: Vec<f64> = [100usize, 200, 400].iter().map(|&n| (solve(&m, &s, n, n / 2).price - exact).abs()).collect();
        assert!(e[0] / e[1] > 3.5 && e[0] / e[1] < 4.5 && e[1] / e[2] > 3.5 && e[1] / e[2] < 4.5, "{e:?}");
        assert!((solve_extrapolated(&m, &s, 200) - exact).abs() < 1e-5);
    }
}

#[test]
fn greeks_from_the_grid() {
    let m = Market::new(95.0, 0.03, 0.01, 0.3);
    let s = spec(OptionKind::Call, Exercise::European, None);
    let sol = solve(&m, &s, 800, 400);
    let g = vanilla::greeks(&m, OptionKind::Call, 100.0, 1.0);
    assert!((sol.delta - g.delta).abs() < 1e-4 && (sol.gamma - g.gamma).abs() < 2e-4);
}

#[test]
fn american_put_matches_the_tree() {
    let m = Market::new(100.0, 0.05, 0.0, 0.2);
    let pde = solve_extrapolated(&m, &spec(OptionKind::Put, Exercise::American, None), 400);
    let tree = binomial::price_extrapolated(&m, OptionKind::Put, Exercise::American, 100.0, 1.0, 2000);
    assert!((pde - tree).abs() < 5e-4, "{pde} vs {tree}");
    assert!(pde > vanilla::price(&m, OptionKind::Put, 100.0, 1.0));
}

#[test]
fn barrier_matches_the_closed_form() {
    let m = Market::new(100.0, 0.05, 0.02, 0.25);
    for &h in &[80.0, 90.0, 97.0] {
        let exact = barrier::down_and_out_call(&m, 100.0, h, 1.0);
        let got = solve_extrapolated(&m, &spec(OptionKind::Call, Exercise::European, Some(h)), 400);
        assert!((got - exact).abs() < 2e-3, "barrier {h}: {got} vs {exact}");
    }
}
