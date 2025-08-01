//! Pricing of equity derivatives under the Black-Scholes model, by four independent families of methods.
//!
//! - `math`: special functions, root finding, linear algebra and random numbers, with no external crate.
//! - `market`: the model parameters shared by every pricer.
//! - `products`: payoffs and their closed-form prices where one exists.
//! - `engines`: binomial trees, finite differences and Monte Carlo, usable for any product.
//! - `vol`: implied volatility and the SABR smile.
//!
//! Every numerical engine is tested against a closed form, and engines are tested against each other where no
//! closed form exists.

pub mod engines;
pub mod market;
pub mod math;
pub mod products;
pub mod vol;

pub use market::Market;
pub use products::OptionKind;
