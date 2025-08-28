# Methods

Notation: spot S, strike K, rate r, dividend yield q, volatility v, maturity T, forward F = S exp((r - q) T).

## Normal distribution

`erfc` uses the Taylor series of erf below 2.5 and a continued fraction above, both run to convergence. The cumulative distribution is `0.5 erfc(-x / sqrt 2)`, accurate in the far tail because nothing is subtracted from one. The inverse starts from a rational approximation and is polished with one Halley step on the exact function.

## Closed forms

- Vanilla and digitals: Black-Scholes with continuous dividend yield. A call is an asset-or-nothing minus K cash-or-nothing digitals, which is tested.
- Down barrier calls (barrier H at or below K): reflection principle. The knock-in is `S exp(-qT) (H/S)^(2 lambda) N(y) - K exp(-rT) (H/S)^(2 lambda - 2) N(y - v sqrt T)` with `lambda = (r - q) / v^2 + 1/2` and `y = ln(H^2 / (S K)) / (v sqrt T) + lambda v sqrt T`. The knock-out is the vanilla minus the knock-in.
- Geometric Asian call: the geometric mean of lognormals is lognormal. Continuous monitoring has variance `v^2 T / 3`; with n equally spaced dates the mean and variance of the log average are sums over the dates, written out in `asian.rs`.
- Floating-strike lookback call: Goldman, Sosin and Gatto formula for a minimum starting at the spot.

## Binomial tree

Cox-Ross-Rubinstein, up factor `exp(v sqrt dt)`. The early-exercise test is applied at every node. `price_extrapolated` averages n and n+1 steps to cancel the odd-even oscillation, then extrapolates between n and 2n assuming first order.

## Finite differences

Log-price grid of half width `max(6 v sqrt T, ln 4)` around the strike, the strike on a node. Crank-Nicolson in time, with the first two steps replaced by four implicit Euler half steps so the kink in the payoff does not ring. American puts use projection in the back-substitution of the Thomas solver, which is exact for this matrix when the exercise region is a single interval at the low end. A knock-out puts its barrier on the first node, with the spacing chosen so that the strike is also a node. Price, delta and gamma come from a parabola through the three nodes nearest the spot.

`solve_extrapolated` combines N and 2N intervals as `(4 fine - coarse) / 3`.

## Monte Carlo

Exact lognormal steps, antithetic pairs, standard error from the pair averages.

- Asian: the control variate is the geometric average on the same path, whose expectation is the discrete closed form. The coefficient is estimated from the sample.
- Lookback: given the log-price at both ends of a step, the minimum in between is `(a + b - sqrt((b - a)^2 - 2 v^2 dt ln U)) / 2` with U uniform. This is the exact conditional law.
- Barrier: the path survives a step with probability `1 - exp(-2 ln(S1/H) ln(S2/H) / (v^2 dt))`. The payoff is weighted by the product of these probabilities.

## Implied volatility

Work with `x = ln(F/K)` folded to the out-of-the-money side and the price divided by `sqrt(F K)`. The unknown is the total volatility `s = v sqrt T`. The solver starts from a closed-form guess, then iterates Halley steps on the logarithm of the price inside a bracket that tightens at every evaluation, with bisection as the fallback. Prices outside the no-arbitrage bounds return nothing.

## SABR

Lognormal implied volatility from the expansion of Hagan, Kumar, Lesniewski and Woodward, with the at-the-money limit taken analytically. `alpha_from_atm` solves the cubic for alpha that reproduces a given at-the-money volatility. `calibrate` fixes beta, eliminates alpha through the at-the-money quote and searches rho and nu on a 21 by 21 grid that is re-centred on the best point and shrunk four times.
