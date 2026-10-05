//! Search-tree query endpoints and the solver-thread responder that answers
//! them.
//!
//! The search tree is `Rc`-based and lives on the solver thread, so it cannot be
//! shared with the HTTP handlers directly. Instead a handler sends a
//! [`TreeQuery`] over an `mpsc` channel; the solver thread answers with JSON over
//! a `tokio` oneshot channel. This keeps the tree lazily queryable without ever
//! moving it across threads.
//!
//! Endpoints:
//! - `GET /tree`                — node count, root id, goal id, the accepted
//!   solution path, and the target measure count.
//! - `GET /tree/node/{id}`      — a node's attributes plus its children.
//! - `GET /tree/score/{id}`     — the `MusicXML` of the node's score snapshot.

use std::collections::HashSet;

use axum::{
    extract::{Path, State},
    http::header,
    response::{IntoResponse, Response},
};

use passacaglia_musicxml::ToMxl;
use passacaglia_species_counterpoint::{NodeKind, SearchNode};
use serde_json::json;

use crate::AppState;

/// The kind of search-tree data an HTTP handler wants from the solver thread.
enum TreeQueryKind {
    Meta,
    Node { id: usize },
    Score { id: usize },
}

/// A query for search-tree data, dispatched from an HTTP handler to the solver
/// thread (which owns the `Rc`-based tree and cannot share it directly).
pub struct TreeQuery {
    kind: TreeQueryKind,
    reply: tokio::sync::oneshot::Sender<serde_json::Value>,
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

fn json_response(json: &serde_json::Value) -> Response {
    ([(header::CONTENT_TYPE, "application/json")], json.to_string()).into_response()
}

/// `GET /tree` — search-tree metadata: node count, root id, goal id and the id
/// path (root → goal) of the accepted solution.
pub async fn tree_meta(State(state): State<AppState>) -> Response {
    json_response(&tree_query(&state.tree_tx, TreeQueryKind::Meta).await)
}

/// `GET /tree/node/{id}` — a node's attributes plus the attributes of its
/// children (so the frontend can lazily expand one node at a time).
pub async fn tree_node(State(state): State<AppState>, Path(id): Path<usize>) -> Response {
    json_response(&tree_query(&state.tree_tx, TreeQueryKind::Node { id }).await)
}

/// `GET /tree/score/{id}` — the `MusicXML` of the score snapshot at `id`.
pub async fn tree_score(State(state): State<AppState>, Path(id): Path<usize>) -> Response {
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
    let mut max_measure_index = vec![0_usize; n];
    let mut subtree_size = vec![1_usize; n];
    for i in (0..n).rev() {
        let node = &nodes[i];
        let mut bp = node.cost - reward * node.n_step;
        let mut md = node.measure_index;
        let mut size = 1_usize;
        for &child in &node.children {
            bp = bp.min(best_priority[child]);
            md = md.max(max_measure_index[child]);
            size += subtree_size[child];
        }
        best_priority[i] = bp;
        max_measure_index[i] = md;
        subtree_size[i] = size;
    }
    (best_priority, max_measure_index, subtree_size)
}

fn node_json(
    nodes: &[SearchNode],
    id: usize,
    start: usize,
    on_solution: &HashSet<usize>,
    best_priority: &[f64],
    max_measure_index: &[usize],
    subtree_size: &[usize],
) -> serde_json::Value {
    let n = &nodes[id];
    json!({
        "id": n.id,
        "depth": n.depth,
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
        "maxMeasureIndex": max_measure_index[n.id],
        "subtreeSize": subtree_size[n.id],
    })
}

/// Answer lazy `/tree` queries from the HTTP handlers for as long as the solver
/// thread lives.
pub fn serve_tree_queries(
    tree_rx: std::sync::mpsc::Receiver<TreeQuery>,
    search_nodes: &[SearchNode],
    start_id: usize,
    goal_id: Option<usize>,
    target_measures: usize,
    reward: f64,
) {
    let solution_path = solution_path(search_nodes, goal_id);
    let on_solution: HashSet<usize> = solution_path.iter().copied().collect();
    let (best_priority, max_measure_index, subtree_size) = subtree_stats(search_nodes, reward);

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
                        &max_measure_index,
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
                                &max_measure_index,
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
