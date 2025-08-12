"""All experiments, checks and the report."""

from __future__ import annotations

import json
from pathlib import Path

import numpy as np
import pandas as pd
import yaml

from . import engine, plotting
from .engine import ROOT
from .report import md_table


def _check(checks, area, name, ok, detail, warn=False):
    checks.append(
        {"area": area, "check": name, "status": "PASS" if ok else ("WARN" if warn else "FAIL"), "detail": detail}
    )


def _order(sizes, errors):
    return float(-np.polyfit(np.log(sizes), np.log(np.abs(errors)), 1)[0])


def run(cfg_path=None, out=None, fast: bool = False) -> dict:
    cfg = yaml.safe_load(open(cfg_path or ROOT / "configs" / "default.yaml"))
    out = Path(out) if out else ROOT / "reports"
    (out / "figures").mkdir(parents=True, exist_ok=True)
    if not engine.available():
        raise SystemExit("Rust library not built: run `make build` first")
    mkt, t, seed = cfg["market"], cfg["maturity"], cfg["seed"]
    paths = cfg["paths"] // (8 if fast else 1)
    tree_steps = cfg["binomial_steps"][: 5 if fast else None]
    grids = cfg["pde_grids"][: 4 if fast else None]
    checks: list[dict] = []

    # 1. one contract, four engines
    rows = {}
    for kind in ("call", "put"):
        for k in cfg["strikes"]:
            r = engine.call(
                "vanilla", **mkt, strike=k, maturity=t, kind=kind, steps=2000, grid=400, paths=paths, seed=seed
            )
            ref = engine.black_scholes(**mkt, strike=k, maturity=t, kind=kind)
            rows[(kind, k)] = {
                "closed form": r["price"],
                "against SciPy": r["price"] - ref,
                "tree, 2,000 steps": r["binomial"] - r["price"],
                "grid, extrapolated": r["pde_extrapolated"] - r["price"],
                "Monte Carlo": r["mc"] - r["price"],
                "standard error": r["mc_std_error"],
                "z": (r["mc"] - r["price"]) / r["mc_std_error"],
            }
    four = pd.DataFrame(rows).T
    four.index.names = ["kind", "strike"]
    _check(
        checks,
        "closed form",
        "matches an independent implementation",
        four["against SciPy"].abs().max() < 1e-11,
        f"largest difference {four['against SciPy'].abs().max():.1e}",
    )
    _check(
        checks,
        "tree",
        "within 0.01 of the closed form at 2,000 steps",
        four["tree, 2,000 steps"].abs().max() < 0.01,
        f"largest error {four['tree, 2,000 steps'].abs().max():.1e}",
    )
    _check(
        checks,
        "grid",
        "extrapolated grid within 1e-4 of the closed form",
        four["grid, extrapolated"].abs().max() < 1e-4,
        f"largest error {four['grid, extrapolated'].abs().max():.1e}",
    )
    _check(
        checks,
        "Monte Carlo",
        "every estimate within 4 standard errors",
        four["z"].abs().max() < 4,
        f"largest |z| {four['z'].abs().max():.2f}",
    )

    # 2. convergence of the two deterministic engines on the at-the-money put
    base = dict(**mkt, strike=100.0, maturity=t, kind="put", paths=10)
    exact = engine.call("vanilla", **base)["price"]
    tree_err = np.array([engine.call("vanilla", **base, steps=n, grid=20)["binomial"] - exact for n in tree_steps])
    grid_err = np.array([engine.call("vanilla", **base, steps=10, grid=n)["pde"] - exact for n in grids])
    orders = {"tree": _order(tree_steps, tree_err), "grid": _order(grids, grid_err)}
    conv = pd.DataFrame(
        {
            "tree steps": pd.Series(tree_steps),
            "tree error": pd.Series(tree_err),
            "grid intervals": pd.Series(grids),
            "grid error": pd.Series(grid_err),
        }
    )
    _check(checks, "tree", "first-order convergence", 0.8 < orders["tree"] < 1.2, f"fitted order {orders['tree']:.2f}")
    _check(checks, "grid", "second-order convergence", 1.8 < orders["grid"] < 2.2, f"fitted order {orders['grid']:.2f}")

    # 3. sensitivities three ways
    g = engine.call("vanilla", **mkt, strike=100.0, maturity=t, kind="call", grid=800, paths=10)
    greeks = pd.DataFrame(
        {
            "analytic": {k: g[k] for k in ("delta", "gamma", "vega", "rho")},
            "bump and reprice": {k: g[f"bump_{k}"] for k in ("delta", "gamma", "vega", "rho")},
            "read off the grid": {"delta": g["pde_delta"], "gamma": g["pde_gamma"], "vega": np.nan, "rho": np.nan},
        }
    )
    bump_gap = float((greeks["bump and reprice"] / greeks["analytic"] - 1).abs().max())
    grid_gap = float((greeks["read off the grid"] / greeks["analytic"] - 1).abs().max())
    _check(
        checks,
        "sensitivities",
        "bumping agrees with the formulas to 1e-5 relative",
        bump_gap < 1e-5,
        f"largest relative gap {bump_gap:.1e}",
    )
    _check(
        checks,
        "sensitivities",
        "grid delta and gamma within 1%",
        grid_gap < 0.01,
        f"largest relative gap {grid_gap:.1e}",
    )

    # 4. American put: grid against tree
    am_rows = {}
    for s in cfg["american_spots"]:
        r = engine.call(
            "american",
            **{**mkt, "spot": s},
            strike=100.0,
            maturity=t,
            grid=200 if fast else 400,
            steps=1000 if fast else 4000,
        )
        am_rows[s] = {
            "grid, extrapolated": r["pde_extrapolated"],
            "tree, extrapolated": r["binomial_extrapolated"],
            "difference": r["pde_extrapolated"] - r["binomial_extrapolated"],
            "European": r["european"],
            "early-exercise premium": r["pde_extrapolated"] - r["european"],
            "delta": r["pde_delta"],
        }
    am = pd.DataFrame(am_rows).T
    am.index.name = "spot"
    _check(
        checks,
        "American put",
        "grid and tree agree to 2e-3",
        am["difference"].abs().max() < 2e-3,
        f"largest difference {am['difference'].abs().max():.1e}",
    )
    _check(
        checks,
        "American put",
        "premium positive and price above intrinsic value",
        bool(
            (am["early-exercise premium"] > 0).all()
            and (am["grid, extrapolated"] >= np.maximum(100.0 - am.index, 0) - 1e-9).all()
        ),
        f"smallest premium {am['early-exercise premium'].min():.4f}",
    )

    # 5. down-and-out call
    ba_rows = {}
    for h in cfg["barriers"]:
        r = engine.call(
            "barrier",
            **mkt,
            strike=100.0,
            maturity=t,
            barrier=h,
            grid=400,
            steps=cfg["barrier_steps"],
            paths=paths,
            seed=seed,
        )
        ba_rows[h] = {
            "closed form": r["down_and_out"],
            "share of the vanilla": r["down_and_out"] / r["vanilla"],
            "in + out - vanilla": r["down_and_out"] + r["down_and_in"] - r["vanilla"],
            "grid error": r["pde_extrapolated"] - r["down_and_out"],
            "Monte Carlo error": r["mc"] - r["down_and_out"],
            "standard error": r["mc_std_error"],
            "z": (r["mc"] - r["down_and_out"]) / r["mc_std_error"],
        }
    ba = pd.DataFrame(ba_rows).T
    ba.index.name = "barrier"
    _check(
        checks,
        "barrier",
        "knock-in plus knock-out equals the vanilla",
        ba["in + out - vanilla"].abs().max() < 1e-11,
        f"largest gap {ba['in + out - vanilla'].abs().max():.1e}",
    )
    _check(
        checks,
        "barrier",
        "grid within 2e-3 of the closed form",
        ba["grid error"].abs().max() < 2e-3,
        f"largest error {ba['grid error'].abs().max():.1e}",
    )
    _check(
        checks,
        "barrier",
        f"bridge-corrected Monte Carlo unbiased with {cfg['barrier_steps']} steps",
        ba["z"].abs().max() < 4,
        f"largest |z| {ba['z'].abs().max():.2f}",
    )

    # 6. arithmetic Asian call with the geometric control variate
    a = engine.call("asian", **mkt, strike=100.0, maturity=t, dates=cfg["asian_dates"], paths=paths // 4, seed=seed)
    ratio = a["plain_std_error"] / a["control_std_error"]
    _check(
        checks,
        "Asian",
        "control variate agrees with the plain estimate",
        abs(a["arithmetic_plain"] - a["arithmetic_control"]) < 4 * a["plain_std_error"],
        f"{a['arithmetic_plain']:.4f} against {a['arithmetic_control']:.4f}",
    )
    _check(
        checks, "Asian", "control variate cuts the standard error at least 20 times", ratio > 20, f"{ratio:.0f} times"
    )

    # 7. lookback call with exact sampling of the minimum
    lb_rows = {}
    for n in cfg["lookback_steps"]:
        r = engine.call("lookback", **mkt, maturity=t, steps=n, paths=paths // 2, seed=seed)
        lb_rows[n] = {
            "closed form": r["closed_form"],
            "Monte Carlo": r["mc"],
            "standard error": r["mc_std_error"],
            "z": (r["mc"] - r["closed_form"]) / r["mc_std_error"],
        }
    lb = pd.DataFrame(lb_rows).T
    lb.index.name = "time steps"
    _check(
        checks,
        "lookback",
        "unbiased for every number of time steps",
        lb["z"].abs().max() < 4,
        f"largest |z| {lb['z'].abs().max():.2f}",
    )

    # 8. implied volatility: price in Python, invert in Rust
    iv_rows = {}
    for v in cfg["implied"]["vols"]:
        for k in cfg["implied"]["strikes"]:
            kind = "call" if k >= 100.0 else "put"
            price = engine.black_scholes(100.0, k, 0.0, 0.0, v, 1.0, kind)
            got = engine.call("implied", price=repr(price), forward=100.0, strike=k, maturity=1.0, kind=kind)["vol"]
            iv_rows[(v, k)] = {
                "price": price,
                "recovered": got,
                "relative error": got / v - 1 if price > 1e-250 else np.nan,
            }
    iv = pd.DataFrame(iv_rows).T
    iv.index.names = ["volatility", "strike"]
    usable = iv[iv["price"] > 1e-12]
    _check(
        checks,
        "implied volatility",
        "round trip to 1e-8 wherever the price exceeds 1e-12",
        usable["relative error"].abs().max() < 1e-8,
        f"largest relative error {usable['relative error'].abs().max():.1e} over {len(usable)} of {len(iv)} quotes",
    )

    # 9. SABR smiles
    sb = cfg["sabr"]
    smiles = {}
    for rho in sb["rhos"]:
        lo, hi, alpha = 1e-4, 5.0, None
        for _ in range(60):  # at-the-money volatility is increasing in alpha: bisection
            alpha = 0.5 * (lo + hi)
            v = engine.call(
                "sabr",
                alpha=alpha,
                beta=sb["beta"],
                rho=rho,
                nu=sb["nu"],
                forward=sb["forward"],
                strike=sb["forward"],
                maturity=t,
            )["vol"]
            lo, hi = (alpha, hi) if v < sb["atm_vol"] else (lo, alpha)
        smiles[rho] = [
            engine.call(
                "sabr", alpha=alpha, beta=sb["beta"], rho=rho, nu=sb["nu"], forward=sb["forward"], strike=k, maturity=t
            )["vol"]
            for k in sb["strikes"]
        ]
    smile = pd.DataFrame(smiles, index=sb["strikes"])
    smile.index.name = "strike"
    smile.columns = [f"rho {r:+.1f}" for r in sb["rhos"]]
    mid = sb["strikes"].index(sb["forward"])
    skew = (smile.iloc[mid + 1] - smile.iloc[mid - 1]).to_numpy()
    _check(
        checks,
        "SABR",
        "all smiles share the at-the-money volatility",
        float((smile.iloc[mid] - sb["atm_vol"]).abs().max()) < 1e-9,
        f"largest gap {float((smile.iloc[mid] - sb['atm_vol']).abs().max()):.1e}",
    )
    _check(
        checks,
        "SABR",
        "skew increases with the correlation",
        bool(np.all(np.diff(skew) > 0)),
        "slope at the money: " + ", ".join(f"{x:+.4f}" for x in skew),
    )

    plotting.overview(tree_steps, tree_err, grids, grid_err, am, ba, smile, out / "figures" / "overview.png")

    val = pd.DataFrame(checks)
    counts = val["status"].value_counts().to_dict()
    L = [
        "# PriceKit report",
        "",
        f"Market: spot {mkt['spot']:g}, rate {mkt['rate']:.0%}, dividend yield {mkt['dividend']:.0%}, volatility {mkt['vol']:.0%}. Maturity {t:g} year. Monte Carlo: {2 * paths:,d} antithetic paths.",
        "",
        "## 1. European options, four engines",
        "",
        "Columns after the first are differences from the closed form.",
        "",
        md_table(four, 5),
        "",
        "## 2. Convergence (at-the-money put)",
        "",
        md_table(conv, 6, index=False),
        "",
        f"Fitted orders: tree {orders['tree']:.2f}, grid {orders['grid']:.2f}.",
        "",
        "## 3. Sensitivities (at-the-money call)",
        "",
        md_table(greeks, 6),
        "",
        "## 4. American put, strike 100",
        "",
        md_table(am, 4),
        "",
        "## 5. Down-and-out call, strike 100",
        "",
        f"Monte Carlo uses {cfg['barrier_steps']} time steps and the Brownian-bridge crossing probability between them.",
        "",
        md_table(ba, 5),
        "",
        "## 6. Arithmetic Asian call",
        "",
        f"{cfg['asian_dates']} averaging dates, {2 * (paths // 4):,d} paths. Plain: {a['arithmetic_plain']:.5f} +- {a['plain_std_error']:.5f}. "
        f"With the geometric control variate: {a['arithmetic_control']:.5f} +- {a['control_std_error']:.5f}, a standard error {ratio:.0f} times smaller, "
        f"worth {ratio**2:,.0f} times as many paths. Geometric closed form (discrete): {a['geometric_discrete']:.5f}.",
        "",
        "## 7. Floating-strike lookback call",
        "",
        md_table(lb, 5),
        "",
        "## 8. Implied volatility round trip",
        "",
        md_table(iv, 6),
        "",
        "## 9. SABR smiles at fixed at-the-money volatility",
        "",
        md_table(smile, 4),
        "",
        "## Validation",
        "",
        f"{counts.get('PASS', 0)} PASS, {counts.get('WARN', 0)} WARN, {counts.get('FAIL', 0)} FAIL. See `validation.md`.",
        "",
    ]
    (out / "report.md").write_text("\n".join(L))
    (out / "validation.md").write_text("# Validation checks\n\n" + md_table(val, index=False) + "\n")
    metrics = {
        "four_engines": {f"{k[0]} {k[1]:g}": v for k, v in four.to_dict("index").items()},
        "orders": orders,
        "greeks": greeks.to_dict(),
        "american": am.to_dict("index"),
        "barrier": ba.to_dict("index"),
        "asian": {k: v for k, v in a.items() if k != "seconds"},
        "asian_error_ratio": ratio,
        "lookback": lb.to_dict("index"),
        "implied_worst": float(usable["relative error"].abs().max()),
        "smile": smile.to_dict(),
        "checks": checks,
    }
    (out / "metrics.json").write_text(json.dumps(metrics, indent=1, default=float))
    return metrics
