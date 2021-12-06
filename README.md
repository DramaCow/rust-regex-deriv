# rust-regex-deriv

A barebones rust implementation of regular expressions using regular expression derivatives.

# What's included

- A programmatic interface of constructing regular expression trees that makes use of "smart-constructors". Supports the canonical regex operations: concatenation, star, plus, alternation. Also supports: complement, intersection (it can easily be shown regexes are closed under such operations).
- Support for the titular "derivative" operation.
- Approximate equivalence relation between `RegEx`s
- DFA construction from single regexes or "regex vectors"
- DFA minimization via Hopcroft's algorithm
- Scanner table construction
- A `Scan` iterator driven by a scanner table that yields tokens.
