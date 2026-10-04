//! Local HTTP/SSE server for the `debug-ui`: runs the species-counterpoint
//! solver on a dedicated thread (the solver and its rule closures are `Rc`-based
//! and not `Send`/`Sync`) and streams progress + the final `MusicXML` to a browser
//! over Server-Sent Events.
//!
//! Endpoints:
//! - `GET /events`      — SSE stream of `progress`, then `ok` or `no-solution`.
//! - `GET /result.mxl`  — the final `Score::to_mxl()` (404 until solved).
//!
//! The frontend is a separate Vite project under `debug-ui/`; its dev server
//! proxies `/events` and `/result.mxl` here.

use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use axum::{
    Router,
    extract::{Path, State},
    http::{StatusCode, header},
    response::{
        IntoResponse,
        sse::{Event as SseEvent, Sse},
    },
    routing::get,
};
use futures_util::{Stream, StreamExt, stream};
#[allow(unused_imports)]
use passacaglia_macros::{std_hept_interval as interval, std_hept_pitch as pitch};
use serde_json::json;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

use passacaglia_common::{rational, rational_value};
use passacaglia_core::std_hept::{scales};
use passacaglia_core::structure::Container;
use passacaglia_musicxml::ToMxl;

#[allow(unused_imports)]
use passacaglia_species_counterpoint::{
    CounterpointContext, CounterpointScoreBuilder, CounterpointSolver, CounterpointSolverProgress, CounterpointSolverRewardStrategy, MelodicSettings, NodeKind, NonHarmonicType, Parameters, Score, SearchNode, define_imitation, imitation, rules, species1, species3, species5,
};

/// A solver event that crosses the thread boundary. Every variant is `Send`.
#[derive(Clone)]
enum ServerEvent {
    Progress(CounterpointSolverProgress),
    Ok {
        mxl: String,
        playable: serde_json::Value,
    },
    NoSolution,
}

impl ServerEvent {
    fn to_sse(&self) -> SseEvent {
        let json = match self {
            ServerEvent::Progress(p) => json!({
                "type": "progress",
                "progress": p.measure_index,
                "furthest": p.furthest,
                "total": p.total_measures,
                "iteration": p.iteration,
            })
            .to_string(),
            ServerEvent::Ok { mxl, playable } => {
                json!({ "type": "ok", "mxl": mxl, "playable": playable }).to_string()
            }
            ServerEvent::NoSolution => json!({ "type": "no-solution" }).to_string(),
        };
        SseEvent::default().data(json)
    }
}

/// Terminal state of the solve, published once by the solver thread.
enum SolveStatus {
    Running,
    Ok {
        mxl: String,
        playable: serde_json::Value,
    },
    NoSolution,
}

/// Solver state shared with the HTTP handlers. The solver thread keeps these
/// current; handlers read them to replay the latest snapshot to late-connecting
/// clients (the solve often finishes before a browser finishes loading).
struct SharedState {
    status: Mutex<SolveStatus>,
    latest_progress: Mutex<Option<CounterpointSolverProgress>>,
}

/// A query for search-tree data, dispatched from an HTTP handler to the solver
/// thread (which owns the `Rc`-based tree and cannot share it directly).
enum TreeQueryKind {
    Meta,
    Node { id: usize },
    Score { id: usize },
}

struct TreeQuery {
    kind: TreeQueryKind,
    reply: tokio::sync::oneshot::Sender<serde_json::Value>,
}

/// Shared HTTP state: the broadcast sender for live `/events` subscribers plus
/// the mutable solver snapshot and the channel used to query the search tree.
#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<ServerEvent>,
    shared: Arc<SharedState>,
    tree_tx: std::sync::mpsc::Sender<TreeQuery>,
}

