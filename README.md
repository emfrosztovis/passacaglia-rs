# `passacaglia`, Op.1

> Dedicated to Nikolai Y. Myaskovsky

Music theory and procedural music generation libraries in Rust.

The codebase has been ported from TypeScript to Rust and this is now considered the definitive version.

The project is in active, early-stage development. 

- [`common`](./crates/common/) contains shared utility classes, types and functions that are not related to music. Most importantly, it provides `Rational` and `HashMap` which are used everywhere.

- [`core`](./crates/core/): structs for basic musical elements `Pitch`, `Interval`, `Scale`, `Degree` etc. These come with a type parameter `S: PitchSystem`, which means they're agnostic on the musical system used, as long as the system consists of a fixed frequency period and a number of pitch classes. The crate also provides 1) definitions for the standatd heptatonic pitch system, with some convenient utilities and 2) traits for generic musical structures (or “containers”, like measures, voices etc) which can be navigated with cursors.

- [`macros`](./crates/macros/): macros for generating compile-time musical constants.

- [`musicxml`](./crates/musicxml/): utility functions to emit MusicXML from musical containers.
  - The code will be cleaned up after we extract stable definitions for common-practice musical container structures from `species-counterpoint`. Before that it's considered internal.

- [`species-counterpoint`](./crates/species-counterpoint/) implements a ruleset for scholar species counterpoint and provides a flexible solver based on a kind of beam search.
  - Currently it's a playground with lots of hardcoded features rather than a public-facing API with any sign of stability.

- [`debug-server`](./crates/debug-server/) is an executable target where we put whatever code to test the libraries during development.

- `serialism` provides several utilities to generate and manipulate tone rows and other serialist structures (currently not ported).

## Using the debug interface

We provide [`debug-ui`](./debug-ui/) as a development GUI for testing and visualization. Use `pnpm` to install the dependencies, run `pnpm run dev`, and navigate to the local site. It should say "debug server not running".

Now run `cargo run -p passacaglia-debug-server --release`. The debug GUI should display the progress and the result, together with the search tree: the path to the accepted solution is expanded initially, and untaken branches are shown collapsed (sized by how much they were explored); click a node to expand or collapse it and hover one to view its score.