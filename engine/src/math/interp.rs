//! Interpolation on a uniform grid.

/// Value, first and second derivative at `x` of the parabola through the three grid nodes nearest to it.
/// `x0` is the first node, `dx` the spacing.
pub fn quadratic(values: &[f64], x0: f64, dx: f64, x: f64) -> (f64, f64, f64) {
    let n = values.len();
    assert!(n >= 3, "need at least three nodes");
    let j = (((x - x0) / dx).round() as isize).clamp(1, n as isize - 2) as usize;
    let (fm, f0, fp) = (values[j - 1], values[j], values[j + 1]);
    let u = x - (x0 + j as f64 * dx);
    let d1 = (fp - fm) / (2.0 * dx);
    let d2 = (fp - 2.0 * f0 + fm) / (dx * dx);
    (f0 + u * d1 + 0.5 * u * u * d2, d1 + u * d2, d2)
}

/// Piecewise-linear interpolation on increasing abscissas, flat outside.
pub fn linear(xs: &[f64], ys: &[f64], x: f64) -> f64 {
    if x <= xs[0] {
        return ys[0];
    }
    let last = xs.len() - 1;
    if x >= xs[last] {
        return ys[last];
    }
    let i = xs.partition_point(|&v| v <= x);
    let w = (x - xs[i - 1]) / (xs[i] - xs[i - 1]);
    ys[i - 1] * (1.0 - w) + ys[i] * w
}
