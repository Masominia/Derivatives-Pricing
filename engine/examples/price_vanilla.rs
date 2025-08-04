//! One European call priced four ways.
//!
//!     cargo run --release --example price_vanilla

use pricekit::engines::pde::{solve_extrapolated, Spec};
use pricekit::engines::{binomial, montecarlo, Exercise};
use pricekit::products::vanilla;
use pricekit::{Market, OptionKind};

fn main() {
    let m = Market::new(100.0, 0.05, 0.0, 0.2);
    let (k, t) = (105.0, 1.0);
    let exact = vanilla::price(&m, OptionKind::Call, k, t);
    let spec = Spec { kind: OptionKind::Call, exercise: Exercise::European, strike: k, maturity: t, lower_barrier: None };
    let mc = montecarlo::european(&m, OptionKind::Call, k, t, 500_000, 1);
    println!("closed form        {exact:.6}");
    println!("binomial, 2,000    {:.6}", binomial::price(&m, OptionKind::Call, Exercise::European, k, t, 2000));
    println!("finite differences {:.6}", solve_extrapolated(&m, &spec, 400));
    println!("Monte Carlo        {:.6} +- {:.6}", mc.price, mc.std_error);
}
