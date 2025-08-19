"""Print a SABR smile:  python scripts/smile.py [alpha beta rho nu]"""

import sys

from pricekit import engine

if __name__ == "__main__":
    alpha, beta, rho, nu = (float(x) for x in (sys.argv[1:5] if len(sys.argv) >= 5 else (2.0, 0.5, -0.35, 0.7)))
    for strike in range(60, 145, 5):
        v = engine.call("sabr", alpha=alpha, beta=beta, rho=rho, nu=nu, forward=100.0, strike=strike, maturity=1.0)[
            "vol"
        ]
        print(f"{strike:>5d}  {100 * v:6.2f}%  " + "#" * int(round(200 * v)))
