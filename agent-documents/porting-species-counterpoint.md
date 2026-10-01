# Porting `species-counterpoint` — design report and plan

Status: research complete; port not started.

> **Note:** the finalized porting approach (crate layout, metadata-as-fields,
> "what corresponds to what") is now in `species-counterpoint-approach.md`.
> This document remains the thorough code inspection and analysis; its
> §3/§11/§12 "extract `tonal` now" proposal is superseded by that approach.

`packages/species-counterpoint` is an experimental playground: a best-first
search solver that fills a `Score` (voices of measures of notes) under a set of
musical rules, then renders/plays it. It also holds the *domain* model (Note,
Measure, Voice, Score, Chord, Harmony), entangled with the solver. This report
documents the codebase, separates the clean domain from the solver, lists the
messy/incorrect parts, and proposes the Rust container design and port order.

It supersedes the "Temporary strategy" section of
`porting-musicxml.md`: the `musicxml` adapters will live in `musicxml` and be
added directly to the domain containers (via a `ToMxl` trait), not deferred to
`species-counterpoint`.

---

## 1. Module map

| File | Role |
| --- | --- |
| `Internal.ts` | Thin re-exports of `core` (Pitch/Interval/Scale/Degree/Scales aliases). |
| `Voice.ts` | **Domain**: `Note`, abstract `Measure`, `MeasureData`, abstract `Voice<M>`, `VoiceData`, `parseNotes`, `NonHarmonicType`. |
| `Chord.ts` | **Domain**: `Chord`, `Chords` presets, `ChordElement`, `HarmonyBackground`. |
| `Score.ts` | **Domain**: `Parameters`, `Score`. |
| `Basic.ts` | **Solver glue**: `CounterpointMeasure`, `CounterpointVoice`, `MelodicContext`, `Step`, `BlankMeasure`, `FixedMeasure`, `FixedVoice`, `CounterpointScoreBuilder`, `MelodicSettings`. |
| `Context.ts` | Rule registry + cost settings (`CounterpointContext`), rule type aliases, candidate generation helpers. |
| `rules/*.ts` | The actual rules (candidate filters and local/global cost functions). |
| `Species.ts` / `SpeciesBase.ts` | Species schemas (`defineSpecies`, `MeasureSchema`, `NoteSchema`, `SpeciesMeasure`, `FakeMeasure`). |
| `Imitation.ts` | Imitation voice (`ImitationMeasure`, `EmptyImitationMeasure`, `defineImitation`). |
| `Solver.ts` | A* / best-first search (`Node`, `CounterpointSolver`). |
| `Play.ts` | MIDI playback via `jzz` (out of scope for the port). |

Dependencies of note: `immutable` is declared but **never imported** (dead);
`memoizee` is used only in `rules/FunctionalHarmony.ts`; `@js-sdsl/priority-queue`
in `Solver.ts`; `jzz`/`jzz-synth-tiny` only in `Play.ts`; `install` is bogus.

## 2. Type inventory

**Domain (clean, should be reusable):**
- `Note { duration: Rational, pitch: H.Pitch | null, type?: NonHarmonicType }`
  (`pitch = null` is a rest; `type` is passing/neighbor/suspension).
- `NonHarmonicType = 'passing_tone' | 'suspension' | 'neighbor'`.
- `Measure` (abstract) — a `SequentialContainer<Note>` **and** a `DurationalElement`
  with an independent `duration`; `writablePosition: Rational | null`.
- `Voice<M extends Measure>` — a `SequentialContainer<M>` plus `index`, `name`, `clef`.
- `Score { parameters, voices, harmony }`.
- `Chord { bass, intervals, tones, position, label? }` + `Chords` presets.
- `ChordElement { duration, chord? }` — a `DurationalElement`.
- `HarmonyBackground` — a `SequentialContainer<ChordElement>` plus `scale`.
- `Clef`/`ClefType` (currently in `musicxml`, should move to the domain crate).

