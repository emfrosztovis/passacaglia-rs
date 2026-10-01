# Porting `species-counterpoint` — current approach

Status: finalized plan (pre-port). This is the authoritative "how we're doing
it" document. It refines the crate-layout and metadata sections of
`porting-species-counterpoint.md` (the thorough inspection) and supersedes that
document's "extract `tonal` now" recommendation.

## 1. Decisions (what changed)

1. **No standalone domain crate yet.** `Score`/`Measure`/`Note`/`Voice` are
   kept local to `species-counterpoint`, because the general model (a `Score`
   that can mix tonal and atonal voices, experimental combinations, multiple
   pitch systems) is a hard design problem we are explicitly *postponing*.
2. **Metadata is dumped as plain fields** on the local types, mirroring the TS
   source, rather than a generic annotation slot.
3. **`musicxml` imports `species-counterpoint`** for its adapters (for now),
   rather than depending on a separate `tonal` crate.
4. Everything else from the inspection (container immutability via `Arc`,
   the solver/dedup fix, the rule bugs, the candidate representation) is
   unchanged.

## 2. Crate layout

```
crates/
  common             (exists)   Rational + utilities
  core               (exists)   PitchSystem/Pitch/Interval/Scale/Tuning + structure
  macros             (exists)
  musicxml           (exists)   quick-xml; adapters (ToMxl) over species-counterpoint types
  species-counterpoint (NEW)    domain + solver + rules + Clef
```

Dependency direction (acyclic):

```
musicxml ──► species-counterpoint ──► core ──► common
```

**`Clef` moves.** `Clef`/`ClefType` currently live in `crates/musicxml`. Because
`musicxml` will now import `Score`/`Note`/`Clef` from `species-counterpoint`
(and Rust forbids circular crate dependencies), `Clef` must move into
`species-counterpoint`. Until that crate exists, `Clef` stays where it is;
`musicxml` drops its own re-export at that point.

## 3. What corresponds to what

### 3.1 Packages / crates

| TS package | Rust crate |
| --- | --- |
| `common` | `passacaglia-common` (exists) |
| `core` | `passacaglia-core` (exists) |
| `musicxml` | `passacaglia-musicxml` (exists) |
| `species-counterpoint` | `passacaglia-species-counterpoint` (**new**) |

### 3.2 `species-counterpoint` source files → Rust modules

| TS file | Rust module (under `crates/species-counterpoint/src/`) |
| --- | --- |
| `index.ts` | `lib.rs` (re-exports) |
| `Internal.ts` | — none; use `passacaglia_core::std_hept` directly (optional `prelude.rs`) |
| `Voice.ts` | `voice.rs` — `Note`, `NonHarmonicType`, `Measure`, `Voice`, `parse_notes` |
| `Chord.ts` | `chord.rs` — `Chord`, `Chords`, `ChordElement`, `Harmony` |
| `Score.ts` | `score.rs` — `Parameters`, `Score` |
| `Basic.ts` | `basic.rs` — `MelodicContext`, `Step`, `CounterpointMeasure`, `CounterpointVoice`, `BlankMeasure`, `FixedMeasure`, `FixedVoice`, `CounterpointScoreBuilder`, `MelodicSettings` |
| `Context.ts` | `context.rs` — `CounterpointContext`, rule type aliases, candidate helpers |
| `rules/index.ts` | `rules/mod.rs` |
| `rules/*.ts` (15 files) | `rules/*.rs` (1:1) |
| `Species.ts` + `SpeciesBase.ts` | `species.rs` — schemas, `SpeciesMeasure`, `FakeMeasure`, `define_species` |
| `Imitation.ts` | `imitation.rs` — `ImitationMeasure`, `EmptyImitationMeasure`, `define_imitation` |
| `Solver.ts` | `solver.rs` — `Node`, `CounterpointSolver`, progress |
| `Play.ts` | **deferred** (MIDI playback via `jzz`, out of scope) |
| `musicxml/Types.ts` (`Clef`) | `clef.rs` (moved into `species-counterpoint`) |

### 3.3 `common` dependencies → Rust

| TS (`common`) | Rust |
| --- | --- |
| `Rational`, `AsRational` | `passacaglia_common::Rational` (exists); `impl Into<Rational>` |
| `HashMap<K,V>` (string-keyed) | `std::collections::HashMap` or a `Candidates` newtype |
| `Hashable` | `std::hash::Hash + Eq` |
| `Serializable` / `Serialized` | **dropped for now** (persistence deferred) |
| `Debug.assert/never/trace/info` | `assert!`/`debug_assert!`/`unreachable!`/`eprintln!` |
| `repeat(n, f)` | `(0..n).map(f).collect()` |
| `rotateArray` | `passacaglia_common::rotate_array` (exists) |
| `modulo` | `rem_euclid` |
| `shuffle`/`randomChoice`/`randomWeighted` | **drop** (unused; stochastic deferred) |
| `memoizee` (FunctionalHarmony) | `OnceLock`/manual cache or recompute |

### 3.4 `core` usage → Rust (already available)

| TS (`core`) | Rust (`passacaglia_core`) |
| --- | --- |
| `StandardHeptatonic as H` | `std_hept` |
| `H.Pitch/Interval/Scale/Degree/PitchClasses/Scales` | `std_hept::{Pitch, Interval, Scale, Degree, PitchClasses, scales}` |
| `DurationalElement`, `SequentialContainer`, `SequentialCursor`, `Cursor` | `structure::{DurationalElement, Container, Cursor, SequentialContainer}` |
| `Pitch.parse`, `Interval.parse`, `Interval.toString` | `Pitch::parse`, `Interval::parse`, `Interval::to_abbreviation` (note: `to_abbreviation(true)` ≈ TS `toString()` for `"M2"`/`"-M2"` comparisons in `enforceMinor`) |

