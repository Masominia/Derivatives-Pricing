import pandas as pd

from pricekit.report import md_table


def test_markdown_table():
    df = pd.DataFrame({"a": [1, 2], "b": [0.5, 1e-9]}, index=["x", "y"])
    text = md_table(df, 3)
    assert text.splitlines()[1].count("---") == 3 and "1.00e-09" in text and "| x | 1 | 0.500 |" in text
