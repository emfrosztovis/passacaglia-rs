# Debug GUI server — design report

Status: plan. No implementation yet. This documents how to stand up a local
server that visualizes the Rust solver's results, starting from the reference
TypeScript debug UI (`passacaglia-ts/apps/debug-ui`) and progressively replacing
its in-browser worker with a Rust-backed server.

## 1. Goals

1. Reproduce the useful parts of the TS `debug-ui` for the Rust port: see the
   rendered MusicXML score and watch solver progress in real time.
2. **Stage 1 (now):** serve only the MXL score and the progress stream. No tree.
3. **Stage 2 (later):** serve the search tree, but *not* all at once (the TS UI
   loads the whole tree and gives up past 10k nodes). Work out a protocol for
   streaming/querying it, and a better visualization for large trees rather than
   bailing out.

## 2. What the TS `debug-ui` does today (the reference)

The app is a Vite/Vue frontend plus a Web Worker that owns the solver.

- `Main.worker.ts` — builds the `CounterpointContext` (rules, costs,
  `allowUnison`), builds a `Score` from a species (e.g. `Species5`), runs
  `solver.aStar(...)`, and `postMessage`s back to the main thread:
  - `log` — debug log lines (via `setLogger`),
  - `progress` — `{ measureIndex, furthest, totalMeasures, iteration }`,
  - `ok` — `{ data: Serialized<VoiceData>[], source: <MusicXML string> }`,
  - `no-solution`,
  - `graph` — the **entire** search tree as a graphology/d3 export, gated by
    `if (result && solver.parents!.size < 10000)`, and
  - `query-score-result` — the MXL for a single focused node, in reply to a
    `query-score` request from the UI.
- `App.vue` — shows a progress widget, renders `source` (MXL) with
  `opensheetmusicdisplay` (`MusicScore.vue`), renders the tree with `sigma.js`
  (`SigmaView.vue`), and offers play/download.
- `MusicScore.vue` / `SigmaView.vue` — thin wrappers around OSMD and sigma.

Key facts to carry over:

- The progress schema is exactly `CounterpointSolverProgress` in Rust.
- The "score" payload is just the MusicXML string produced by the port's
  `Score::to_mxl()`.
- The tree is built from `solver.parents` (a `child -> parent` map) plus
  `solver.startNode` and per-node fields (`measureIndex`, `voiceIndex`,
  `thisCost`, `debug`, `isGoal`, `nExpanded`). The 10k-node guard is the part we
  want to replace, not keep.

## 3. What the Rust port already has

- `CounterpointSolver` (`crates/species-counterpoint/src/solver.rs`):
  - `a_star(&mut self, &Score, CounterpointSolverRewardStrategy) -> Option<Arc<Score>>`,
  - `on_progress: Option<Box<dyn FnMut(CounterpointSolverProgress)>>`,
  - `batch`, `remove_old`, `report_interval`,
  - `parents() -> Option<&HashMap<Arc<Score>, Option<Arc<Score>>>>` and
    `start_node() -> Option<&Arc<Score>>`.
- `Score::to_mxl()` (`crates/musicxml`) — produces the MusicXML document.
- The example `crates/musicxml/examples/test.rs` already wires up the full rule
  set + `on_progress` (println) + prints MXL on success. This is the seed for
  the server: it proves the config, the progress callback, and the MXL path all
  work end-to-end.

Gaps for stage 2 (tree serving), which do **not** block stage 1:

- `Node` in `solver.rs` is private. `parents` only maps `Score -> Option<Score>`;
  the per-node fields (`measure_index`, `voice_index`, `n_step`, `cost`,
  `this_cost`, `debug`, `is_goal`, and a would-be `n_expanded`) are dropped when
  the node is consumed. To serve the tree we must persist a public per-node
  record alongside `parents`.
