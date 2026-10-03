use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::collections::{BinaryHeap, HashMap};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use passacaglia_common::{rational_value, Rational};
use passacaglia_core::structure::Container;

use crate::context::CounterpointContext;
use crate::score::Score;
use crate::voice::Voice;

const POWER: f64 = 0.9;

/// The reward strategy for the solver.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CounterpointSolverRewardStrategy {
    Constant { value: f64 },
}

/// Progress reported by the solver.
#[derive(Debug, Clone, Copy)]
pub struct CounterpointSolverProgress {
    pub measure_index: usize,
    pub furthest: usize,
    pub total_measures: usize,
    pub iteration: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NodeKind {
    Initial,
    Harmony,
    Note,
}

#[derive(Debug, Clone, Copy)]
enum Target {
    Chord,
    Measure { voice_index: usize },
}

/// A search node holding a score snapshot.
#[derive(Clone)]
#[allow(dead_code)]
struct Node {
    pub score: Rc<Score>,
    pub score_hash: u64,
    ctx: Rc<CounterpointContext>,
    pub measure_index: usize,
    pub voice_index: Option<usize>,
    pub kind: NodeKind,
    pub n_step: f64,
    pub cost: f64,
    pub this_cost: f64,
    pub debug: String,
    pub is_goal: bool,
    target: Option<Target>,
}

fn score_hash(score: &Score) -> u64 {
    let mut h = DefaultHasher::new();
    score.hash(&mut h);
    h.finish()
}

fn find_writable(score: &Score, ctx: &CounterpointContext, measure_index: usize) -> Option<Target> {
    let ch = score.harmony.cursor(measure_index).expect("harmony cursor");
    if ch.chord.is_none() && !ctx.harmony_rules.is_empty() {
        return Some(Target::Chord);
    }

    let mut earliest = None;
    let mut time: Option<Rational> = None;
    for (voice_index, v) in score.voices.iter().enumerate().rev() {
        if !matches!(v, Voice::Counterpoint(_)) {
            continue;
        }
        let Some(m) = v.cursor(measure_index) else {
            continue;
        };
        let Some(wp) = m.writable_position() else {
            continue;
        };
        if time.is_none_or(|t| wp < t) {
            time = Some(wp);
            earliest = Some(voice_index);
        }
    }
    earliest.map(|voice_index| Target::Measure { voice_index })
}

impl Node {
    fn new(
        score: Rc<Score>,
        ctx: Rc<CounterpointContext>,
        measure_index: usize,
        voice_index: Option<usize>,
        kind: NodeKind,
        n_step: f64,
        cost: f64,
        this_cost: f64,
        debug: String,
    ) -> Node {
        let mut mi = measure_index;
        let mut target = None;
        while mi < ctx.target_measures {
            if let Some(t) = find_writable(&score, &ctx, mi) {
                target = Some(t);
                break;
            }
            mi += 1;
        }
        let is_goal = target.is_none();
        let score_hash = score_hash(&score);
        Node {
            score,
            score_hash,
            ctx,
            measure_index: mi,
            voice_index,
            kind,
            n_step,
            cost,
            this_cost,
            debug,
            is_goal,
            target,
        }
    }

    fn get_neighbors(&self) -> Vec<Node> {
        let Some(target) = self.target else {
            return Vec::new();
        };
        match target {
            Target::Chord => {
                let ch = self.score.harmony.cursor(self.measure_index).expect("chord cursor");
                let nexts = self.ctx.get_chord_candidates(&self.score, ch);
                nexts
                    .iter()
                    .map(|(chord, cost)| {
                        let new_harmony = self.score.harmony.replace_chord(self.measure_index, Some(chord.clone()));
                        let new_score = self.score.replace_harmony(new_harmony);
                        Node::new(
                            Rc::new(new_score),
                            self.ctx.clone(),
                            self.measure_index,
                            None,
                            NodeKind::Harmony,
                            self.n_step,
                            self.cost * POWER + cost,
                            *cost,
                            String::new(),
                        )
                    })
                    .collect()
            }
            Target::Measure { voice_index } => {
                let v = &self.score.voices[voice_index];
                let m = v.cursor(self.measure_index).expect("measure cursor");
                let nexts = m.get_next_steps(&self.score, m);
                nexts
                    .into_iter()
                    .filter_map(|step| {
                        let new_score = if let Some(s) = step.score {
                            s
                        } else {
                            let new_voice = v.replace_measure(m.index(), step.measure);
                            self.score.replace_voice(voice_index, new_voice)
                        };
                        if self
                            .ctx
                            .global_rules
                            .iter()
                            .any(|r| r(&self.ctx, &new_score).is_some())
                        {
                            return None;
                        }
                        Some(Node::new(
                            Rc::new(new_score),
                            self.ctx.clone(),
                            self.measure_index,
                            Some(voice_index),
                            NodeKind::Note,
                            self.n_step + rational_value(step.advanced),
                            self.cost * POWER.powf(rational_value(step.advanced)) + step.cost,
                            step.cost,
                            step.debug,
                        ))
                    })
                    .collect()
            }
        }
    }
}

struct HeapEntry {
    priority: f64,
    seq: u64,
    node: Node,
}

impl PartialEq for HeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.seq == other.seq
    }
}

