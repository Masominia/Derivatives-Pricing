//! Tridiagonal systems.

/// Solve a tridiagonal system with constant coefficients (lower, diagonal, upper) in place of `rhs`
/// (Thomas algorithm). `scratch` must have the same length as `rhs`.
pub fn solve_constant(lower: f64, diag: f64, upper: f64, rhs: &mut [f64], scratch: &mut [f64]) {
    let n = rhs.len();
    if n == 0 {
        return;
    }
    scratch[0] = upper / diag;
    rhs[0] /= diag;
    for i in 1..n {
        let m = diag - lower * scratch[i - 1];
        scratch[i] = upper / m;
        rhs[i] = (rhs[i] - lower * rhs[i - 1]) / m;
    }
    for i in (0..n - 1).rev() {
        rhs[i] -= scratch[i] * rhs[i + 1];
    }
}

/// General tridiagonal solve. `a` is the sub-diagonal (a[0] unused), `b` the diagonal, `c` the super-diagonal
/// (c[n-1] unused). Returns the solution.
pub fn solve(a: &[f64], b: &[f64], c: &[f64], d: &[f64]) -> Vec<f64> {
    let n = d.len();
    let mut cp = vec![0.0; n];
    let mut x = vec![0.0; n];
    cp[0] = c[0] / b[0];
    x[0] = d[0] / b[0];
    for i in 1..n {
        let m = b[i] - a[i] * cp[i - 1];
        cp[i] = if i + 1 < n { c[i] / m } else { 0.0 };
        x[i] = (d[i] - a[i] * x[i - 1]) / m;
    }
    for i in (0..n - 1).rev() {
        x[i] -= cp[i] * x[i + 1];
    }
    x
}
