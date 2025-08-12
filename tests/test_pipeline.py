import json

import pytest

from pricekit import engine
from pricekit.pipeline import run


@pytest.mark.skipif(not engine.available(), reason="Rust library not built")
def test_fast_pipeline(tmp_path):
    m = run(out=tmp_path, fast=True)
    assert not [c for c in m["checks"] if c["status"] == "FAIL"]
    assert (tmp_path / "report.md").exists() and (tmp_path / "figures" / "overview.png").exists()
    assert json.loads((tmp_path / "metrics.json").read_text())["orders"]["grid"] > 1.8
