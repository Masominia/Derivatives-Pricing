//! Numerical pricing engines.

pub mod binomial;
pub mod greeks;
pub mod montecarlo;
pub mod pde;

/// European exercise at expiry only, or American exercise at any time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exercise {
    European,
    American,
}