### 3.5 Key types

| TS type | Rust (local, pragmatic) |
| --- | --- |
| `Note` | `struct Note { duration: Rational, pitch: Option<Pitch>, non_harmonic: Option<NonHarmonicType>, debug: String }` |
| `NonHarmonicType` | `enum NonHarmonicType { PassingTone, Suspension, Neighbor }` |
| `Measure` (abstract) | `struct Measure` (concrete; `Container<Item=Note>` + `DurationalElement`) |
| `CounterpointMeasure` | `struct CounterpointMeasure` (holds `ctx`, `melodic_context`, `writable_position`, `schema`…), or fold into `Measure` — see §4 |
| `Voice<M>` / `CounterpointVoice` | `struct Voice` / `struct CounterpointVoice` |
| `Score` | `struct Score { parameters: Parameters, voices: Arc<[Voice]>, harmony: Harmony }` |
| `Parameters` | `struct Parameters { measure_length: Rational }` |
| `Chord` | `struct Chord { bass: Pitch, intervals: Arc<[Interval]>, tones: Arc<[Pitch]>, position: usize, label: Option<String> }` |
| `Chords` | `mod chords` (consts) |
| `ChordElement` | `struct ChordElement { duration: Rational, chord: Option<Chord> }` |
| `HarmonyBackground` | `struct Harmony { scale: Scale, elements: Arc<[ChordElement]> }` |
| `Clef`/`ClefType` | `struct Clef { sign: ClefType, line: i32, octave: Option<i32> }` (moved) |
| `MelodicContext` | `struct MelodicContext { … }` (dumped as a field for now) |
| `Step` | `struct Step { measure, advanced: Rational, cost: i64, debug: String }` |
| `MeasureSchema`/`NoteSchema` | `struct MeasureSchema` / `enum NoteSchema` |
| `CounterpointContext` | `struct CounterpointContext { local_rules, global_rules, …, costs }` |
| `LocalRule`/`GlobalRule`/`CandidateRule`/`HarmonyRule` | `type` aliases over `Box<dyn Fn(…) -> …>` (or a `Rule` trait) |
| `Node` | `struct Node` (solver) |
| `CounterpointSolver` | `struct CounterpointSolver` |

## 4. How to handle the entanglement (pragmatic)

The cleanest end-state splits the pure domain from the solver, but for now we
mirror the TS: keep `CounterpointMeasure`/`CounterpointVoice` (with `ctx`,
`melodic_context`, `writable_position`, `get_next_steps`, `make_new_measure`)
as solver-aware subtypes. Two acceptable ways to express that in Rust without
the full domain/solver split:

- **A. Enums.** `enum Measure { Blank(..), Fixed(..), Species(..), Imitation(..) }`
  (or a `Kind` field). One concrete `Measure`; solver-specific data lives in the
  variant. Simplest for the heterogeneous-voice problem too.
- **B. Mirror TS.** A `Measure` struct + `CounterpointMeasure` carrying the
  extra fields, with `Voice`/`CounterpointVoice` likewise. Closest to the source,
  most mechanical port.

Recommend **A** where possible (it also resolves the heterogeneous
`Score.voices` issue), but **B** is fine for a first mechanical pass. Either
way, the metadata (`melodic_context`, `writable_position`) stays local to
`species-counterpoint` and is never pushed into `core`.

## 5. Port order

1. Create `crates/species-counterpoint`; move `Clef`/`ClefType` here; update
   `musicxml` to import them.
2. Port domain types + metadata fields (mirror §3.5), implementing
   `Container`/`DurationalElement` with `Arc<[T]>` structural sharing and
   `Hash`/`Eq`.
3. Port `CounterpointContext`, `Candidates`, rule type aliases, then the 15
   rules (with tests ported from `tests/rules/*.test.ts`).
4. Port `Species`/`Imitation` schemas + `Solver`/`Node` (fixing the dedup).
5. Port `musicxml` adapters (`note`/`measure`/`part`/`score`) as
   `impl ToMxl for species_counterpoint::*`, replacing the stub `pitch()`-only
   crate.
6. Fix the known rule bugs (see `porting-species-counterpoint.md` §7) and add
   tests as we go.

## 6. Postponed design problems (do not solve now)

1. **General / heterogeneous `Score`.** Mixing tonal and atonal voices, multiple
   pitch systems in one score, arbitrary experimental combinations. This is the
   big one — the reason `Score`/`Measure`/`Note`/`Voice` stay local and
   `StandardHeptatonic`-specific for now.
2. **Metadata model.** A general annotation mechanism (generic slot with `()`
   default vs. external ECS side-tables keyed by stable element ids). For now:
   plain fields.
3. **Extraction to a standalone domain crate** (`tonal` or a more general
   `music`), once the shape of the domain is stable.
4. **Parameterizing `Note`/`Measure`/`Voice`/`Score` over `S: PitchSystem`.**
5. **`Clef` / notation placement** (domain vs. a notation/staff crate).
6. **Persistence / serialization** (Serde), and web-worker-like parallelism.
7. **Score `Hash` strategy for solver dedup** — full structural hash vs.
   content-addressing/interning (decide during §5 step 4, not deferred forever).

## 7. Kept from the inspection report

Unchanged and still binding: the container design (immutability + `Arc`/`im`
structural sharing, the `Cursor` incremental-time optimization), the solver
dedup fix, the candidate `Vec`-vs-`HashMap` question, and the list of
incorrect/suspect rules to port-first-then-fix.