**Solver (entangled, should not leak into the domain):**
- `CounterpointContext` — rule lists + cost settings + candidate helpers.
- `MelodicContext` — leap-tracking state threaded through measures.
- `CounterpointMeasure`/`CounterpointVoice` — add `ctx`, `melodicContext`,
  `getNextSteps`, `makeNewMeasure`, `replaceMeasure` to the domain types.
- `Step { measure, advanced, cost, debug }`.
- `BlankMeasure`, `FixedMeasure`, `FixedVoice`, `CounterpointScoreBuilder`.
- `SpeciesMeasure`, `FakeMeasure`, `MeasureSchema`, `NoteSchema`, `defineSpecies`.
- `ImitationMeasure`, `EmptyImitationMeasure`, `defineImitation`.
- `Node`, `CounterpointSolver`, rule type aliases (`LocalRule`, `GlobalRule`,
  `CandidateRule`, `HarmonyRule`).

## 3. Clean domain vs. solver entanglement

The entanglement is class-inheritance based: `CounterpointMeasure extends
Measure` and `CounterpointVoice extends Voice`, injecting `ctx` and
`melodicContext` (mutable search state) into otherwise-pure containers.
`Score`/`Measure`/`Voice`/`Note`/`Chord` are otherwise already immutable in
spirit (`replaceVoice`/`replaceMeasure`/`replaceChord` return new objects).

Recommended Rust shape (this is the disentanglement, done as part of the port):

```
passacaglia-tonal                (clean domain, immutable)
  Note, Measure, Voice, Score, Chord, ChordElement, Harmony, Clef, NonHarmonicType
  └ implements core::structure::{Container, DurationalElement}

passacaglia-species-counterpoint (solver + rules + species)
  CounterpointContext, rules, MelodicContext, species/imitation schemas,
  Solver, Node, Step, …           └ depends on tonal, core, common

passacaglia-musicxml             (adapters)
  impl ToMxl for tonal::Score/Voice/…   └ depends on tonal, core, quick-xml
```

`MelodicContext` stays in the solver crate (or becomes a pure function of a
voice prefix — see §6), and `Clef` moves into `tonal`.

## 4. Container design (the performance-critical part)

### 4.1 Principle: immutability + structural sharing

Every `replace*` returns a new value sharing the unchanged sub-structures.
The solver builds a new `Score` per candidate, so this is the single most
important performance decision. Two viable representations:

- **`Arc<[T]>` (or `Rc<[T]>`)** — simplest. `replace(i, x)` clones the `Arc`
  (O(1)) and rebuilds one small array (O(n) of *that container*). Because a
  measure has a handful of notes and a voice a handful of measures, n is tiny.
- **`im::Vector<T>`** — persistent vector, O(log n) update with full structural
  sharing. Marginally more complex; use if profiling shows the `Arc<[T]>`
  rebuilds matter. The `im` crate is `Arc`-backed and `Send + Sync`, which also
  future-proofs parallel beam search.

Recommendation: start with `Arc<[T]>` (or a small wrapper `Shared<T> =
Arc<T>`), keep element types `Clone` (cheap `Arc` clones), and only move to
`im::Vector` if benchmarks justify it.

### 4.2 Time model and the `Container` trait

`core::structure::SequentialContainer<T>` precomputes `starts: Vec<Rational>`
so `start(i)` is O(1). That is fine for static data but forces an O(n) rebuild
of the starts table on every `replace`, which fights structural sharing.

Two observations about the existing `core` `Cursor`:

1. `Cursor::next()/prev()` currently recompute `local_time` by calling
   `self.container.start(new_index)` instead of doing it incrementally. The TS
   `SequentialCursor` maintains time incrementally (`time + duration`), O(1).
2. For a *sequential* container (`start(i+1) == start(i) + span(i)`),
   `next()/prev()` can compute time incrementally from `span()` and never touch
   `start()`. `start(i)` is then only needed for random access
   (`cursor(i)`, `cursor_at_time`), which the solver uses rarely.

Recommended: add a `Sequential: Container` capability (a marker method or
sub-trait) and specialize `Cursor::next/prev` to advance time incrementally.
Combined with `Arc<[T]>`/`im::Vector`, `replace` no longer needs to recompute a
starts table at all — `start(i)` becomes a lazy prefix-sum (O(i), used only on
random access). This is the change that best "exploits immutability".

