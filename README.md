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