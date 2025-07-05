//! Running mean and variance.

/// Accumulates a sample and reports its mean and the standard error of the mean.
#[derive(Clone, Copy, Debug, Default)]
pub struct Accumulator {
    n: u64,
    sum: f64,
    sum_sq: f64,
}

impl Accumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, x: f64) {
        self.n += 1;
        self.sum += x;
        self.sum_sq += x * x;
    }

    pub fn count(&self) -> u64 {
        self.n
    }

    pub fn mean(&self) -> f64 {
        self.sum / self.n as f64
    }

    pub fn variance(&self) -> f64 {
        let m = self.mean();
        ((self.sum_sq / self.n as f64 - m * m) * self.n as f64 / (self.n as f64 - 1.0)).max(0.0)
    }

    pub fn std_error(&self) -> f64 {
        (self.variance() / self.n as f64).sqrt()
    }
}

/// Sample covariance of two equally long slices.
pub fn covariance(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len() as f64;
    let (mx, my) = (x.iter().sum::<f64>() / n, y.iter().sum::<f64>() / n);
    x.iter().zip(y).map(|(a, b)| (a - mx) * (b - my)).sum::<f64>() / (n - 1.0)
}
