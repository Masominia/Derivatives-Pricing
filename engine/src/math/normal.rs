//! Error function and normal distribution to double precision, from series that involve no cancellation.

use std::f64::consts::PI;

/// Complementary error function. For |x| < 2.5 a series with positive terms, beyond that a continued fraction.
pub fn erfc(x: f64) -> f64 {
    if x < 0.0 {
        return 2.0 - erfc(-x);
    }
    if x < 2.5 {
        // erf(x) = 2/sqrt(pi) exp(-x^2) sum_n 2^n x^(2n+1) / (1 * 3 * ... * (2n+1))
        let mut term = x;
        let mut sum = x;
        let mut n = 0.0;
        while term > 1e-17 * sum {
            n += 1.0;
            term *= 2.0 * x * x / (2.0 * n + 1.0);
            sum += term;
        }
        1.0 - 2.0 / PI.sqrt() * (-x * x).exp() * sum
    } else {
        // erfc(x) = exp(-x^2) / (x sqrt(pi)) * 1 / (1 + a1 / (1 + a2 / (1 + ...))), a_k = k / (2 x^2)
        let z = 2.0 * x * x;
        let mut frac = 1.0;
        for k in (1..=80).rev() {
            frac = 1.0 + k as f64 / z / frac;
        }
        (-x * x).exp() / (x * PI.sqrt()) / frac
    }
}

/// Standard normal distribution function.
pub fn cdf(x: f64) -> f64 {
    0.5 * erfc(-x / std::f64::consts::SQRT_2)
}

/// Standard normal density.
pub fn pdf(x: f64) -> f64 {
    (-0.5 * x * x).exp() / (2.0 * PI).sqrt()
}

/// Inverse of the standard normal distribution function: Acklam's rational approximation, then one Halley step.
pub fn inverse_cdf(p: f64) -> f64 {
    assert!(p > 0.0 && p < 1.0, "probability must be strictly between 0 and 1");
    const A: [f64; 6] = [-3.969683028665376e+01, 2.209460984245205e+02, -2.759285104469687e+02, 1.383577518672690e+02, -3.066479806614716e+01, 2.506628277459239e+00];
    const B: [f64; 5] = [-5.447609879822406e+01, 1.615858368580409e+02, -1.556989798598866e+02, 6.680131188771972e+01, -1.328068155288572e+01];
    const C: [f64; 6] = [-7.784894002430293e-03, -3.223964580411365e-01, -2.400758277161838e+00, -2.549732539343734e+00, 4.374664141464968e+00, 2.938163982698783e+00];
    const D: [f64; 4] = [7.784695709041462e-03, 3.224671290700398e-01, 2.445134137142996e+00, 3.754408661907416e+00];
    let tail = |q: f64| (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5]) / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0);
    let x = if p < 0.02425 {
        tail((-2.0 * p.ln()).sqrt())
    } else if p > 1.0 - 0.02425 {
        -tail((-2.0 * (1.0 - p).ln()).sqrt())
    } else {
        let q = p - 0.5;
        let r = q * q;
        (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
    };
    let e = cdf(x) - p;
    let u = e * (2.0 * PI).sqrt() * (0.5 * x * x).exp();
    x - u / (1.0 + 0.5 * x * u)
}