/// Stream solver events to an SSE subscriber. A snapshot (latest progress, then
/// `ok`/`no-solution` if already finished) is replayed first so a client that
/// connects after the solve still sees the result; live events follow. Lagged
/// messages (a slow client falling behind the broadcast buffer) are dropped
/// rather than terminating the stream.
async fn events(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<SseEvent, Infallible>>> {
    let snapshot = {
        let mut initial = Vec::new();
        if let Some(p) = *state
            .shared
            .latest_progress
            .lock()
            .expect("progress lock poisoned")
        {
            initial.push(Ok::<SseEvent, Infallible>(
                ServerEvent::Progress(p).to_sse(),
            ));
        }
        match &*state.shared.status.lock().expect("status lock poisoned") {
            SolveStatus::Ok { mxl, playable } => {
                initial.push(Ok::<SseEvent, Infallible>(
                    ServerEvent::Ok {
                        mxl: mxl.clone(),
                        playable: playable.clone(),
                    }
                    .to_sse(),
                ));
            }
            SolveStatus::NoSolution => {
                initial.push(Ok::<SseEvent, Infallible>(ServerEvent::NoSolution.to_sse()));
            }
            SolveStatus::Running => {}
        }
        initial
    };

    let live = BroadcastStream::new(state.tx.subscribe()).filter_map(|item| async move {
        let Ok(ev) = item else {
            return None;
        };
        Some(Ok::<SseEvent, Infallible>(ev.to_sse()))
    });

    Sse::new(stream::iter(snapshot).chain(live))
}

/// Serve the final `MusicXML` document, or 404 while the solver is still running
/// (or if it found no solution).
async fn result_mxl(State(state): State<AppState>) -> impl IntoResponse {
    let mxl = match &*state.shared.status.lock().expect("status lock poisoned") {
        SolveStatus::Ok { mxl, .. } => Some(mxl.clone()),
        SolveStatus::Running | SolveStatus::NoSolution => None,
    };
    let Some(mxl) = mxl else {
        return StatusCode::NOT_FOUND.into_response();
    };
    (
        [(
            header::CONTENT_TYPE,
            "application/vnd.recordare.musicxml+xml",
        )],
        mxl,
    )
        .into_response()
}

/// Send a search-tree query to the solver thread and await its JSON reply.
async fn tree_query(
    tx: &std::sync::mpsc::Sender<TreeQuery>,
    kind: TreeQueryKind,
) -> serde_json::Value {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    if tx.send(TreeQuery { kind, reply: reply_tx }).is_err() {
        return json!({ "error": "solver thread unavailable" });
    }
    reply_rx
        .await
        .unwrap_or_else(|_| json!({ "error": "solver thread dropped" }))
}

fn json_response(json: &serde_json::Value) -> axum::response::Response {
    ([(header::CONTENT_TYPE, "application/json")], json.to_string()).into_response()
}

/// `GET /tree` — search-tree metadata: node count, root id, goal id and the id
/// path (root → goal) of the accepted solution.
async fn tree_meta(State(state): State<AppState>) -> impl IntoResponse {
    json_response(&tree_query(&state.tree_tx, TreeQueryKind::Meta).await)
}

/// `GET /tree/node/{id}` — a node's attributes plus the attributes of its
/// children (so the frontend can lazily expand one node at a time).
async fn tree_node(State(state): State<AppState>, Path(id): Path<usize>) -> impl IntoResponse {
    json_response(&tree_query(&state.tree_tx, TreeQueryKind::Node { id }).await)
}

/// `GET /tree/score/{id}` — the `MusicXML` of the score snapshot at `id`.
async fn tree_score(State(state): State<AppState>, Path(id): Path<usize>) -> impl IntoResponse {
    json_response(&tree_query(&state.tree_tx, TreeQueryKind::Score { id }).await)
}

fn kind_str(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Initial => "initial",
        NodeKind::Harmony => "harmony",
        NodeKind::Note => "note",
    }
}

/// Reconstruct the root → goal path by walking parent pointers backwards.
fn solution_path(nodes: &[SearchNode], goal: Option<usize>) -> Vec<usize> {
    let Some(mut id) = goal else {
        return Vec::new();
    };
    let mut path = vec![id];
    while let Some(node) = nodes.get(id)
        && let Some(parent) = node.parent
    {
        path.push(parent);
        id = parent;
    }
    path.reverse();
    path
}

/// Aggregate per-node subtree statistics used to rank "untaken" branches:
/// `best_priority` (lowest search priority seen in the subtree), `max_depth`
/// (furthest measure reached) and `subtree_size` (number of nodes).
///
/// Children always carry a higher `id` than their parent, so a single reverse
/// pass accumulates the children's statistics before the parent's.
fn subtree_stats(
    nodes: &[SearchNode],
    reward: f64,
) -> (Vec<f64>, Vec<usize>, Vec<usize>) {
    let n = nodes.len();
    let mut best_priority = vec![0.0_f64; n];
    let mut max_depth = vec![0_usize; n];
    let mut subtree_size = vec![1_usize; n];
    for i in (0..n).rev() {
        let node = &nodes[i];
        let mut bp = node.cost - reward * node.n_step;
        let mut md = node.measure_index;
        let mut size = 1_usize;
        for &child in &node.children {
            bp = bp.min(best_priority[child]);
            md = md.max(max_depth[child]);
            size += subtree_size[child];
        }
        best_priority[i] = bp;
        max_depth[i] = md;
        subtree_size[i] = size;
    }
    (best_priority, max_depth, subtree_size)
}

