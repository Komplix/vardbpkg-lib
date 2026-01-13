# vardbpkg
[<img alt="github" src="https://img.shields.io/badge/github-Komplix%2Fvardbpkg--lib-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/Komplix/vardbpkg-lib)
[<img alt="crates.io" src="https://img.shields.io/crates/v/vardbpkg.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/vardbpkg)
[![Build](https://github.com/Komplix/vardbpkg-lib/actions/workflows/build.yml/badge.svg)](https://github.com/Komplix/vardbpkg-lib/actions/workflows/build.yml)
![maintenance-status](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)

A Rust library to parse and search the `Gentoo Linux` Portage Package installed database `/var/db/pkg`.

## Usage

```rust
use vardbpkg::parse_vardb;
use std::path::Path;

fn main() {
    let packages = parse_vardb(Path::new("/var/db/pkg"));

    for pkg in packages {
        println!("{}/{}: {}", pkg.category, pkg.package, pkg.description);
    }
}
```

## Examples

### vardbpkg2json

The library includes an example tool `vardbpkg2json` that converts the Portage database to JSON format.

```bash
cargo run --example vardbpkg2json -- /var/db/pkg
```

If no directory is specified, it defaults to `/var/db/pkg`.

## License
Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.



