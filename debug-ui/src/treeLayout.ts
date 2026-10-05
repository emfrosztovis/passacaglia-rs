// Reingold–Tilford (Buchheim et al.) tree layout plus a side fan for leaves.
//
// The layout operates on a lazily-loaded tree. Internal nodes (those that have
// been expanded and have children) are laid out first with the classic
// Reingold–Tilford algorithm; only then are the unexpanded children positioned,
// so their fan can be steered around the exact directions of each node's
// expanded children. Non-dead-end unexpanded branches get a proper fan on the
// sides, while dead-ends are grouped into a tight, small fan at the back.

export interface LayoutNode {
  id: number;
  children: number[];
  isGoal: boolean;
  onSolutionPath: boolean;
  nExpanded: number;
}

export interface PlacedNode {
  id: number;
  x: number;
  y: number;
  depth: number;
}

export interface LayoutEdge {
  from: number;
  to: number;
  x1: number;
  y1: number;
  x2: number;
  y2: number;
  onPath: boolean;
}

export interface TreeLayout {
  placed: PlacedNode[];
  edges: LayoutEdge[];
  width: number;
  height: number;
}

const PAD = 24;
const LEVEL_GAP = 75;
const SIBLING_GAP = 75;
// Fan geometry, in degrees. `FAN_STEP` is the angular gap between adjacent
// fanned branches; `FAN_MAX_SPREAD` caps the spread on one side; `SIDE_MARGIN`
// keeps a small clearance around the cone occupied by the expanded children.
const FAN_RADIUS = 30;
const FAN_STEP = 25;
const FAN_MAX_SPREAD = 150;
const SIDE_MARGIN_DEG = 10;
// Dead-end leaves are uninteresting: a tight fan at the back with a small
// radius and spacing.
const DEAD_FAN_RADIUS = 24;
const DEAD_FAN_STEP = 12;
const DEAD_FAN_MAX_SPREAD = 60;
const DEG = Math.PI / 180;
const RT_GAP = 1;

interface RTNode {
  id: number;
  children: RTNode[];
  leafIds: number[];
  parent: RTNode | null;
  i: number;
  z: number;
  m: number;
  s: number;
  c: number;
  t?: RTNode;
  a?: RTNode;
  x: number;
  depth: number;
}

function buildRT(nodes: ReadonlyMap<number, LayoutNode>, id: number, parent: RTNode | null, depth: number): RTNode {
  const view = nodes.get(id)!;
  const n: RTNode = {
    id,
    children: [],
    leafIds: [],
    parent,
    i: 0,
    z: 0,
    m: 0,
    s: 0,
    c: 0,
    t: undefined,
    a: undefined,
    x: 0,
    depth,
  };
  for (const cid of view.children) {
    const child = nodes.get(cid)!;
    if (child.children.length > 0) {
      n.children.push(buildRT(nodes, cid, n, depth + 1));
    } else {
      n.leafIds.push(cid);
    }
  }
  n.children.forEach((c, idx) => {
    c.i = idx;
  });
  return n;
}

function rtFirstWalk(v: RTNode) {
  for (const c of v.children) {
    rtFirstWalk(c);
  }
  const siblings = v.parent ? v.parent.children : null;
  const w = v.i > 0 && siblings ? siblings[v.i - 1] : null;
  if (v.children.length > 0) {
    rtExecuteShifts(v);
    const mid = (v.children[0].z + v.children[v.children.length - 1].z) / 2;
    if (w) {
      v.z = w.z + RT_GAP;
      v.m = v.z - mid;
    } else {
      v.z = mid;
    }
  } else if (w) {
    v.z = w.z + RT_GAP;
  }
  if (v.parent) {
    v.parent.a = rtApportion(v, w, v.parent.a ?? v.parent.children[0]);
  }
}

function rtSecondWalk(v: RTNode, m: number) {
  v.x = v.z + m;
  v.m += m;
  for (const c of v.children) {
    rtSecondWalk(c, v.m);
  }
}