fn node_json(
    nodes: &[SearchNode],
    id: usize,
    start: usize,
    on_solution: &HashSet<usize>,
    best_priority: &[f64],
    max_depth: &[usize],
    subtree_size: &[usize],
) -> serde_json::Value {
    let n = &nodes[id];
    json!({
        "id": n.id,
        "parent": n.parent,
        "measureIndex": n.measure_index,
        "voiceIndex": n.voice_index,
        "kind": kind_str(n.kind),
        "nStep": n.n_step,
        "cost": n.cost,
        "thisCost": n.this_cost,
        "debug": n.debug,
        "isGoal": n.is_goal,
        "nExpanded": n.n_expanded,
        "isStart": n.id == start,
        "onSolutionPath": on_solution.contains(&n.id),
        "bestPriority": best_priority[n.id],
        "maxDepth": max_depth[n.id],
        "subtreeSize": subtree_size[n.id],
    })
}

/// Serialize the playable content of a solved score: one entry per voice with
/// per-note MIDI pitch, start (whole-note units), duration, and a tie flag.
/// Rests carry `"pitch": null`. This mirrors the data the TS debug-ui handed to
/// its jzz-based `play` helper.
fn score_to_playable(score: &Score) -> serde_json::Value {
    let mut voices = Vec::new();
    for voice in &score.voices {
        let mut notes = Vec::new();
        for (mi, measure) in voice.measures().iter().enumerate() {
            let measure_start = voice.start(mi);
            for (ni, note) in measure.notes.iter().enumerate() {
                let start = measure_start + measure.start(ni);
                notes.push(json!({
                    "pitch": note.pitch.map(|p| rational_value(p.to_midi())),
                    "start": rational_value(start),
                    "duration": rational_value(note.duration),
                    "tied": note.is_tied(),
                }));
            }
        }
        voices.push(json!({
            "index": voice.index(),
            "name": voice.name(),
            "notes": notes,
        }));
    }
    json!({ "voices": voices })
}

/// Build the fixed rule/species configuration (mirrors
/// `crates/musicxml/examples/test.rs`) and run the solver on a dedicated thread,
/// publishing progress and the final result over the broadcast channel. Once the
/// solve finishes, the thread keeps the search tree alive and answers lazy
/// `/tree` queries from the HTTP handlers.
fn run_solver(
    tx: &broadcast::Sender<ServerEvent>,
    shared: &Arc<SharedState>,
    tree_rx: std::sync::mpsc::Receiver<TreeQuery>,
) {
    let mut ctx = CounterpointContext::new(
        12,
        Parameters {
            measure_length: rational(4),
        },
    );

    ctx.harmony_rules = vec![
        // rules::enforce_functional_progression_major(),
        rules::enforce_functional_progression_minor(),
        rules::enforce_valid_chords(),
    ];

    ctx.local_rules = vec![
        rules::limit_consecutive_leaps(),
        rules::forbid_perfects_by_similar_motion(),
        rules::forbid_nearby_perfects(),
        rules::prioritize_voice_motion(),
        rules::enforce_vertical_consonance_with_moving_local(),
    ];

    ctx.candidate_rules_before = vec![
        rules::enforce_scale_tones(),
        rules::enforce_minor(pitch!("a")),
        rules::enforce_stepwise_around_short_notes(),
        rules::enforce_passing_tones(),
        rules::enforce_neighbor_tones(),
        rules::enforce_suspension(),
        rules::forbid_voice_overlapping2(),
        rules::avoid_repeat2(),
        rules::avoid_stagnation(),
    ];

    ctx.candidate_rules_after = vec![
        rules::enforce_melody_intervals(),
        rules::enforce_leap_preparation(),
        rules::enforce_leap_resolution(),
    ];

    ctx.harmonic_tone_rules = vec![
        rules::enforce_chord_tone()
    ];

    ctx.non_harmonic_tone_rules = HashMap::from([
        (
            NonHarmonicType::Neighbor,
            vec![rules::make_neighbor_tone()],
        ),
        (
            NonHarmonicType::PassingTone,
            vec![rules::make_passing_tone()],
        ),
        (
            NonHarmonicType::Suspension,
            vec![rules::make_suspension()],
        ),
    ]);

    ctx.allow_unison = true;

    let ctx = Rc::new(ctx);

    let score = CounterpointScoreBuilder::new(ctx.clone())
        // .soprano(&species5())
        // .alto(&define_imitation(
        //     MelodicSettings::unrestricted(),
        //     0, 1, |x| {
        //         vec![
        //             x.add(&interval!("-d5")),
        //             x.add(&interval!("-P5")),
        //             x.add(&interval!("-A5")),
        //         ]
        //     }))
        // .bass(&species5())

        .soprano(&species5())
        .alto(&species5())
        // .tenor(&species1())
        .bass(&species1())

        // .build(&scales::major(pitch!("c")), None)
        .build(&scales::minor(pitch!("a")), None)
    ;

    let mut solver = CounterpointSolver::new(ctx.clone());

    let progress_tx = tx.clone();
    let progress_shared = shared.clone();
    solver.set_reporter(move |p| {
        *progress_shared.latest_progress.lock()
            .expect("progress lock poisoned") = Some(p);
        let _ = progress_tx.send(ServerEvent::Progress(p));
    });

    solver.report_interval = 1000;
    solver.batch = 100;
    solver.remove_old = 6;

    let reward = 30.0;
    let solution = solver.run(&score, CounterpointSolverRewardStrategy::Constant { value: reward });

    if let Some(s) = solution {
        let mxl = s.to_mxl();
        let playable = score_to_playable(&s);
        *shared.status.lock().expect("status lock poisoned") = SolveStatus::Ok {
            mxl: mxl.clone(),
            playable: playable.clone(),
        };
        let _ = tx.send(ServerEvent::Ok { mxl, playable });
        println!("solver finished: found a solution");
    } else {
        *shared.status.lock().expect("status lock poisoned") = SolveStatus::NoSolution;
        let _ = tx.send(ServerEvent::NoSolution);
        println!("solver finished: no solution");
    }

    let start_id = solver.start_node_id().unwrap_or(0);
    let goal_id = solver.goal_node_id();
    let target_measures = ctx.target_measures;
    let search_nodes: Vec<SearchNode> = solver
        .search_nodes()
        .map_or_else(Vec::new, <[SearchNode]>::to_vec);

    serve_tree_queries(
        tree_rx,
        &search_nodes,
        start_id,
        goal_id,
        target_measures,
        reward,
    );
}

