# Validation checks

| area | check | status | detail |
|---|---|---|---|
| closed form | matches an independent implementation | PASS | largest difference 2.5e-14 |
| tree | within 0.01 of the closed form at 2,000 steps | PASS | largest error 1.2e-03 |
| grid | extrapolated grid within 1e-4 of the closed form | PASS | largest error 6.0e-06 |
| Monte Carlo | every estimate within 4 standard errors | PASS | largest |z| 0.37 |
| tree | first-order convergence | PASS | fitted order 1.00 |
| grid | second-order convergence | PASS | fitted order 2.00 |
| sensitivities | bumping agrees with the formulas to 1e-5 relative | PASS | largest relative gap 8.6e-07 |
| sensitivities | grid delta and gamma within 1% | PASS | largest relative gap 2.8e-05 |
| American put | grid and tree agree to 2e-3 | PASS | largest difference 1.5e-04 |
| American put | premium positive and price above intrinsic value | PASS | smallest premium 0.0810 |
| barrier | knock-in plus knock-out equals the vanilla | PASS | largest gap 3.6e-15 |
| barrier | grid within 2e-3 of the closed form | PASS | largest error 2.0e-06 |
| barrier | bridge-corrected Monte Carlo unbiased with 10 steps | PASS | largest |z| 1.32 |
| Asian | control variate agrees with the plain estimate | PASS | 6.3702 against 6.3727 |
| Asian | control variate cuts the standard error at least 20 times | PASS | 30 times |
| lookback | unbiased for every number of time steps | PASS | largest |z| 0.55 |
| implied volatility | round trip to 1e-8 wherever the price exceeds 1e-12 | PASS | largest relative error 1.0e-14 over 19 of 25 quotes |
| SABR | all smiles share the at-the-money volatility | PASS | largest gap 1.1e-16 |
| SABR | skew increases with the correlation | PASS | slope at the money: -0.0515, -0.0313, -0.0107, +0.0103 |
