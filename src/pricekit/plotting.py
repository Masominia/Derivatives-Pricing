"""Static figures for the report (matplotlib, PNG for the README)."""

from __future__ import annotations

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402
import numpy as np  # noqa: E402

# Validated categorical order (fixed, never cycled) + recessive ink.
SERIES = ["#2a78d6", "#eb6834", "#1baf7a", "#eda100", "#e87ba4", "#008300", "#6250d6", "#e34948"]
SURFACE, INK, INK2, GRID = "#fcfcfb", "#0b0b0b", "#52514e", "#e4e3df"
NEUTRAL = "#8a8985"
CRITICAL = "#c4271a"

plt.rcParams.update(
    {
        "figure.facecolor": SURFACE,
        "axes.facecolor": SURFACE,
        "savefig.facecolor": SURFACE,
        "axes.edgecolor": GRID,
        "axes.labelcolor": INK2,
        "xtick.color": INK2,
        "ytick.color": INK2,
        "text.color": INK,
        "axes.grid": True,
        "grid.color": GRID,
        "grid.linewidth": 0.6,
        "axes.spines.top": False,
        "axes.spines.right": False,
        "font.size": 9,
        "axes.titlesize": 10,
        "axes.titleweight": "bold",
        "legend.frameon": False,
        "lines.linewidth": 1.6,
        "figure.dpi": 150,
        "axes.axisbelow": True,
    }
)


def _save(fig, path):
    fig.tight_layout()
    fig.savefig(path, bbox_inches="tight")
    plt.close(fig)


def overview(tree_steps, tree_err, grids, grid_err, am, ba, smile, path):
    fig, ax = plt.subplots(2, 2, figsize=(10.5, 7.2))
    a = ax[0, 0]
    a.loglog(tree_steps, np.abs(tree_err), "o-", color=SERIES[0], ms=4, label="binomial tree, steps")
    a.loglog(grids, np.abs(grid_err), "s-", color=SERIES[1], ms=4, label="finite differences, intervals")
    a.set(title="Convergence, at-the-money put", xlabel="size", ylabel="absolute error")
    a.legend()

    a = ax[0, 1]
    a.plot(am.index, am["grid, extrapolated"], "o-", color=SERIES[0], ms=4, label="American")
    a.plot(am.index, am["European"], "s-", color=SERIES[1], ms=4, label="European")
    a.plot(am.index, np.maximum(100.0 - am.index, 0.0), color=NEUTRAL, lw=1, ls="--", label="exercise value")
    a.set(title="Put, strike 100", xlabel="spot", ylabel="price")
    a.legend()

    a = ax[1, 0]
    a.plot(ba.index, ba["closed form"], color=SERIES[0], label="closed form")
    a.plot(ba.index, ba["closed form"] + ba["grid error"], "s", color=SERIES[1], ms=5, label="finite differences")
    a.errorbar(
        ba.index,
        ba["closed form"] + ba["Monte Carlo error"],
        yerr=2 * ba["standard error"],
        fmt="o",
        color=SERIES[2],
        ms=3,
        capsize=3,
        label="Monte Carlo, 2 standard errors",
    )
    a.set(title="Down-and-out call, strike 100", xlabel="barrier", ylabel="price")
    a.legend()

    a = ax[1, 1]
    for i, col in enumerate(smile.columns):
        a.plot(smile.index, 100 * smile[col], "o-", color=SERIES[i], ms=3, label=col)
    a.set(title="SABR smiles, same at-the-money volatility", xlabel="strike", ylabel="implied volatility, %")
    a.legend()
    _save(fig, path)
