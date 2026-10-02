//! Local HTTP/SSE server for the `debug-ui`: runs the species-counterpoint
//! solver on a dedicated thread (the solver and its rule closures are `Rc`-based
//! and not `Send`/`Sync`) and streams progress + the final MusicXML to a browser
//! over Server-Sent Events.
//!
//! Endpoints:
//! - `GET /events`      — SSE stream of `progress`, then `ok` or `no-solution`.
//! - `GET /result.mxl`  — the final `Score::to_mxl()` (404 until solved).
//!
//! The frontend is a separate Vite project under `debug-ui/`; its dev server
//! proxies `/events` and `/result.mxl` here.

use std::collections::HashMap;
use std::convert::Infallible;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use axum::{
    Router,
    extract::State,
    http::{StatusCode, header},
    response::{
        IntoResponse,
        sse::{Event as SseEvent, Sse},
    },
    routing::get,
};
use futures_util::{Stream, StreamExt, stream};
use serde_json::json;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

use passacaglia_common::{rational, rational_value};
use passacaglia_core::std_hept;
use passacaglia_core::structure::Container;
use passacaglia_musicxml::ToMxl;
use passacaglia_species_counterpoint::{
    CandidateRule, CounterpointContext, CounterpointScoreBuilder, CounterpointSolver,
    CounterpointSolverProgress, CounterpointSolverRewardStrategy, NonHarmonicType, Parameters,
    Score, import_rules, rules, species5,
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

/// Shared HTTP state: the broadcast sender for live `/events` subscribers plus
/// the mutable solver snapshot.
#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<ServerEvent>,
    shared: Arc<SharedState>,
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

/// Serve the final MusicXML document, or 404 while the solver is still running
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

/// Serialize the playable content of a solved score: one entry per voice with
/// per-note MIDI pitch, start (whole-note units), duration, and a tie flag.
/// Rests carry `"pitch": null`. This mirrors the data the TS debug-ui handed to
/// its jzz-based `play` helper.
fn score_to_playable(score: &Score) -> serde_json::Value {
    let mut voices = Vec::new();
    for voice in score.voices.iter() {
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
/// publishing progress and the final result over the broadcast channel.
fn run_solver(tx: &broadcast::Sender<ServerEvent>, shared: &Arc<SharedState>) {
    let mut ctx = CounterpointContext::new(
        4,
        Parameters {
            measure_length: rational(4),
        },
    );

    ctx.harmony_rules = import_rules![
        rules::enforce_functional_progression_major,
        rules::enforce_valid_chords,
    ];

    ctx.local_rules = import_rules![
        rules::limit_consecutive_leaps,
        rules::forbid_perfects_by_similar_motion,
        rules::forbid_nearby_perfects,
        rules::prioritize_voice_motion,
        rules::enforce_vertical_consonance_with_moving_local,
    ];

    ctx.candidate_rules_before = import_rules![
        rules::enforce_scale_tones,
        rules::enforce_stepwise_around_short_notes,
        rules::enforce_passing_tones,
        rules::enforce_neighbor_tones,
        rules::enforce_suspension,
        rules::forbid_voice_overlapping2,
        rules::avoid_repeat2,
    ];

    ctx.candidate_rules_after = import_rules![
        rules::enforce_melody_intervals,
        rules::enforce_leap_preparation,
        rules::enforce_leap_resolution,
    ];

    ctx.harmonic_tone_rules = import_rules![rules::enforce_chord_tone];

    ctx.non_harmonic_tone_rules = HashMap::from([
        (
            NonHarmonicType::Neighbor,
            import_rules!(CandidateRule; rules::make_neighbor_tone),
        ),
        (
            NonHarmonicType::PassingTone,
            import_rules!(CandidateRule; rules::make_passing_tone),
        ),
        (
            NonHarmonicType::Suspension,
            import_rules!(CandidateRule; rules::make_suspension),
        ),
    ]);

    ctx.allow_unison = true;

    let ctx = Rc::new(ctx);

    let score = CounterpointScoreBuilder::new(ctx.clone())
        .soprano(&species5())
        .alto(&species5())
        .tenor(&species5())
        // .bass(&species5())
        .build(&std_hept::scales::c::MAJOR, None);

    let mut solver = CounterpointSolver::new(ctx.clone());

    let progress_tx = tx.clone();
    let progress_shared = shared.clone();
    solver.set_reporter(move |p| {
        *progress_shared
            .latest_progress
            .lock()
            .expect("progress lock poisoned") = Some(p);
        let _ = progress_tx.send(ServerEvent::Progress(p));
    });

    solver.report_interval = 2000;
    solver.batch = 50;
    solver.remove_old = 5;

    let solution = solver.a_star(
        &score,
        CounterpointSolverRewardStrategy::Constant { value: 50.0 },
    );

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
}

#[tokio::main]
async fn main() {
    let (tx, _rx) = broadcast::channel::<ServerEvent>(1024);
    let shared = Arc::new(SharedState {
        status: Mutex::new(SolveStatus::Running),
        latest_progress: Mutex::new(None),
    });

    {
        let tx = tx.clone();
        let shared = shared.clone();
        std::thread::spawn(move || run_solver(&tx, &shared));
    }

    let app = Router::new()
        .route("/events", get(events))
        .route("/result.mxl", get(result_mxl))
        .with_state(AppState { tx, shared });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("bind debug-server to 127.0.0.1:8080");
    println!("debug-server listening on http://127.0.0.1:8080");

    axum::serve(listener, app).await.expect("run debug-server");
}
