import numpy as np
import pytest

from pricekit import engine

pytestmark = pytest.mark.skipif(not engine.available(), reason="Rust library not built")
MKT = {"spot": 100.0, "rate": 0.03, "dividend": 0.01, "vol": 0.3}


@pytest.mark.parametrize("kind", ["call", "put"])
@pytest.mark.parametrize("strike", [70.0, 100.0, 150.0])
def test_closed_form_matches_scipy(kind, strike):
    r = engine.call("vanilla", **MKT, strike=strike, maturity=2.0, kind=kind, paths=10, grid=20, steps=10)
    assert abs(r["price"] - engine.black_scholes(**MKT, strike=strike, maturity=2.0, kind=kind)) < 1e-11


def test_engines_agree():
    r = engine.call("vanilla", **MKT, strike=110.0, maturity=1.0, paths=100000, grid=200, steps=1000)
    assert abs(r["binomial"] - r["price"]) < 0.02
    assert abs(r["pde_extrapolated"] - r["price"]) < 1e-4
    assert abs(r["mc"] - r["price"]) < 4 * r["mc_std_error"]
    assert abs(r["bump_delta"] - r["delta"]) < 1e-6 and abs(r["pde_delta"] - r["delta"]) < 1e-3


def test_american_and_barrier():
    a = engine.call("american", **MKT, strike=100.0, maturity=1.0, grid=200, steps=1000)
    assert a["pde_extrapolated"] > a["european"] and abs(a["pde_extrapolated"] - a["binomial_extrapolated"]) < 3e-3
    b = engine.call("barrier", **MKT, strike=100.0, maturity=1.0, barrier=85.0, paths=1000)
    assert abs(b["down_and_out"] + b["down_and_in"] - b["vanilla"]) < 1e-11
    assert abs(b["pde_extrapolated"] - b["down_and_out"]) < 2e-3


def test_control_variate_and_lookback():
    a = engine.call("asian", **MKT, strike=100.0, maturity=1.0, paths=20000)
    assert a["control_std_error"] < a["plain_std_error"] / 15
    lb = engine.call("lookback", **MKT, maturity=1.0, steps=8, paths=100000)
    assert abs(lb["mc"] - lb["closed_form"]) < 4 * lb["mc_std_error"]


def test_implied_volatility_and_sabr():
    price = engine.black_scholes(100.0, 130.0, 0.0, 0.0, 0.4, 0.5)
    got = engine.call("implied", price=repr(price), forward=100.0, strike=130.0, maturity=0.5)["vol"]
    assert abs(got - 0.4) < 1e-9
    assert np.isnan(engine.call("implied", price=200.0, forward=100.0, strike=130.0, maturity=0.5)["vol"])
    flat = engine.call("sabr", alpha=0.3, beta=1.0, rho=0.0, nu=0.0, forward=100.0, strike=80.0, maturity=1.0)["vol"]
    assert abs(flat - 0.3) < 1e-9