function rtApportion(v: RTNode, w: RTNode | null, ancestor: RTNode): RTNode {
  if (w) {
    let vip = v;
    let vop = v;
    let vim = w;
    let vom = v.parent!.children[0];
    let sip = v.m;
    let sop = v.m;
    let sim = w.m;
    let som = vom.m;
    for (;;) {
      vim = rtNextRight(vim)!;
      vip = rtNextLeft(vip)!;
      if (!vim || !vip) {
        break;
      }
      vom = rtNextLeft(vom)!;
      vop = rtNextRight(vop)!;
      vop.a = v;
      const shift = vim.z + sim - (vip.z + sip) + RT_GAP;
      if (shift > 0) {
        rtMoveSubtree(rtNextAncestor(vim, v, ancestor), v, shift);
        sip += shift;
        sop += shift;
      }
      sim += vim.m;
      sip += vip.m;
      som += vom.m;
      sop += vop.m;
    }
    if (vim && !rtNextRight(vop)) {
      vop.t = vim;
      vop.m += sim - sop;
    }
    if (vip && !rtNextLeft(vom)) {
      vom.t = vip;
      vom.m += sip - som;
      ancestor = v;
    }
  }
  return ancestor;
}

function rtMoveSubtree(wm: RTNode, wp: RTNode, shift: number) {
  const change = shift / (wp.i - wm.i);
  wp.c -= change;
  wp.s += shift;
  wm.c += change;
  wp.z += shift;
  wp.m += shift;
}

function rtExecuteShifts(v: RTNode) {
  let shift = 0;
  let change = 0;
  for (let i = v.children.length - 1; i >= 0; i--) {
    const w = v.children[i];
    w.z += shift;
    w.m += shift;
    shift += w.s + (change += w.c);
  }
}

function rtNextAncestor(vim: RTNode, v: RTNode, ancestor: RTNode): RTNode {
  return vim.a && vim.a.parent === v.parent ? vim.a : ancestor;
}

function rtNextLeft(v: RTNode): RTNode | undefined {
  return v.children.length > 0 ? v.children[0] : v.t;
}

function rtNextRight(v: RTNode): RTNode | undefined {
  return v.children.length > 0 ? v.children[v.children.length - 1] : v.t;
}

// Angle (radians) from `from` to `to` in the position convention used by the
// fan (`θ = 0` points forward/right, `π/2` up, `-π/2` down, `π` back/left).
function angleTo(from: { x: number; y: number }, to: { x: number; y: number }): number {
  return Math.atan2(from.y - to.y, to.x - from.x);
}

// Evenly distribute `count` angles over [start, end], spaced by `FAN_STEP` and
// capped so a small fan stays compact.
function distribute(count: number, start: number, end: number): number[] {
  if (count === 0) {
    return [];
  }
  const range = end - start;
  const step = FAN_STEP * DEG;
  const spread = count <= 1 ? 0 : Math.min(range, FAN_MAX_SPREAD * DEG, (count - 1) * step);
  const center = (start + end) / 2;
  const result: number[] = [];
  for (let j = 0; j < count; j++) {
    const t = count <= 1 ? 0 : j / (count - 1) - 0.5;
    result.push(center + t * spread);
  }
  return result;
}

// Fan the interesting (non-dead-end) unexpanded branches into the free side
// space, steering clear of the forward cone `[minDir, maxDir]` occupied by the
// expanded children's edges.
function fanSides(count: number, minDir: number, maxDir: number): number[] {
  if (count === 0) {
    return [];
  }
  const margin = SIDE_MARGIN_DEG * DEG;
  const lowerCount = Math.floor(count / 2);
  const upperCount = count - lowerCount;
  const lower = distribute(lowerCount, -Math.PI / 2, Math.max(-Math.PI / 2, minDir - margin));
  const upper = distribute(upperCount, Math.min(Math.PI / 2, maxDir + margin), Math.PI / 2);
  return [...lower, ...upper];
}

// Dead-end leaves fan tightly around the back direction (`π`), out of the way.
function fanDead(count: number): number[] {
  if (count === 0) {
    return [];
  }
  const spread = count <= 1 ? 0 : Math.min(DEAD_FAN_MAX_SPREAD * DEG, (count - 1) * DEAD_FAN_STEP * DEG);
  const result: number[] = [];
  for (let j = 0; j < count; j++) {
    const t = count <= 1 ? 0 : j / (count - 1) - 0.5;
    result.push(Math.PI + t * spread);
  }
  return result;
}