If we keep the eager `starts` table, then store it as `Arc<[Rational]>` in
parallel with `Arc<[T]>` and accept the O(n) rebuild of a small container.

### 4.3 Concrete domain types

`Measure` is *not* a pure sequential container (its `duration` ≠ sum of its
notes). Model it as composition (already in `porting-musicxml.md` §3):

```rust
struct Measure {
    notes: Arc<[Note]>,       // impl Container { Item = Note } (delegation)
    duration: Rational,       // impl DurationalElement
    writable_position: Option<Rational>,
}
struct Voice {
    measures: Arc<[Measure]>, // impl Container { Item = Measure }
    index: usize, name: String, clef: Clef, lower_range: Pitch, higher_range: Pitch,
}
struct Score {
    parameters: Parameters,
    voices: Arc<[Voice]>,      // heterogeneous voices → enum (see §6)
    harmony: Harmony,          // HarmonyBackground
}
```

`Chord`/`ChordElement`/`Harmony` similarly use `Arc<[..]>` and `Clone`-by-Arc.

### 4.4 Heterogeneous voices

`Score.voices` in TS holds a mix of `FixedVoice` and `CounterpointVoice`
(via array covariance + `Voice<Measure>`). Rust has no covariance and
`Container` is not object-safe (associated `Item` + `&Self::Item`), so
`Vec<Box<dyn VoiceLike>>` is unavailable. Resolution (as in
`porting-musicxml.md`): unify into a concrete `Voice` struct with a `kind`
enum, or an `enum Voice { Fixed(..), Counterpoint(..) }` that `impl Container`
by dispatch. This is a solver-crate concern, not a `core` concern.

## 5. Candidate representation

Rules operate on `HashMap<Pitch, Cost>` (TS: string-keyed). In Rust this is
`HashMap<Pitch, i64>` (`Pitch` already derives `Hash`/`Eq`) — no string
hashing. Candidate sets are small (scale tones in a voice range, tens of
elements), and the operations are `filter`, `filter_map`, `intersect`,
`union`, `difference`, `get`/`set`/`has`.

Recommendation: a `Candidates` newtype over `Vec<(Pitch, Cost)>` (sorted or
just linear-scanned) with in-place `filter`/`intersect` methods to avoid
allocation churn; fall back to `std::collections::HashMap` if profiling says
so. Rules should be pure (take/return `Candidates`) — the TS rules mutate the
map in place and return it, but a small `Vec` returned by value is cheap and
cache-friendly.

## 6. State management problems (to fix during port)

1. **`MelodicContext` stored per measure.** Each `SpeciesMeasure` snapshots the
   leap counters (`nConsecutiveLeaps`, `n3rdLeaps`, …) from the previous note.
   This is redundant, derived state threaded through the domain and easy to get
   inconsistent. Fix: keep `Measure` clean; compute `MelodicContext` on demand
   from the voice prefix (pure fold over notes) or thread it as solver-side
   state, not a field on the measure.
2. **`FakeMeasure.counter`** (global static mutable counter) and **`Node.id`**
   (static counter) — non-deterministic global state used to synthesize
   "unique" hashes. Remove; see §7 on hashing.
3. **`Node.hash()` returns the id, not the score.** `#parents` (the visited
   set) is keyed by `hash()`, so with a unique id per node the set never
   dedups — the search is effectively an *unbounded tree/beam search*, with only
   `removeOld` pruning keeping memory in check. This is the single most
   impactful solver issue (perf *and* correctness). Fix: a real structural
   `Hash`/`Eq` on `Score` (derive, or incremental hash stored alongside the
   `Arc`), then a `HashSet<Score>` visited set. If full-score hashing is too
   costly, use content-addressing/interning so `Eq`/`Hash` is O(1).
4. **`shuffle` imported but unused** in `Solver.ts` (dead).
5. **`HashMap` methods mutate in place** (`filter`, `intersectWith`, …), which
   is why candidates "chain". In Rust make these pure or explicitly
   `&mut self` to avoid aliasing surprises.

