# Search-tree data structures: replace_* and hashing dominate, not the rules

Status: profiling-based (`pprof` flamegraph, release profile,
`crates/debug-server/examples/test.rs`). Supersedes the data-structure notes in
`behavior-performance-audit.md` — the numbers below are from an actual profile,
not source reading.

Scope: the `CounterpointSolver::a_star` search that `test.rs` runs
(4 voices × `species5`, `target_measures = 4`, `batch = 50`, `remove_old = 5`).

---

## 1. Measured profile (the two numbers that matter)

- **~50% of samples**: `replace_*` — i.e. the full-array clones that materialize
  a child score.
- **~25% of samples**: hashing — `HashMap::insert` / `contains_key` on the
  solver's `parents` map.
- The rule closures themselves (the ~20 registered `local_rules` /
  `candidate_rules_*`) do **not** show up as a top cost. The port's rules are
  already cheap enough; they are *not* the bottleneck.

So the search is spending three quarters of its time on *moving the state
around* (copying it and hashing it), not on *evaluating* it.

---

## 2. Where each bucket comes from

### 2.1 `replace_*` (~50%)

Every child node differs from its parent by **one note** (or one chord), yet the
code rebuilds the entire score with full-array clones:

- `Score::replace_voice` — `score.rs:42` — `self.voices.to_vec()` clones the
  whole `Vec<Voice>` (every voice's `name: String` is re-allocated, every
  `Rc` bumped), then wraps it in a fresh `Rc`.
- `CounterpointVoice::replace_measure` — `voice.rs:302` — `self.measures.to_vec()`
  clones the whole `Vec<Measure>`.
- `Harmony::replace_chord` — `chord.rs:250` — `self.elements.to_vec()` clones
  the whole chord array.
- the note write itself — `species.rs:113` — `measure.notes.to_vec()` clones the
  whole note array of the measure.

Worse, the work is done **twice per child**:

1. `Context::fill_in` — `context.rs:283-284` — builds
   `voice.replace_measure(..)` + `s.replace_voice(..)` *only to evaluate the
   local rules*, then throws the resulting `new_score` away and keeps just the
   `Step.measure`.
2. `Node::get_neighbors` — `solver.rs:156-157` — rebuilds
   `v.replace_measure(..)` + `self.score.replace_voice(..)` from that same
   measure to produce the actual child `Score`.

For a score with `V` voices, `M` measures, and `K` notes per measure, one note
write costs `O(K + M + V)` allocations plus `String` clones — and it is paid
twice. At `target_measures = 4` this is invisible; at 100+ measures it is the
dominant cost of the whole program.

### 2.2 Hashing (~25%)

`solver.rs:263` keys the visited set by `Rc<Score>`:

```rust
let mut parents: HashMap<Rc<Score>, Option<Rc<Score>>> = HashMap::new();
```

The key point (often misremembered): **`Rc<T>` does not hash by pointer.** It
hashes the *pointee* (`(**self).hash(..)`), so `Rc<Score>` hashes the entire
`Score` — every voice, every measure, every note, every `Ratio<i64>` through
SipHash — on both the `contains_key` (`solver.rs:323`) and the `insert`
(`solver.rs:326`). That is ~2 full deep hashes per neighbor, plus a full deep
`==` on any hash collision.

The `Candidates` map was already moved off `HashMap` (commit `ee8ff5a`), so what
remains is almost entirely this one deep-hash-per-neighbor on `parents`.

---

## 3. The right mental model

The search is a best-first walk over a tree of **immutable snapshots**. Each
edge is a *local, single-slot* mutation (one note, or one chord). Two properties
matter:

1. We create an **enormous number** of snapshots (millions of nodes).
2. Each snapshot is **comparatively small**, but the *score* they capture is
   going to get large (this repo can't handle big scores yet — the test uses 4
   measures for that reason, not because 4 is representative).

This is exactly the workload persistent (a.k.a. "purely functional") data
structures were invented for: versioned, gradually-mutating small-ish objects
with structural sharing.

The three costs above map onto three persistent-data-structure techniques:

| Measured cost | Structural cause | Technique that removes it |
|---|---|---|
| `replace_*` full clones | `Rc<[T]>` + `to_vec()` copy | persistent sequence with `set(i, x)` = O(log n) + sharing |
| deep `Hash` of the whole score | key = `Rc<Score>` (hashes pointee) | cached/incremental hash in the persistent nodes |
| deep `==` / duplicate work | no canonical identity | pointer/identity short-circuit or interning |

---

## 4. Recommended data structures

### 4.1 Persistent sequence for the mutable-by-one arrays

Replace `Rc<[Voice]>`, `Rc<[Measure]>`, `Rc<[Note]>`, `Rc<[ChordElement]>` with a
**persistent vector** — a bitmapped vector trie (Clojure-style) or RRB tree:

- `set(i, v)` / `update(i, f)` is **O(log₃₂ n)**, sharing every untouched
  sibling. One note write becomes O(log K + log M + log V) with no full-array
  copy and no `String` re-allocation.