- There is no server crate and no serde anywhere in the workspace (only
  `num-rational`/`num-traits`/`quick-xml`). Progress is a tiny struct, so manual
  JSON is trivial; tree payloads will want a real serializer.

## 4. Proposed architecture

Add a `debug-server` binary crate (e.g. `crates/debug-server`, added to the
workspace `members`), depending on `species-counterpoint`, `musicxml`, `common`,
`core`. It owns:

1. the fixed rule/species configuration (copied from `test.rs`),
2. the solver run, on a **dedicated std thread**,
3. an HTTP/SSE (and later WebSocket) front to the browser.

```
browser (reused TS debug-ui, minus worker)
   │  GET /            static page
   │  GET /events      Server-Sent Events: progress… then ok|no-solution
   │  GET /result.mxl  final MusicXML
   ▼
debug-server
   ├─ http layer (axum)  ── channels ──► solver thread
   └─ solver thread (std thread; not Send/Sync-bound)
```

Threading matters: the solver and its rule closures are intentionally
`Arc`-backed and **not** `Send`/`Sync` (see `lib.rs`'s
`clippy::arc_with_non_send_sync`). Keep the solver on its own `std::thread` and
communicate through `std::sync::mpsc` (or `tokio` `mpsc`) channels that carry
plain `Send` payloads (`CounterpointSolverProgress`, `String` MXL). The HTTP
handlers never touch solver internals directly.

Recommend **axum + tokio** for the transport: SSE (and later WebSocket) is
first-class and the frontend already speaks `EventSource`/`fetch`. A simpler
`tiny_http`/`rouille` alternative works if we avoid SSE and just poll, but SSE is
worth the dependency for live progress.

## 5. Stage 1 — serve MXL + progress (concrete)

### 5.1 Solver wiring

Adapt `test.rs` so the progress callback pushes over a channel instead of
`println!`:

```rust
// solver thread
let (tx, rx) = std::sync::mpsc::channel::<Event>();
solver.on_progress = Some(Box::new(move |p| {
    let _ = tx.send(Event::Progress(p));
}));
let result = solver.a_star(&score, Reward::Constant { value: 25.0 });
let _ = tx.send(match result {
    Some(s) => Event::Ok(s.to_mxl()),
    None    => Event::NoSolution,
});
```

`CounterpointSolverProgress` is `Copy` and `Send`, so it crosses the channel as-is.

### 5.2 Wire protocol (SSE)

`GET /events` streams JSON lines with a `type` discriminator:

```
data: {"type":"progress","iteration":458,"furthest":3,"measureIndex":3,"totalMeasures":4}
data: {"type":"ok","mxl":"<?xml version=\"1.0\"?>…"}
```

Terminal events are `ok` (with the MXL string) or `no-solution`. Keep the field
names identical to the TS `MainMessage` (`measureIndex`, `furthest`,
`totalMeasures`, `iteration`) so the existing frontend components can be reused
nearly unchanged.

Convenience endpoint for the final artifact:

- `GET /result.mxl` — `Content-Type: application/vnd.recordare.musicxml+xml`,
  body = the final `Score::to_mxl()` (or 404 until solved).

### 5.3 Frontend

Stage 1 does not need sigma.js at all. Two options, in order of preference:

1. **Reuse the TS `debug-ui` frontend**, delete `Main.worker.ts`, and replace the
   worker with a small `debugServerClient.ts` that does
   `new EventSource('/events')` + `fetch('/result.mxl')`. `App.vue` drops the
   `graph`/`query-score` handling and keeps the progress widget + `MusicScore.vue`.
   This reuses OSMD for free.
2. **Minimal Rust-served HTML**: a static page that subscribes to SSE, renders a
   progress bar, and embeds OSMD from a `<script>` to draw `result.mxl`. Less
   parity, but zero frontend build step.

## 6. Stage 2 — serving the search tree (open design)

### 6.1 The problem

