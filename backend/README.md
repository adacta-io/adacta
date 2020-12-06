# Adacta Backend

The Backend and REST server of Adacta.

## Dependencies

To build the backend, `rust` and `cargo` are required.
Currently, nightly rust compilers newer than 2020-11-09 are supported.

## Build

Run `cargo build` to build the project. The build artifacts will be stored in the `target/` directory. Use the `--release` flag for a production build.

## Bundling the Frontend

The [frontend](../frontend) must be build before the backend.

On a production build, the frontend is embedded inside the resulting backend executable.
On a development build, the frontend is served from disk.

