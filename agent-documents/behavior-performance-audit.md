# Rust port vs. TypeScript reference — behavior & performance audit

Status: source-level analysis (no profiling yet). Compares
`/Users/emf/Experiments/passacaglia-rs` against
`/Users/emf/Experiments/passacaglia-ts`.

Scope: the solver + rule pipeline that the `debug-server` actually runs
(`soprano/alto/tenor = species5`, `bass = species1`, `targetMeasures = 8`,
`measureLength = 4`, `removeOld = 5`, `batch = 50`, `reward = 25`), plus the
`core`/`common` primitives it sits on. The rule sets registered in
`crates/debug-server/src/main.rs` and `apps/debug-ui/src/Main.worker.ts` are
identical (same rules, same order, `allowUnison = true`).

---

## 1. What is faithful (verified line-by-line)

These were compared 1:1 and match the TS semantics:

- **Species schemas** (`species1`–`species5`), including the `repeat(n)`/`ceil`
  semantics and the `sp5.5.4` `floor(measureLength * 3/4)` math.
  `crates/species-counterpoint/src/species.rs` vs `Species.ts`/`SpeciesBase.ts`.
- **`MelodicContext` update** (`update_melodic_context` vs `updateMelodicContext`).
- **All 15 rules** (melody, leaps, scales, vertical consonance, voice
  overlapping, voice motion, similar motion, suspension, neighbor/passing
  tones, root progression, functional harmony, valid chords) — including the
  deliberate `ord() == 0` skip in `voice_overlapping` (ported as the TS `if
  (!nord) continue` truthiness bug, preserved rather than "fixed").
- **`Scale`/`Degree`/`Pitch`/`Interval`** core: `ord`, `distance_to`, `steps_to`,
  `interval_to`, `add`, `to_simple`, `add_period`, `get_degrees_in_range`,
  `get_exact_degree(allow_enharmonic=false)`, system constants
  (`N_DEGREES=7`, `N_PITCH_CLASSES=12`, `DEGREE_OFFSETS=[0,2,4,5,7,9,11]`).
- **`Chord`** construction/equality/hash (label excluded), `Chords` presets.
- **`Node::new` / `find_writable`** target selection and goal detection.
- **Solver reward** `cost - f * nStep`; `@js-sdsl` is a min-heap by its `cmp`,
  and the Rust `BinaryHeap` is inverted to also pop the minimum priority —
  confirmed by reading `@js-sdsl`'s sift-up/sift-down in `dist/esm/index.js`.
- **Rule ordering** (`candidate_rules_before` → per-call rules →
  `candidate_rules_after`) and the early-return-on-empty-candidates behavior.
- **`FakeMeasure` harmonic-first expansion** (local rules evaluated against the
  single `p0`-filled note), `ImitationMeasure`, `EmptyImitationMeasure`.

In other words: the port is a faithful transcription. The differences below are
in *data-structure choice* and *cursor cost model*, not in the musical rules.

---

## 2. Behavioral differences

### 2.1 Non-determinism: candidate sets are a `HashMap` (highest impact)

The TS candidate map is `common::HashMap`, backed by a JS `Map` keyed by the
`hash()` **string** (`common/src/Common.ts`). JS `Map` iterates in **insertion
order**, so the whole search is deterministic: `enforceScaleTones` seeds the
candidate set in ascending scale order, and every `filter`/`intersectWith`
mutates in place, preserving that order.

The Rust port uses `std::collections::HashMap<Pitch, f64>`
(`crates/species-counterpoint/src/context.rs:14`, `Candidates`). That map uses
`RandomState` (SipHash with a per-process random seed), so:

1. **Iteration order is randomized per run.** `fill_in` iterates candidates to
   emit `Step`s (`context.rs:256`), which becomes the neighbor order, which
   feeds the priority queue, which decides which equal-cost solution is reached
   first. The solver's output therefore **varies between runs of the same
   binary**.
2. Even within a run, `HashMap` iteration order is *not* ascending pitch, so the
   "first good enough" solution the TS version finds deterministically will
   differ from what Rust finds.

This is the primary source of "the Rust version produces a different score than
the TS version."

Note: `porting-species-counterpoint.md` §5 explicitly recommended a
`Vec<(Pitch, Cost)>`-based `Candidates` for exactly this reason (candidate sets
are tens of elements; linear scan beats a hash map, and it stays ordered). That
recommendation was not followed.

### 2.2 Priority-queue tie-breaking differs

Both pop the minimum `cost - f·nStep` (confirmed in §1), but ties are broken
differently:

