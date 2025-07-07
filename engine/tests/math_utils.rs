use pricekit::math::rng::Rng;
use pricekit::math::stats::{covariance, Accumulator};
use pricekit::math::{interp, roots, tridiag};

#[test]
fn root_finders_agree() {
    let f = |x: f64| x * x * x - 2.0 * x - 5.0;
    let exact = 2.094_551_481_542_326_6;
    assert!((roots::bisect(f, 2.0, 3.0, 1e-13).unwrap() - exact).abs() < 1e-12);
    assert!((roots::brent(f, 2.0, 3.0, 1e-14).unwrap() - exact).abs() < 1e-12);
    assert!((roots::newton(|x| (f(x), 3.0 * x * x - 2.0), 2.5, 2.0, 3.0, 1e-14).unwrap() - exact).abs() < 1e-12);
    assert!(roots::bisect(f, 3.0, 4.0, 1e-12).is_none());
}

#[test]
fn tridiagonal_solvers() {
    let n = 7;
    let (a, b, c) = (vec![-1.0; n], vec![2.5; n], vec![-0.7; n]);
    let x_true: Vec<f64> = (0..n).map(|i| (i as f64 * 0.9).sin()).collect();
    let d: Vec<f64> = (0..n)
        .map(|i| b[i] * x_true[i] + if i > 0 { a[i] * x_true[i - 1] } else { 0.0 } + if i + 1 < n { c[i] * x_true[i + 1] } else { 0.0 })
        .collect();
    let x = tridiag::solve(&a, &b, &c, &d);
    let mut y = d.clone();
    let mut scratch = vec![0.0; n];
    tridiag::solve_constant(-1.0, 2.5, -0.7, &mut y, &mut scratch);
    for i in 0..n {
        assert!((x[i] - x_true[i]).abs() < 1e-13 && (y[i] - x_true[i]).abs() < 1e-13);
    }
}

#[test]
fn random_numbers_have_the_right_moments() {
    let mut rng = Rng::new(42);
    let (mut u, mut z, mut z2) = (Accumulator::new(), Accumulator::new(), Accumulator::new());
    for _ in 0..200_000 {
        u.add(rng.uniform());
        let n = rng.normal();
        z.add(n);
        z2.add(n * n);
    }
    assert!((u.mean() - 0.5).abs() < 4.0 * u.std_error() && (u.variance() - 1.0 / 12.0).abs() < 1e-3);
    assert!(z.mean().abs() < 4.0 * z.std_error() && (z2.mean() - 1.0).abs() < 0.02);
    let (mut a, mut b) = (Rng::new(7), Rng::new(7));
    assert_eq!(a.next_u64(), b.next_u64());
}

#[test]
fn interpolation_and_covariance() {
    let xs: Vec<f64> = (0..11).map(|i| i as f64 * 0.1).collect();
    let ys: Vec<f64> = xs.iter().map(|x| 1.0 + 2.0 * x + 3.0 * x * x).collect();
    let (v, d1, d2) = interp::quadratic(&ys, 0.0, 0.1, 0.537);
    assert!((v - (1.0 + 2.0 * 0.537 + 3.0 * 0.537 * 0.537)).abs() < 1e-12 && (d1 - (2.0 + 6.0 * 0.537)).abs() < 1e-10 && (d2 - 6.0).abs() < 1e-9);
    assert!((interp::linear(&xs, &xs, 0.234) - 0.234).abs() < 1e-15 && interp::linear(&xs, &ys, -1.0) == ys[0]);
    assert!((covariance(&xs, &xs) - 0.11).abs() < 1e-12);
}
