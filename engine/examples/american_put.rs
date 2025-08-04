//! American put: price, early-exercise premium and sensitivities from the grid.
//!
//!     cargo run --release --example american_put

use pricekit::engines::pde::{solve, solve_extrapolated, Spec};
use pricekit::engines::Exercise;
use pricekit::products::vanilla;
use pricekit::{Market, OptionKind};

fn main() {
    println!("{:>6} {:>10} {:>10} {:>9} {:>9} {:>9}", "spot", "American", "European", "premium", "delta", "gamma");
    for spot in [80.0, 90.0, 100.0, 110.0, 120.0] {
        let m = Market::new(spot, 0.05, 0.0, 0.2);
        let spec = Spec { kind: OptionKind::Put, exercise: Exercise::American, strike: 100.0, maturity: 1.0, lower_barrier: None };
        let am = solve_extrapolated(&m, &spec, 400);
        let eu = vanilla::price(&m, OptionKind::Put, 100.0, 1.0);
        let sol = solve(&m, &spec, 800, 400);
        println!("{spot:>6.1} {am:>10.4} {eu:>10.4} {:>9.4} {:>9.4} {:>9.4}", am - eu, sol.delta, sol.gamma);
    }
}
