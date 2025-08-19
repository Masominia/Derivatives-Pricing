"""Price one contract from the command line.

python scripts/price.py vanilla spot=100 strike=105 vol=0.2 rate=0.05 maturity=1 kind=put
python scripts/price.py american spot=95 strike=100 vol=0.25 rate=0.05 maturity=0.5
python scripts/price.py barrier spot=100 strike=100 barrier=90 vol=0.25 rate=0.05 maturity=1
"""

import sys

from pricekit import engine

if __name__ == "__main__":
    if len(sys.argv) < 2:
        raise SystemExit(__doc__)
    params = dict(a.split("=", 1) for a in sys.argv[2:])
    for key, value in engine.call(sys.argv[1], **params).items():
        print(f"{key:<24}{value:.8g}")
