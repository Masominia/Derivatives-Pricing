//! Command line front end: `pricekit <command> key=value ...`, output as `key value` lines.
//!
//! Commands: vanilla, digital, barrier, asian, lookback, american, implied, sabr.
//! Common keys: spot strike rate dividend vol maturity kind (call|put).

use std::collections::HashMap;
use std::env;
use std::process;
use std::time::Instant;

use pricekit::engines::{binomial, greeks, montecarlo, pde, Exercise};
use pricekit::products::{asian, barrier, digital, lookback, vanilla};
use pricekit::vol::{implied, sabr::Sabr};
use pricekit::{Market, OptionKind};

struct Args(HashMap<String, String>);

impl Args {
    fn num(&self, key: &str, default: Option<f64>) -> f64 {
        match self.0.get(key) {
            Some(v) => v.parse().unwrap_or_else(|_| fail(&format!("{key} is not a number"))),
            None => default.unwrap_or_else(|| fail(&format!("missing {key}=..."))),
        }
    }
    fn kind(&self) -> OptionKind {
        OptionKind::parse(self.0.get("kind").map(String::as_str).unwrap_or("call")).unwrap_or_else(|| fail("kind must be call or put"))
    }
    fn market(&self) -> Market {
        Market::new(self.num("spot", None), self.num("rate", Some(0.0)), self.num("dividend", Some(0.0)), self.num("vol", None))
    }
}

fn fail(msg: &str) -> ! {
    eprintln!("pricekit: {msg}");
    process::exit(1)
}