/// Answer lazy `/tree` queries from the HTTP handlers for as long as the solver
/// thread lives.
fn serve_tree_queries(
    tree_rx: std::sync::mpsc::Receiver<TreeQuery>,
    search_nodes: &[SearchNode],
    start_id: usize,
    goal_id: Option<usize>,
    target_measures: usize,
    reward: f64,
) {
    let solution_path = solution_path(search_nodes, goal_id);
    let on_solution: HashSet<usize> = solution_path.iter().copied().collect();
    let (best_priority, max_depth, subtree_size) = subtree_stats(search_nodes, reward);

    for query in tree_rx {
        let reply = match query.kind {
            TreeQueryKind::Meta => json!({
                "nodeCount": search_nodes.len(),
                "start": start_id,
                "goal": goal_id,
                "solutionPath": solution_path,
                "targetMeasures": target_measures,
            }),
            TreeQueryKind::Node { id } => {
                if id < search_nodes.len() {
                    let mut obj = node_json(
                        search_nodes,
                        id,
                        start_id,
                        &on_solution,
                        &best_priority,
                        &max_depth,
                        &subtree_size,
                    );
                    let children = search_nodes[id]
                        .children
                        .iter()
                        .map(|&c| {
                            node_json(
                                search_nodes,
                                c,
                                start_id,
                                &on_solution,
                                &best_priority,
                                &max_depth,
                                &subtree_size,
                            )
                        })
                        .collect::<Vec<_>>();
                    obj["children"] = json!(children);
                    obj
                } else {
                    json!({ "error": "node not found" })
                }
            }
            TreeQueryKind::Score { id } => {
                if id < search_nodes.len() {
                    json!({ "mxl": search_nodes[id].score.to_mxl() })
                } else {
                    json!({ "error": "node not found" })
                }
            }
        };
        let _ = query.reply.send(reply);
    }
}

#[tokio::main]
async fn main() {
    let (tx, _rx) = broadcast::channel::<ServerEvent>(1024);
    let (tree_tx, tree_rx) = std::sync::mpsc::channel::<TreeQuery>();
    let shared = Arc::new(SharedState {
        status: Mutex::new(SolveStatus::Running),
        latest_progress: Mutex::new(None),
    });

    {
        let tx = tx.clone();
        let shared = shared.clone();
        std::thread::spawn(move || run_solver(&tx, &shared, tree_rx));
    }

    let app = Router::new()
        .route("/events", get(events))
        .route("/result.mxl", get(result_mxl))
        .route("/tree", get(tree_meta))
        .route("/tree/node/{id}", get(tree_node))
        .route("/tree/score/{id}", get(tree_score))
        .with_state(AppState { tx, shared, tree_tx });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("bind debug-server to 127.0.0.1:8080");
    println!("debug-server listening on http://127.0.0.1:8080");

    axum::serve(listener, app).await.expect("run debug-server");
}