`parents` grows with every distinct score visited. The TS UI serializes the whole
thing and skips rendering past 10k nodes. We want to *not* give up — serve the
tree incrementally and render a better view when it is large.

### 6.2 What to serve, and how to identify nodes

The Rust `parents` map is already a `child -> parent` forest (start node has a
`None` parent). To reconstruct the TS visualization we need, per node:

- a stable wire id,
- `parent` id,
- `measure_index`, `voice_index`, `this_cost`, `debug`, `is_goal`, and
  (optionally) `n_expanded` (currently not tracked in Rust — add it).

Rust dedups by structural `Score` (`Hash`/`Eq`), so `Arc<Score>` is a natural
node identity, but it is not a cheap wire id. Assign monotonic `u64` ids in visit
order via an interning arena (`HashMap<Arc<Score>, u64>`), or reuse the score's
structural hash. This is the same "Score hash for dedup" question already noted
in `porting-species-counterpoint.md` §6.3/§13.

**Required solver change:** add an optional, opt-in node record (so normal solves
pay nothing). For example:

```rust
pub struct NodeRecord {
    pub score: Arc<Score>,
    pub parent: Option<Arc<Score>>,
    pub measure_index: usize,
    pub voice_index: Option<usize>,
    pub cost: f64,
    pub this_cost: f64,
    pub debug: String,
    pub is_goal: bool,
    pub n_expanded: usize,
}
```

`CounterpointSolver` gains a `record_nodes: bool` flag; when set, `get_neighbors`
records each `Node` (and its parent) into a `Vec`/`HashMap` retained after the
search, keyed by `Arc<Score>`. The existing `parents` map can be replaced by (or
derived from) this table.

### 6.3 Protocol options (do not serve it all at once)

- **Windowed query:** `GET /tree?around=<id>&depth=<n>` returns the local
  neighborhood (node + n levels of ancestors/descendants). The client fetches on
  pan/zoom, exactly like the TS `query-score` handler but for a subtree.
- **Lazy expansion:** client requests children of a node on demand
  (`GET /nodes/<id>/children`). Server already stores `parent`; adding a
  `children` index (or deriving it) makes this O(1).
- **Server-side pruning/summary:** keep only the winning path plus a bounded
  frontier (nearest-to-goal by cost), and aggregate the rest (e.g. counts per
  measure). This bounds the payload and is the seed of a "big tree" view.

Whatever the protocol, batch pagination (`limit`/`cursor`) rather than a single
blob, so memory on the client stays flat.

### 6.4 Better visualization for large trees (open question)

Instead of giving up at 10k nodes:

- render only the winning path + a small explored frontier by default,
- level-of-detail: coarsen distant subtrees into single "explored region" nodes
  with a count/aggregate cost,
- highlight the path to the goal and the "furthest" frontier,
- keep the `query-score` affordance so hovering a node still renders its score.

This is the part to prototype after stage 1; the tree protocol in §6.3 should be
designed so any of these renderers can be plugged in without changing the wire
format.

## 7. Milestones

1. `crates/debug-server` binary that runs the `test.rs` config and streams
   progress + MXL over SSE (and serves a static page). No tree.
2. Port the TS frontend to fetch/SSE (drop the worker), reuse `MusicScore.vue`.
3. Add `record_nodes` + `NodeRecord` to the solver (opt-in).
4. Add the tree query endpoint(s) from §6.3.
5. Prototype the large-tree visualization from §6.4.

## 8. Open questions

- axum/tokio vs. a synchronous mini-server — decide by how much we value SSE/WS
  ergonomics vs. keeping the dependency surface tiny.
- Node identity: sequential arena id vs. structural score hash.
- Whether `NodeRecord` should always be recorded (cheap enough?) or gated behind
  the flag.
- Exact tree protocol: windowed `around/depth` vs. `children` + `summary`.
- How aggressive the server-side pruning/summarization should be, and what the
  "coarse region" node should carry (count, cost range, representative path).
