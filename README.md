# `passacaglia`, Op.1 (Stage 2 Rust port)

> Dedicated to Nikolai Y. Myaskovsky

Music theory and procedural music generation libraries in early-stage development.

- [`common`](./crates/common/) contains shared utility classes, types and functions that are not related to music. Most importantly, it provides `Rational` and `HashMap` which are used everywhere.

- [`core`](./crates/core/): structs for basic musical elements `Pitch`, `Interval`, `Scale`, `Degree` etc. These come with a type parameter `S: PitchSystem`, which means they're agnostic on the musical system used, as long as the system consists of a fixed frequency period and a number of pitch classes. The standatd heptatonic pitch system used in common-practice Western music is provided as a reference `PitchSystem`; in the `std_hept` module heptatonic-specific features like parsing and pretty-printing are added to the types.
  - The crate also contains traits for generic musical structures (or “containers”, like measures, voices etc) which can be navigated with cursors.

- [`macros`](./crates/macros/): macros for generating compile-time musical constants.

- [`musicxml`](./crates/musicxml/): utility functions to emit MusicXML from musical containers.

- [`species-counterpoint`](./crates/species-counterpoint/) implements a ruleset for scholar species counterpoint and provides a flexible solver based on a kind of beam search.

- `serialism` provides several utilities to generate and manipulate tone rows and other serialist structures (currently not ported).

## Using the debug interface

We provide [`debug-ui`](./debug-ui/) as a development GUI for testing and visualization. Use `pnpm` to install the dependencies, run `pnpm run dev`, and navigate to the local site. It should say "debug server not running".

Now run `cargo run -p passacaglia-debug-server --release`. The debug GUI should display the progress and the result. Note that currently the search tree visualization has not been ported yet.