## 7. Suspect / incorrect rule implementations

These should be ported as-is first (with tests), then fixed; do not silently
"improve" them during porting.

1. `Solver.ts:147` — `hash()` returns `id` (dedup disabled), see §6.3.
2. `Imitation.ts:115` — `return \`…:${this.hashNotes}\`;` interpolates the
   *function* (missing `()`), so every imitation measure of the same
   targetVoice/measure hashes identically. Bug (latent while dedup is off).
3. `VoiceOverlapping.ts:22,41,50` — `if (!nord) continue;` skips pitch ordinal
   `0` (and any falsy value); boundary bug.
4. `VerticalConsonance.ts` — only pairwise-checks *moving* voices against each
   other; a moving voice is never checked for consonance against a *held* note
   in another voice. Strict species counterpoint requires every simultaneous
   dyad to be consonant. Suspect simplification.
5. `Utils.ts:44 isConsonance` — treats P4 (`simple == 5`) as consonant unless
   `withBass`, while `Context.forbidWithBass` also tries to forbid P4-with-bass
   (and appears unused anywhere). Two overlapping, fragile mechanisms.
6. `Species.ts` (species 5) — many commented-out schemas; `sp5.5.2` has
   `// FIXME: require suspension`; several `measureLength.value() / 2` repeat
   counts are non-integer (floored silently by `repeat`), producing
   off-by-one note counts for odd measure lengths. Species 5 is the least
   finished.
7. `Scales.ts:79 enforceMinor` — hard-coded degree indices `6/7/8` against the
   9-degree `completeMinor` scale (magic numbers); the ascending/descending
   melodic-minor handling is hand-rolled and covers only a few cases. Needs
   verification against the intended rules.
8. `NearbyPerfects.ts` — `(NOT IMPLEMENTED)` note for the contrary-motion case
   (line 14); incomplete.
9. `Suspension.ts:42` — `prev.duration < cur.duration` preparation constraint
   is plausibly inverted/too strict at boundaries.
10. `Melody.ts:64 avoidRepeat2` — commented-out duration-equality checks show
    known incompleteness.
11. `Chord.withLabel` mutates `this.label` then returns `this` — the one place
    the otherwise-immutable `Chord` mutates; a footgun under structural
    sharing. Make it pure in the port.

## 8. The solver (what it actually is)

`CounterpointSolver.aStar` is a **best-first / beam search**, not a true A*:

- Priority queue ordered by `cost - f * nStep` (constant reward `f`); pops
  lowest each iteration, processes in batches of `batch = 5`.
- A `Node` advances its `measureIndex` internally (in the constructor) until it
  finds the next *writable* position (`#findWritable`): either an unresolved
  chord, or the earliest `writablePosition` across voices.
- `getNeighbors()` produces candidates for that single writable note/chord, each
  building a new `Score`; harmony moves don't advance `nStep`, note moves
  advance `nStep` by the note's duration.
- `cost = cost * POWER + thisCost` with `POWER = 0.9` (discounting); the reward
  term `f * nStep` biases toward longer completions.
- No visited-set dedup (see §6.3); only `measureIndex < furthest - removeOld`
  prunes stale frontier nodes.

Porting notes: the algorithm maps directly to Rust with a `BinaryHeap` (or the
`priority-queue` crate) + a custom `Ord` on a `State` struct; `Node` becomes a
plain struct holding `Arc<Score>`. Fix the dedup by implementing `Hash`/`Eq`
for `Score`. The `CounterpointSolverProgress`/`onProgress` callback becomes a
`&mut dyn FnMut` or a channel.

## 9. Rule architecture in Rust

The four rule shapes are pure functions:

```rust
type LocalRule     = Box<dyn Fn(&Context, &Score, NoteCursor) -> Cost>;
type GlobalRule    = Box<dyn Fn(&Context, &Score) -> Option<String>>;
type CandidateRule = Box<dyn Fn(&Context, &Score, NoteCursor, Candidates, Option<NonHarmonicType>) -> Candidates>;
type HarmonyRule   = Box<dyn Fn(&Context, &Score, ChordCursor, Candidates) -> Candidates>;
```

