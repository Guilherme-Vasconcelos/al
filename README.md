# al
al (**a**wful **l**isp) is a simple lisp implementation made for fun.

## Building
You will need a rust toolchain, then simply run `$ cargo build` (or `$ cargo build --release` for a more optimized release version).

Any modern rust version should work, but if any problems arise use the version indicated in [Cargo.toml](./Cargo.toml), as the project has been more extensively tested with that version.

## Development utils
- [tools/format_lint_test.sh](./tools/format_lint_test.sh): As the name suggests, runs all the available checks and also formats the source code. This script is meant to be run frequently.

## License
Licensed under the GNU General Public License, either version 3 or any later versions at your choice.
