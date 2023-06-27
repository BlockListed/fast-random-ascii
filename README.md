# fast-random-ascii
* BLAZINGLY FAST 🚀🚀
    * 11Gib/s in my testing (to `/dev/null`).
* Multithreaded.
* Generates random ascii characters (excluding control codes) extremely quickly.
* WARNING: The random generator is NOT cryptographically secure.
    * But its reasonably statistically random [1][xorshiftrng].
* WARNING: The first 69 ascii characters are 33.3% more likely to appear. (this is probably acceptable)

# Usage
* `fast-random-ascii`
## Description:
Outputs random ascii characters to stdout.


# Performance optimizations
* TLDR thanks to the release profile, just use `RUSTFLAGS="-C target-feature=+avx2"`
* `-C lto` turns performance to shit
    * `-C embed-bitcode=yes` is not the culprit
    * `-C lto=thin` reduces performance very slighty
* disabling inline deactivation on `u8_to_ascii` VERY SLIGHTLY improves performance
* `-C opt-level=3` improves performance.
    * anything below `-C opt-level=2` shits the bed for performance
* enabling avx2 (avx is not enough) `-C target-feature=+avx2` improves performance significantly, since `u8_to_ascii` is actually a decent part of runtime usage.
* `-C strip=symbols` MAY improve performance EVER SO SLIGHTLY, but definitely doesn't decrease.

[xorshiftrng]: https://www.jstatsoft.org/v08/i14/paper