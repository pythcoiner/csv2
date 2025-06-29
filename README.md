# A tool to do some experiments with bitcoin relative locktimes

This binary is a helper to build & spend "anyone can spend after timelock" coins.
It's not intended to be used in real-world conditions but as a helper for experimenting
with the possibility to extend Bitcoin's relative locktime by adding a new flag to the nSequence field.

# Usage

- `csv2 address <timelock>`  Generate a P2WSH address with the specified timelock
- `csv2 spend <outpoint> <sat_amount_to_spend> <address>`  Spend from the specified outpoint
- `csv2 -help` or `csv2 -h`  Show this help message