`CounterpointContext` holds `Vec<LocalRule>` etc. The `enforceX(...)` /
`prioritizeX(...)` closures that capture config (`DegreeMatrix`, root
intervals, chord lists) become closures or small structs implementing a `Rule`
trait. The cursor types are `core::structure::Cursor` with the parent chain
typed (e.g. `Cursor<'a, Measure, Cursor<'a, Voice, ()>>`); the TS
`@ts-expect-error` `withParent` hacks disappear because Rust's `Cursor`
carries the typed parent natively.

## 10. What `core` already provides (mapping)

Already ported and sufficient: `Pitch`, `Interval` (incl. `to_abbreviation`
→ `"M2"`/`"-M2"`, needed by `enforceMinor`), `Scale` (`root`, `degrees`,
`at`, `get_exact_degree`, `get_degrees_in_range`, `transpose_to`, `rotate`),
`Degree` (`index`, `next`, `to_pitch`), `PitchSystem`/`StandardHeptatonic`,
`structure::{Container, Cursor, DurationalElement, SequentialContainer}`.
`common` has `Rational`, `rational`, `rational_value`, `rotate_array`.

Gaps to add (mostly in the solver crate, not `core`): `Chord` (build from
pitches/intervals, `contains`, `withRoot`/`withBass`/`toPosition`, `label`),
a `Hash`/`Eq` for `Score`/`Measure`/`Voice` (derive), and a `Debug`/assertion
helper replacing the TS `Debug.assert/never/trace`.

## 11. `musicxml` adapters (revised strategy)

`musicxml` will depend on `tonal` (not the solver crate) and add serialization
directly to the containers via a trait:

```rust
// in passacaglia-musicxml
pub trait ToMxl { fn to_mxl(&self) -> String; }
impl ToMxl for tonal::Score { … }     // score()/part()/measure()
impl ToMxl for tonal::Pitch?          // or: fn pitch(&Pitch) already there
```

The traversal in §4 of `porting-musicxml.md` (`iter_cursors`, `first_child`,
`next_global`) applies directly to `tonal` types implementing `Container`. This
is why `tonal` must exist before the full `musicxml` adapters are written, and
why `Clef`/`NonHarmonicType` move into `tonal`.

## 12. Porting order (milestones)

1. **`tonal` crate** — `Note`, `NonHarmonicType`, `Measure`, `Voice`, `Score`,
   `Chord`, `ChordElement`, `Harmony`, `Clef`, `Parameters`; `Container`/
   `DurationalElement` impls with `Arc<[T]>` structural sharing; `Hash`/`Eq`;
   unit tests (port `Chord.test.ts`, the builder helpers).
2. **`core` tweak (optional but recommended)** — `Sequential` capability so
   `Cursor::next/prev` advance time incrementally (perf), enabling lazy
   `start`; keep `Container` API unchanged.
3. **`species-counterpoint` crate** — `CounterpointContext`, `Candidates`,
   `MelodicContext` (as derived state), rules (ported 1:1 with tests from
   `tests/rules/*.test.ts`), `defineSpecies`/schemas, `defineImitation`,
   `Solver`/`Node` (with real dedup), `Step`, `CounterpointScoreBuilder`.
4. **`musicxml` crate** — full adapters `impl ToMxl for tonal::*` (port the
   `note/measure/part/score` functions), replacing the stub `pitch()` only
   state; tests against `tonal` fixtures.
5. **Follow-ups** — fix the §7 rules after porting; `Play` (MIDI) out of scope;
   extract `MelodicContext`/species into their own module or crate if desired.

## 13. Open questions

- Single-threaded `Rc` vs `Arc` for shared nodes (parallel beam later?).
- `Candidates`: `Vec`-based vs `HashMap`-based — benchmark.
- Score `Hash` for dedup: full structural vs content-addressed/interning.
- Whether `MelodicContext` should be derived (pure) or threaded as solver state.
- Confirm species-5 schema semantics before porting its `repeat` counts.