- TS `@js-sdsl` min-heap: `cmp` returns `0` for ties, so tie order is whatever
  the heap's internal array order happens to be (insertion-dependent, but
  deterministic given deterministic insertion order).
- Rust `HeapEntry::cmp` (`solver.rs:198`) breaks ties by `seq`, and because the
  comparison is inverted, **later-inserted** nodes are popped first.

Combined with 2.1's random iteration order, the exploration order (and thus the
first goal found) is genuinely different. Both are valid best-first searches;
neither is a correctness bug, but they won't reproduce each other's output.

### 2.3 Exact rational arithmetic vs. float rational arithmetic

TS `Rational` stores `num`/`den` as IEEE-754 `number`s (`common/Rational.ts`),
with `Rational.from(x, eps=0.00001)` doing a decimal-expansion loop and all ops
reduced by gcd on floats. The Rust port uses `num_rational::Ratio<i64>`
(`crates/common/src/rational.rs:4`), which is exact.

For the durations/intervals actually in play (halves, quarters, semitone
integers) both are exact, so this **does not** show up in the default solve.
But it is a genuine divergence for any microtonal/fractional input: TS can
silently drift or mis-reduce; Rust stays exact. The Rust behavior is the *more
correct* one — it just won't bit-for-bit reproduce TS's float path.

### 2.4 (Minor) `furthest`/progress reporting is coupled in Rust

`solver.rs:304` folds the `furthest` update into the progress-reporting branch:

```rust
if current.measure_index > furthest || n_node.is_multiple_of(self.report_interval) {
    if current.measure_index > furthest { furthest = current.measure_index; }
    ...
}
```

TS updates `furthest` unconditionally (`Solver.ts:215`) and reports progress in
a separate `if` (`Solver.ts:218`). `furthest` ends up equal either way, so the
pruning decision (`measureIndex < furthest - removeOld`) is unaffected; only
the timing of `onProgress` callbacks differs (cosmetic).

---

## 3. Performance analysis (from the code)

The port is *algorithmically* the same beam search. The slowdown comes from
constant-factor and per-call overhead in the primitives, concentrated in the
two places the hot path exercises constantly: **cursor time math** and
**candidate-set operations**.

### 3.1 `Cursor::next()`/`prev()` recompute `start()` (O(n) instead of O(1))

`crates/core/src/structure/container.rs:138-168`:

```rust
pub fn prev(&self) -> Option<Self> {
    ...
    let new_local = self.container.start(new_index);   // re-folds the prefix
    ...
}
pub fn next(&self) -> Option<Self> {
    ...
    let new_local = self.container.start(new_index);
    ...
}
```

For the domain containers, `start(i)` is a fold over the prefix:

- `Measure::start` → `note_start` (`crates/species-counterpoint/src/voice.rs:193,211`)
- `Voice::start` → fold (`voice.rs:404`)
- `Harmony::start` → fold (`chord.rs:271`)

Each `next()`/`prev()` (and therefore every `next_global()`/`prev_global()`,
which the rules call constantly) does an O(i) fold of `Ratio<i64>` additions,
each of which does an integer gcd reduction.