export function layoutTree(
  nodes: ReadonlyMap<number, LayoutNode>,
  startId: number,
): TreeLayout {
  const root = buildRT(nodes, startId, null, 0);
  rtFirstWalk(root);
  rtSecondWalk(root, 0);
  const rootX = root.x;

  const placed: PlacedNode[] = [];
  const positions = new Map<number, PlacedNode>();

  // Pass 1: place the expanded (internal) nodes.
  const placeInternal = (n: RTNode) => {
    const p = {
      id: n.id,
      x: n.depth * LEVEL_GAP,
      y: (n.x - rootX) * SIBLING_GAP,
      depth: n.depth,
    };
    placed.push(p);
    positions.set(n.id, p);
    for (const c of n.children) {
      placeInternal(c);
    }
  };
  placeInternal(root);

  // Pass 2: place the unexpanded children, now that the expanded children's
  // positions are known.
  const placeLeaves = (n: RTNode) => {
    const p = positions.get(n.id)!;

    const goalId = n.leafIds.find((id) => {
      const v = nodes.get(id)!;
      return v.onSolutionPath && v.isGoal;
    });
    const others = n.leafIds.filter((id) => id !== goalId);

    const branches: number[] = [];
    const dead: number[] = [];
    for (const id of others) {
      const node = nodes.get(id)!;
      (node.nExpanded === 0 && !node.isGoal ? dead : branches).push(id);
    }

    if (goalId !== undefined) {
      placed.push({ id: goalId, x: (n.depth + 1) * LEVEL_GAP, y: p.y, depth: n.depth + 1 });
    }

    const dirs = n.children.map((c) => angleTo(p, positions.get(c.id)!));
    const minDir = dirs.length > 0 ? Math.min(...dirs) : 0;
    const maxDir = dirs.length > 0 ? Math.max(...dirs) : 0;

    const branchAngles = fanSides(branches.length, minDir, maxDir);
    branches.forEach((id, idx) => {
      const a = branchAngles[idx];
      placed.push({
        id,
        x: p.x + Math.cos(a) * FAN_RADIUS,
        y: p.y - Math.sin(a) * FAN_RADIUS,
        depth: n.depth + 1,
      });
    });

    const deadAngles = fanDead(dead.length);
    dead.forEach((id, idx) => {
      const a = deadAngles[idx];
      placed.push({
        id,
        x: p.x + Math.cos(a) * DEAD_FAN_RADIUS,
        y: p.y - Math.sin(a) * DEAD_FAN_RADIUS,
        depth: n.depth + 1,
      });
    });

    for (const c of n.children) {
      placeLeaves(c);
    }
  };
  placeLeaves(root);

  let minX = Number.POSITIVE_INFINITY;
  let minY = Number.POSITIVE_INFINITY;
  let maxX = Number.NEGATIVE_INFINITY;
  let maxY = Number.NEGATIVE_INFINITY;
  for (const p of placed) {
    minX = Math.min(minX, p.x);
    minY = Math.min(minY, p.y);
    maxX = Math.max(maxX, p.x);
    maxY = Math.max(maxY, p.y);
  }
  const shiftX = PAD - minX;
  const shiftY = PAD - minY;
  for (const p of placed) {
    p.x += shiftX;
    p.y += shiftY;
  }

  const pos = new Map(placed.map((p) => [p.id, p] as const));
  const edges: LayoutEdge[] = [];
  for (const p of placed) {
    const view = nodes.get(p.id)!;
    for (const cid of view.children) {
      const c = pos.get(cid);
      if (c) {
        edges.push({
          from: p.id, to: cid,
          x1: p.x, y1: p.y,
          x2: c.x, y2: c.y,
          onPath: view.onSolutionPath && nodes.get(cid)!.onSolutionPath,
        });
      }
    }
  }

  return {
    placed,
    edges,
    width: maxX + shiftX + PAD,
    height: maxY + shiftY + PAD,
  };
}
