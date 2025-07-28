//! Finite differences on the Black-Scholes equation in log-price.
//!
//! Crank-Nicolson in time with four implicit half-steps at the start (Rannacher), central differences in space on
//! a uniform grid centred on the strike. American puts are handled by the Brennan-Schwartz projected solve, which
//! is exact for a single exercise boundary. A knock-out barrier is imposed as a grid boundary.

use super::Exercise;
use crate::market::Market;
use crate::math::interp;
use crate::products::OptionKind;

/// What the grid computes.
#[derive(Clone, Copy, Debug)]
pub struct Spec {
    pub kind: OptionKind,
    pub exercise: Exercise,
    pub strike: f64,
    pub maturity: f64,
    /// Lower knock-out barrier: the option is worthless at and below this spot.
    pub lower_barrier: Option<f64>,
}

#[derive(Clone, Copy, Debug)]
pub struct Solution {
    pub price: f64,
    pub delta: f64,
    pub gamma: f64,
}

pub fn solve(m: &Market, spec: &Spec, space_steps: usize, time_steps: usize) -> Solution {
    assert!(!(spec.exercise == Exercise::American && spec.kind == OptionKind::Call), "American exercise is implemented for puts");
    let half = (6.0 * m.vol * spec.maturity.sqrt()).max(4f64.ln()).max(1.2 * (m.spot / spec.strike).ln().abs());
    let centre = spec.strike.ln();
    let (x_lo, x_hi, n) = match spec.lower_barrier {
        Some(h) => {
            // the barrier is the first node; the spacing is chosen so that the strike is also a node
            let below = centre - h.ln();
            let cells_below = ((space_steps as f64) * below / (below + half)).round().max(1.0);
            let dx = below / cells_below;
            let cells_above = (half / dx).ceil();
            (h.ln(), centre + cells_above * dx, (cells_below + cells_above) as usize + 1)
        }
        None => {
            let cells = 2 * (space_steps / 2);
            (centre - half, centre + half, cells + 1)
        }
    };
    let dx = (x_hi - x_lo) / (n - 1) as f64;
    let s: Vec<f64> = (0..n).map(|i| (x_lo + i as f64 * dx).exp()).collect();
    let payoff: Vec<f64> = s.iter().map(|&v| spec.kind.payoff(v, spec.strike)).collect();
    let a = 0.5 * m.vol * m.vol;
    let b = m.rate - m.dividend - a;
    let (lo, di, up) = (a / (dx * dx) - b / (2.0 * dx), -2.0 * a / (dx * dx) - m.rate, a / (dx * dx) + b / (2.0 * dx));

    let mut v = payoff.clone();
    if spec.lower_barrier.is_some() {
        v[0] = 0.0;
    }
    let mut rhs = vec![0.0; n];
    let mut inv = vec![0.0; n];
    let mut wfac = vec![0.0; n];
    let mut factored = (0.0, -1.0);
    let dt = spec.maturity / time_steps as f64;
    let mut tau = 0.0;
    let american = spec.exercise == Exercise::American;
    let call = spec.kind == OptionKind::Call;
    for step in 0..time_steps {
        let plan: &[(f64, f64)] = if step < 2 { &[(0.5, 1.0), (0.5, 1.0)] } else { &[(1.0, 0.5)] };
        for &(frac, theta) in plan {
            let h = frac * dt;
            tau += h;
            let (el, ed, eu) = ((1.0 - theta) * h * lo, 1.0 + (1.0 - theta) * h * di, (1.0 - theta) * h * up);
            let (il, id, iu) = (-theta * h * lo, 1.0 - theta * h * di, -theta * h * up);
            for i in 1..n - 1 {
                rhs[i] = el * v[i - 1] + ed * v[i] + eu * v[i + 1];
            }
            // boundary values at the new time level
            let fwd_disc = (-m.dividend * tau).exp();
            let left = if spec.lower_barrier.is_some() {
                0.0
            } else if call {
                0.0
            } else if american {
                spec.strike - s[0]
            } else {
                spec.strike * (-m.rate * tau).exp() - s[0] * fwd_disc
            };
            let right = if call { s[n - 1] * fwd_disc - spec.strike * (-m.rate * tau).exp() } else { 0.0 };
            rhs[1] -= il * left;
            rhs[n - 2] -= iu * right;
            if (h, theta) != factored {
                inv[n - 2] = 1.0 / id;
                for i in (1..n - 2).rev() {
                    wfac[i] = iu * inv[i + 1];
                    inv[i] = 1.0 / (id - wfac[i] * il);
                }
                factored = (h, theta);
            }
            for i in (1..n - 2).rev() {
                rhs[i] -= wfac[i] * rhs[i + 1];
            }
            v[0] = left;
            v[n - 1] = right;
            let mut below = 0.0;
            for i in 1..n - 1 {
                let mut val = (rhs[i] - il * below) * inv[i];
                if american && val < payoff[i] {
                    val = payoff[i];
                }
                v[i] = val;
                below = val;
            }
        }
    }
    let (price, vx, vxx) = interp::quadratic(&v, x_lo, dx, m.spot.ln());
    Solution { price, delta: vx / m.spot, gamma: (vxx - vx) / (m.spot * m.spot) }
}

/// Two grids combined by Richardson extrapolation, assuming second-order convergence.
pub fn solve_extrapolated(m: &Market, spec: &Spec, space_steps: usize) -> f64 {
    let coarse = solve(m, spec, space_steps, space_steps / 2).price;
    let fine = solve(m, spec, 2 * space_steps, space_steps).price;
    fine + (fine - coarse) / 3.0
}