TS's `SequentialCursor` does this **incrementally**:
`next() = this.time.add(this.value.duration)` (`Containers.ts:126`), O(1). This
exact regression was flagged in `porting-species-counterpoint.md` §4.2 ("add a
`Sequential` capability so `Cursor::next/prev` advance time incrementally") and
in `NOTEPAD.md`, but the capability was never added. The generic `Stepper`
abstraction (`container.rs:213`) is in place — nothing specializes it to the
sequential case.

### 3.2 `cursor_at_time` / `note_at` are O(n²)

`container.rs:47` loops `i in 0..len` and calls `start(i)` (itself O(i)) inside,
so `cursor_at_time` is O(n²) per call; `Voice::note_at` (`voice.rs:373`) then
also does `index_at_time` (O(n)) and `child` (calls `start`, O(i)).

`note_at` is in the innermost loop of `vertical_consonance`, `voice_motion`,
`similar_motion`, `voice_overlapping`, `nearby_perfects`, and the fake-measure
expansion — i.e. several times per candidate pitch per voice. TS's
`cursorAtTime` iterates `entries()` with incremental O(1) `next()`, i.e. O(n).

`n` is small (8 measures, ≤4 notes), so this is a constant-factor hit (~a few
gcd-fold operations per navigation) rather than asymptotic, but it is applied a
huge number of times.

### 3.3 `Candidates` as `HashMap` — slow hashing + full rebuilds

- `HashMap` uses `RandomState`/SipHash; every `get`/`set`/`filter`/`intersect`
  hashes a `Pitch` (which hashes an `Ratio<i64>`: two `i64`s through SipHash).
  TS hashes a short string into a JS `Map`. On candidate sets of ~10-20 pitches,
  SipHash dominates.
- `filter_map` (`context.rs:69`) and `intersect_with` (`context.rs:83`) do
  `self.0.drain()` into a **brand-new `HashMap`** on every call, re-inserting and
  re-hashing every element. `enforce_suspension`, `make_suspension`,
  `make_passing_tone`, `make_neighbor_tone`, and `enforce_melody_intervals`
  (via `intersect_with`) all hit this path per candidate. TS's
  `filterMap`/`intersectWith` mutate the JS `Map` in place (`Common.ts:56,69`).

A `Vec<(Pitch, f64)>` with in-place `retain` would remove both the SipHash cost
and the reallocations, and would also restore determinism (§2.1).

### 3.4 Dynamic dispatch through `Rc<dyn Fn>` rule closures

Rules are `Rc<dyn for<'a> Fn(…) -> …>` (`context.rs:95-121`). Every rule
application is an indirect call through a fat pointer. In TS these are plain
top-level functions that V8 inlines/monomorphizes. This is a per-rule-call
overhead, and rules run once per candidate pitch per note.

### 3.5 `replace_*` full array clones (parity with TS, but still on the hot path)

`Voice::replace_measure` (`voice.rs:298`), `Score::replace_voice`
(`score.rs:41`), and `Harmony::replace_chord` (`chord.rs:250`) each do
`to_vec()` + `Rc::from(...)` — an O(n) clone + fresh allocation of the sibling
array, plus a `Measure` write clones the note array (`species.rs:113`). TS does
the same shape (`[...this.elements]; splice`), so this is not a *regression*,
but `porting-species-counterpoint.md` §4.1 recommended `im::Vector` (O(log n)
structural sharing) precisely because this is the most-executed operation; that
was not adopted (the `Rc<[T]>` choice is the fallback plan).

### 3.6 `parents` map keyed by `Rc<Score>` pointer identity (no dedup, unbounded)

`solver.rs:221,263,323` key `parents` by `Rc<Score>`. `Rc` hashes by *pointer*,
and each neighbor is a fresh `Rc::new`, so `contains_key` never matches — dedup
is disabled, exactly as TS does via `Node.hash() → id` (`Solver.ts:147`). This
is parity (both leak memory as the frontier grows and do an O(1) hash per
neighbor on a never-matching key), but it is the single most impactful solver
inefficiency flagged in the porting report §6.3, and Rust is where the fix is
trivial: key the map by `Score` (structural `Hash`/`Eq` already derive) or
intern scores.

### 3.7 (Resolved) `Arc` → `Rc`

Commit `26a09de` already replaced `Arc` with `Rc` for the single-threaded
solver; not a current cost.

---

## 4. Priority-ordered recommendations

1. **Make `Candidates` an ordered `Vec<(Pitch, f64)>`** (or `BTreeMap`). Fixes
   non-determinism (§2.1), removes SipHash cost and full-map rebuilds (§3.3).
   Candidate sets are tiny; linear scan is fine.
2. **Add a `Sequential` specialization so `Cursor::next/prev` advance time by
   `+span()`/`-span()`** instead of re-folding `start()` (§3.1), and make
   `cursor_at_time` walk with incremental time (§3.2). This is the intended
   `Stepper` design already scaffolded in `container.rs`.
3. **Key `parents` by `Score`** (not `Rc<Score>`) to enable real state
   deduplication (§3.6) — the largest correctness/performance win available.
4. **Consider `im::Vector` (or an ordered persistent vector) for `Voice`/`Score`
   `replace`** (§3.5) if profiling shows the `to_vec()` clones dominate.
5. If byte-for-byte parity with the TS output is ever required, either fix the
   iteration order (recommendation 1 does this) *and* match the TS
   float-`Rational` path, or accept that Rust is the *more correct* engine and
   treat TS output as an approximation.

---

## 5. Bottom line

- The port is a faithful transcription of the rules and species logic; the
  differences are **not** in the musical logic.
- The observable "different result" is driven by **HashMap iteration-order
  non-determinism** (§2.1) and **priority-queue tie-breaking** (§2.2).
- The "noticeably slower" is driven by **O(n) cursor time recomputation on every
  next/prev** (§3.1-3.2), **SipHash + rebuild churn in `Candidates`** (§3.3),
  and **dynamic dispatch** (§3.4) — all constant-factor, all in the primitives,
  none requiring a change to the algorithm.