fn main() {
    let mut argv = env::args().skip(1);
    let command = argv.next().unwrap_or_else(|| fail("usage: pricekit <command> key=value ..."));
    let a = Args(argv.filter_map(|s| s.split_once('=').map(|(k, v)| (k.to_string(), v.to_string()))).collect());
    let t0 = Instant::now();
    match command.as_str() {
        "vanilla" => {
            let (m, k, t, kind) = (a.market(), a.num("strike", None), a.num("maturity", None), a.kind());
            let g = vanilla::greeks(&m, kind, k, t);
            println!("price {:.15e}", vanilla::price(&m, kind, k, t));
            println!("delta {:.15e}\ngamma {:.15e}\nvega {:.15e}\ntheta {:.15e}\nrho {:.15e}", g.delta, g.gamma, g.vega, g.theta, g.rho);
            let b = greeks::bump(|mm| vanilla::price(mm, kind, k, t), &m);
            println!("bump_delta {:.15e}\nbump_gamma {:.15e}\nbump_vega {:.15e}\nbump_rho {:.15e}", b.delta, b.gamma, b.vega, b.rho);
            let steps = a.num("steps", Some(500.0)) as usize;
            println!("binomial {:.15e}", binomial::price(&m, kind, Exercise::European, k, t, steps));
            let spec = pde::Spec { kind, exercise: Exercise::European, strike: k, maturity: t, lower_barrier: None };
            let grid = a.num("grid", Some(400.0)) as usize;
            let sol = pde::solve(&m, &spec, grid, grid / 2);
            println!("pde {:.15e}\npde_delta {:.15e}\npde_gamma {:.15e}\npde_extrapolated {:.15e}", sol.price, sol.delta, sol.gamma, pde::solve_extrapolated(&m, &spec, grid));
            let mc = montecarlo::european(&m, kind, k, t, a.num("paths", Some(200_000.0)) as u64, a.num("seed", Some(1.0)) as u64);
            println!("mc {:.15e}\nmc_std_error {:.15e}", mc.price, mc.std_error);
        }
        "digital" => {
            let (m, k, t, kind) = (a.market(), a.num("strike", None), a.num("maturity", None), a.kind());
            println!("cash_or_nothing {:.15e}\nasset_or_nothing {:.15e}", digital::cash_or_nothing(&m, kind, k, t), digital::asset_or_nothing(&m, kind, k, t));
        }
        "barrier" => {
            let (m, k, t, h) = (a.market(), a.num("strike", None), a.num("maturity", None), a.num("barrier", None));
            println!("down_and_out {:.15e}\ndown_and_in {:.15e}", barrier::down_and_out_call(&m, k, h, t), barrier::down_and_in_call(&m, k, h, t));
            println!("vanilla {:.15e}", vanilla::price(&m, OptionKind::Call, k, t));
            let spec = pde::Spec { kind: OptionKind::Call, exercise: Exercise::European, strike: k, maturity: t, lower_barrier: Some(h) };
            let grid = a.num("grid", Some(400.0)) as usize;
            println!("pde {:.15e}\npde_extrapolated {:.15e}", pde::solve(&m, &spec, grid, grid / 2).price, pde::solve_extrapolated(&m, &spec, grid));
            let mc = montecarlo::down_and_out_call(&m, k, h, t, a.num("steps", Some(50.0)) as usize, a.num("paths", Some(100_000.0)) as u64, a.num("seed", Some(1.0)) as u64);
            println!("mc {:.15e}\nmc_std_error {:.15e}", mc.price, mc.std_error);
        }
        "asian" => {
            let (m, k, t) = (a.market(), a.num("strike", None), a.num("maturity", None));
            let (dates, paths, seed) = (a.num("dates", Some(50.0)) as usize, a.num("paths", Some(100_000.0)) as u64, a.num("seed", Some(1.0)) as u64);
            println!("geometric_continuous {:.15e}\ngeometric_discrete {:.15e}", asian::geometric_call(&m, k, t), asian::geometric_call_discrete(&m, k, t, dates));
            let plain = montecarlo::asian_call(&m, k, t, dates, paths, seed, false);
            let ctrl = montecarlo::asian_call(&m, k, t, dates, paths, seed, true);
            println!("arithmetic_plain {:.15e}\nplain_std_error {:.15e}", plain.price, plain.std_error);
            println!("arithmetic_control {:.15e}\ncontrol_std_error {:.15e}", ctrl.price, ctrl.std_error);
        }
        "lookback" => {
            let (m, t) = (a.market(), a.num("maturity", None));
            println!("closed_form {:.15e}", lookback::floating_strike_call(&m, t));
            let mc = montecarlo::lookback_call(&m, t, a.num("steps", Some(20.0)) as usize, a.num("paths", Some(200_000.0)) as u64, a.num("seed", Some(1.0)) as u64);
            println!("mc {:.15e}\nmc_std_error {:.15e}", mc.price, mc.std_error);
        }
        "american" => {
            let (m, k, t) = (a.market(), a.num("strike", None), a.num("maturity", None));
            let spec = pde::Spec { kind: OptionKind::Put, exercise: Exercise::American, strike: k, maturity: t, lower_barrier: None };
            let grid = a.num("grid", Some(400.0)) as usize;
            let sol = pde::solve(&m, &spec, grid, grid / 2);
            println!("pde {:.15e}\npde_delta {:.15e}\npde_gamma {:.15e}\npde_extrapolated {:.15e}", sol.price, sol.delta, sol.gamma, pde::solve_extrapolated(&m, &spec, grid));
            let steps = a.num("steps", Some(1000.0)) as usize;
            println!("binomial {:.15e}", binomial::price(&m, OptionKind::Put, Exercise::American, k, t, steps));
            println!("binomial_extrapolated {:.15e}", binomial::price_extrapolated(&m, OptionKind::Put, Exercise::American, k, t, steps));
            println!("european {:.15e}", vanilla::price(&m, OptionKind::Put, k, t));
        }
        "implied" => {
            let (f, k, t) = (a.num("forward", None), a.num("strike", None), a.num("maturity", None));
            match implied::implied_vol(a.num("price", None), f, k, t, a.kind()) {
                Some(v) => println!("vol {v:.15e}"),
                None => println!("vol nan"),
            }
        }
        "sabr" => {
            let model = Sabr { alpha: a.num("alpha", None), beta: a.num("beta", Some(1.0)), rho: a.num("rho", Some(0.0)), nu: a.num("nu", Some(0.0)) };
            println!("vol {:.15e}", model.vol(a.num("forward", None), a.num("strike", None), a.num("maturity", None)));
        }
        _ => fail("unknown command"),
    }
    println!("seconds {:.6}", t0.elapsed().as_secs_f64());
}