impl Eq for HeapEntry {}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .priority
            .partial_cmp(&self.priority)
            .unwrap_or(Ordering::Equal)
            .then_with(|| other.seq.cmp(&self.seq))
    }
}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Parent-pointers keyed by a score's cached structural hash (with the full
/// score kept for collision resolution), so lookups avoid re-hashing the whole
/// score on every neighbor.
type ParentMap = HashMap<u64, Vec<(Rc<Score>, Option<Rc<Score>>)>>;

/// Best-first / beam search solver over scores.
pub struct CounterpointSolver {
    ctx: Rc<CounterpointContext>,
    pub batch: usize,
    pub remove_old: usize,
    pub report_interval: usize,
    on_progress: Option<Box<dyn FnMut(CounterpointSolverProgress)>>,
    parents: Option<ParentMap>,
    start: Option<Rc<Score>>,
}

impl CounterpointSolver {
    #[must_use]
    pub fn new(ctx: Rc<CounterpointContext>) -> Self {
        CounterpointSolver {
            ctx,
            batch: 5,
            remove_old: 2,
            report_interval: 1000,
            on_progress: None,
            parents: None,
            start: None,
        }
    }

    pub fn set_reporter(&mut self, on_progress: impl FnMut(CounterpointSolverProgress) + 'static) {
        self.on_progress = Some(Box::new(on_progress));
    }

    #[must_use]
    pub fn parents(&self) -> Option<&ParentMap> {
        self.parents.as_ref()
    }

    #[must_use]
    pub fn start_node(&self) -> Option<&Rc<Score>> {
        self.start.as_ref()
    }

    pub fn a_star(
        &mut self,
        s: &Score,
        strategy: CounterpointSolverRewardStrategy,
    ) -> Option<Rc<Score>> {
        let f = match strategy {
            CounterpointSolverRewardStrategy::Constant { value } => value,
        };

        let mut open: BinaryHeap<HeapEntry> = BinaryHeap::new();
        let mut parents: ParentMap = HashMap::new();
        let start = Node::new(
            Rc::new(s.clone()),
            self.ctx.clone(),
            0,
            None,
            NodeKind::Initial,
            0.0,
            0.0,
            0.0,
            String::new(),
        );
        let start_score = start.score.clone();
        parents
            .entry(start.score_hash)
            .or_default()
            .push((start_score.clone(), None));
        open.push(HeapEntry {
            priority: start.cost - f * start.n_step,
            seq: 0,
            node: start,
        });

        let mut seq = 1u64;
        let mut furthest = 0usize;
        let mut n_node = 0usize;
        let mut progress: usize;

        while !open.is_empty() {
            let mut new_nodes = Vec::new();
            for _ in 0..self.batch {
                let Some(HeapEntry { node: current, .. }) = open.pop() else {
                    break;
                };

                if current.is_goal {
                    self.parents = Some(parents);
                    self.start = Some(start_score);
                    return Some(current.score);
                }

                if current.measure_index < furthest.saturating_sub(self.remove_old) {
                    continue;
                }
                if current.measure_index > furthest || n_node.is_multiple_of(self.report_interval) {
                    if current.measure_index > furthest {
                        furthest = current.measure_index;
                    }

                    progress = current.measure_index;
                    if let Some(cb) = self.on_progress.as_mut() {
                        cb(CounterpointSolverProgress {
                            measure_index: progress,
                            furthest,
                            total_measures: self.ctx.target_measures,
                            iteration: n_node,
                        });
                    }
                }

                let neighbors: Vec<Node> = current
                    .get_neighbors()
                    .into_iter()
                    .filter(|n| {
                        parents
                            .get(&n.score_hash)
                            .is_none_or(|bucket| bucket.iter().all(|(s, _)| s != &n.score))
                    })
                    .collect();
                for n in &neighbors {
                    parents
                        .entry(n.score_hash)
                        .or_default()
                        .push((n.score.clone(), Some(current.score.clone())));
                }
                new_nodes.extend(neighbors);
                n_node += 1;
            }
            for n in new_nodes {
                let priority = n.cost - f * n.n_step;
                open.push(HeapEntry { priority, seq, node: n });
                seq += 1;
            }
        }

        self.parents = Some(parents);
        self.start = Some(start_score);
        None
    }
}
