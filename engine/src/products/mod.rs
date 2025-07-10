//! Payoffs and closed-form prices.

pub mod asian;
pub mod barrier;
pub mod digital;
pub mod lookback;
pub mod vanilla;

/// Call or put.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptionKind {
    Call,
    Put,
}

impl OptionKind {
    /// +1 for a call, -1 for a put.
    pub fn sign(self) -> f64 {
        match self {
            OptionKind::Call => 1.0,
            OptionKind::Put => -1.0,
        }
    }

    pub fn payoff(self, spot: f64, strike: f64) -> f64 {
        (self.sign() * (spot - strike)).max(0.0)
    }

    pub fn parse(text: &str) -> Option<Self> {
        match text.to_ascii_lowercase().as_str() {
            "call" | "c" => Some(OptionKind::Call),
            "put" | "p" => Some(OptionKind::Put),
            _ => None,
        }
    }
}
