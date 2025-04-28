[![Apache License](https://img.shields.io/github/license/jonaspleyer/short_default?style=flat-square)](https://opensource.org/license/apache-2-0)
[![MIT License](https://img.shields.io/github/license/jonaspleyer/short_default?style=flat-square)](https://opensource.org/license/mit)
[![Test](https://img.shields.io/github/actions/workflow/status/jonaspleyer/short_default/test.yml?label=Test&style=flat-square)](https://github.com/jonaspleyer/short_default/actions)
[![Crate](https://img.shields.io/crates/v/short_default.svg?style=flat-square)](https://crates.io/crates/short-default)
![Crates.io Total Downloads](https://img.shields.io/crates/d/short_default?style=flat-square)
[![API](https://img.shields.io/docsrs/short_default/latest?style=flat-square)](https://docs.rs/short_default)

# short_default

Avoid writing tedious [Default](https://doc.rust-lang.org/std/default/trait.Default.html)
implementations by using a simple
[`default!`](https://docs.rs/short_default/latest/short_default/macro.default.html) macro instead.

```Rust
use short_default::default;

default! {
    struct Config {
        version: (u64, u64, u64) = (0, 1, 0),
        // This default value will be inferred via
        // authors: Default::default(),
        authors: Vec<String>,
    }
}
```
