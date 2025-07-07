//! One-dimensional root finding.

/// Bisection on a bracket [lo, hi] with a sign change. Returns None if there is none.
pub fn bisect<F: Fn(f64) -> f64>(f: F, mut lo: f64, mut hi: f64, tol: f64) -> Option<f64> {
    let (mut flo, fhi) = (f(lo), f(hi));
    if flo == 0.0 {
        return Some(lo);
    }
    if fhi == 0.0 {
        return Some(hi);
    }
    if flo.signum() == fhi.signum() {
        return None;
    }
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        let fm = f(mid);
        if fm == 0.0 || hi - lo < tol {
            return Some(mid);
        }
        if fm.signum() == flo.signum() {
            lo = mid;
            flo = fm;
        } else {
            hi = mid;
        }
    }
    Some(0.5 * (lo + hi))
}

/// Brent's method: inverse quadratic interpolation with a bisection safeguard.
pub fn brent<F: Fn(f64) -> f64>(f: F, lo: f64, hi: f64, tol: f64) -> Option<f64> {
    let (mut a, mut b) = (lo, hi);
    let (mut fa, mut fb) = (f(a), f(b));
    if fa.signum() == fb.signum() && fa != 0.0 && fb != 0.0 {
        return None;
    }
    if fa.abs() < fb.abs() {
        std::mem::swap(&mut a, &mut b);
        std::mem::swap(&mut fa, &mut fb);
    }
    let (mut c, mut fc, mut d, mut flag) = (a, fa, a, true);
    for _ in 0..200 {
        if fb == 0.0 || (b - a).abs() < tol {
            return Some(b);
        }
        let mut s = if fa != fc && fb != fc {
            a * fb * fc / ((fa - fb) * (fa - fc)) + b * fa * fc / ((fb - fa) * (fb - fc)) + c * fa * fb / ((fc - fa) * (fc - fb))
        } else {
            b - fb * (b - a) / (fb - fa)
        };
        let between = (s - (3.0 * a + b) / 4.0) * (s - b) < 0.0;
        if !between || (flag && (s - b).abs() >= (b - c).abs() / 2.0) || (!flag && (s - b).abs() >= (c - d).abs() / 2.0) {
            s = 0.5 * (a + b);
            flag = true;
        } else {
            flag = false;
        }
        let fs = f(s);
        d = c;
        c = b;
        fc = fb;
        if fa.signum() != fs.signum() {
            b = s;
            fb = fs;
        } else {
            a = s;
            fa = fs;
        }
        if fa.abs() < fb.abs() {
            std::mem::swap(&mut a, &mut b);
            std::mem::swap(&mut fa, &mut fb);
        }
    }
    Some(b)
}

/// Newton iterations with a derivative, falling back to None when they leave [lo, hi] or stall.
pub fn newton<F: Fn(f64) -> (f64, f64)>(f: F, start: f64, lo: f64, hi: f64, tol: f64) -> Option<f64> {
    let mut x = start;
    for _ in 0..100 {
        let (v, dv) = f(x);
        if dv == 0.0 {
            return None;
        }
        let step = v / dv;
        x -= step;
        if !(x > lo && x < hi) {
            return None;
        }
        if step.abs() < tol {
            return Some(x);
        }
    }
    None
}
