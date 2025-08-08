"""Python side of the Rust library: runs the command line binary and parses its `key value` output."""

from __future__ import annotations

import subprocess
from pathlib import Path

import numpy as np
from scipy.stats import norm

ROOT = Path(__file__).resolve().parents[2]
BINARY = ROOT / "engine" / "target" / "release" / "pricekit"


def available() -> bool:
    return BINARY.exists()


def call(command: str, **params) -> dict[str, float]:
    """`call("vanilla", spot=100, strike=105, vol=0.2, maturity=1)` -> {"price": ..., "delta": ..., ...}"""
    args = [f"{k}={v}" for k, v in params.items()]
    out = subprocess.run([str(BINARY), command, *args], capture_output=True, text=True, check=True).stdout
    return {k: float(v) for k, v in (line.split() for line in out.splitlines())}


def black_scholes(spot, strike, rate, dividend, vol, maturity, kind: str = "call") -> float:
    """Independent closed form (SciPy normal distribution), used to check the Rust one."""
    sd = vol * np.sqrt(maturity)
    d1 = (np.log(spot / strike) + (rate - dividend) * maturity) / sd + 0.5 * sd
    w = 1.0 if kind == "call" else -1.0
    fwd, disc = spot * np.exp(-dividend * maturity), strike * np.exp(-rate * maturity)
    return float(w * (fwd * norm.cdf(w * d1) - disc * norm.cdf(w * (d1 - sd))))
