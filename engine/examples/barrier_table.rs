//! Down-and-out call against its barrier: closed form, finite differences, Monte Carlo.
//!
//!     cargo run --release --example barrier_table

use pricekit::engines::montecarlo;
use pricekit::engines::pde::{solve_extrapolated, Spec};
use pricekit::engines::Exercise;
use pricekit::products::barrier;
use pricekit::{Market, OptionKind};

fn main() {
    let m = Market::new(100.0, 0.05, 0.02, 0.25);
    println!("{:>8} {:>12} {:>12} {:>20}", "barrier", "closed form", "grid", "Monte Carlo");
    for h in [70.0, 80.0, 90.0, 95.0, 99.0] {
        let exact = barrier::down_and_out_call(&m, 100.0, h, 1.0);
        let spec = Spec { kind: OptionKind::Call, exercise: Exercise::European, strike: 100.0, maturity: 1.0, lower_barrier: Some(h) };
        let mc = montecarlo::down_and_out_call(&m, 100.0, h, 1.0, 20, 200_000, 1);
        println!("{h:>8.1} {exact:>12.5} {:>12.5} {:>12.5} +- {:.5}", solve_extrapolated(&m, &spec, 400), mc.price, mc.std_error);
    }
}
