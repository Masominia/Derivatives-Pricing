//! Model parameters.

/// Black-Scholes market: spot, continuously compounded rate and dividend yield, volatility.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Market {
    pub spot: f64,
    pub rate: f64,
    pub dividend: f64,
    pub vol: f64,
}

impl Market {
    pub fn new(spot: f64, rate: f64, dividend: f64, vol: f64) -> Self {
        assert!(spot > 0.0 && vol > 0.0, "spot and volatility must be positive");
        Market { spot, rate, dividend, vol }
    }

    /// Forward price for delivery at `t`.
    pub fn forward(&self, t: f64) -> f64 {
        self.spot * ((self.rate - self.dividend) * t).exp()
    }

    /// Discount factor to `t`.
    pub fn discount(&self, t: f64) -> f64 {
        (-self.rate * t).exp()
    }

    /// Risk-neutral drift of the logarithm of the spot.
    pub fn log_drift(&self) -> f64 {
        self.rate - self.dividend - 0.5 * self.vol * self.vol
    }

    pub fn with_spot(&self, spot: f64) -> Self {
        Market { spot, ..*self }
    }

    pub fn with_vol(&self, vol: f64) -> Self {
        Market { vol, ..*self }
    }

    pub fn with_rate(&self, rate: f64) -> Self {
        Market { rate, ..*self }
    }
}