- `Clone` is O(1) (bump an `Arc`), so the whole `Rc<..>` wrapper layer disappears.
- Memory per node drops from O(V+M+K) to O(log V + log M + log K) — this is what
  makes *millions of nodes* and *large scores* feasible.

Rust crates, in order of fit:

1. **`rpds::Vector`** — Clojure-style 32-way bitmapped trie. The best fit
   because the `rpds` family is built around exactly this use case and its map
   types carry **cached hashes** (see 4.2).
2. **`im::Vector`** — RRB-tree persistent vector; fine for `set`/`get`, but its
   collections do not cache hashes, so it only solves half the problem.

Note: `set`/`update` require `T: Clone`. `Voice`, `Measure`, `Note`,
`ChordElement` already derive `Clone`.

### 4.2 Cached / incremental hashing for the visited set

The `parents` map's deep hash is the 25%. Persistent tries make this O(1):

- Store a cached `u64` hash in each trie node (children' hashes combined). Then
  `Hash for Score` reads one cached field instead of walking the tree.
- A single-slot mutation updates the hash in O(log n) by combining
  `old_child_hash` and `new_child_hash` at each level — no re-hash of siblings.

This is precisely what `rpds::HashTrieMap` does internally (its nodes cache
hashes). Either use `rpds::HashTrieMap<Rc<Score>, ...>` with a `Score` whose
`Hash` is cached, or — cleaner — give `Score` a memoized `u64` hash and key
`parents` by a small `ScoreKey { hash: u64, ptr: Rc<Score> }`.

A Chess-style **Zobrist** variant is also on the table: precompute a random `u64`
per `(voice_index, measure_index, note_index, pitch)`, and maintain
`score_hash = parent_hash ^ delta` in O(1). It is the fastest possible, but it is
probabilistic and needs a full deep `==` fallback on collision; cached structural
hashes are deterministic and collision-free-by-construction, so prefer those
unless the last constant factor matters.

### 4.3 Identity / interning for dedup

Once hashing is cached, make equality cheap too:

- Persistent tries that share subtrees can short-circuit `==` on root-pointer
  identity: two scores that were built with the same `set(..)` from the same
  parent share the same child pointers, so `ptr_eq` on the roots is often enough.
- Stronger option: **hash-cons / intern** the scores — canonicalize each new
  `Score` against a table, so equal scores collapse to one `Rc<Score>`, and the
  visited set becomes `Rc::ptr_eq`. This costs memory (every distinct score stays
  alive); for a search tree a bounded **transposition table** (evict old entries,
  chess-style) is the standard compromise.

Recommendation: cached hash + pointer short-circuit first; only add interning if
profile shows deep `==` still hurting after 4.1 + 4.2.

---

## 5. Concrete incremental plan

1. **Eliminate the double build (cheap, no new deps).** Have `fill_in` return the
   `new_score` it already constructs (e.g. add `score: Option<Score>` to `Step`,
   set it in `fill_in`, reuse it in `get_neighbors`). Halves the `replace_*` cost
   immediately at `target_measures = 4`.
2. **Swap `Rc<[T]>` → persistent vector** in `Score`/`Voice`/`Measure`/`Harmony`
   (section 4.1). This is the change that scales to large scores; do it against
   the current `Container` trait (its `item`/`len`/`span` map directly onto
   `Vector` indexing; `start()` prefix sums remain O(n) and are a *separate*
   cursor issue).
3. **Cache the score hash** and re-key `parents` (section 4.2). This removes the
   25% hashing bucket.
4. **Optional:** pointer-identity short-circuit in `Score` equality, then a
   bounded transposition table instead of the unbounded `parents` map (which
   currently grows without bound as the frontier expands).

Steps 2 and 3 are the real answer to "what data structures": a persistent
bitmapped-vector-trie for the collections plus a hash-cached key for the visited
set. Together they turn "copy the whole score and re-hash it" into
"patch one slot and read a cached hash," which is the right asymptotics for many
nodes over large scores.

---

## 6. What NOT to do

- Do **not** keep the `Rc<[T]>` arrays and micro-optimize the clone loops. That
  shaves a constant; it cannot change the O(V+M+K)-per-node copy that kills large
  scores.
- Do **not** revert `Candidates` to `HashMap` for "speed" — the ordered `Vec`
  is correct for genuinely tiny candidate sets and already removed its hashing.
- Do **not** "fix" the `parents` map by keying on `Rc` identity assuming it
  dedups cheaply — it currently *does* dedup (by deep value), at the cost of the
  25% hashing; the fix is cached hashing, not disabling dedup.

---

## 7. Bottom line

The rules are not the bottleneck. The solver spends ~75% of its time copying
(`replace_*`) and hashing (`parents`) the score snapshots. For a search over many
gradually-mutating small immutable objects, the right tools are:

- a **persistent vector trie** (`rpds::Vector`) for the collection fields, making
  each one-slot mutation O(log n) with full structural sharing, and
- a **cached structural hash** (plus optional interning/pointer short-circuit)
  for the visited set, making dedup O(1) instead of a full deep SipHash per
  neighbor.
