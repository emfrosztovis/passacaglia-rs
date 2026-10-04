<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref, watch } from 'vue';
import {
  fetchTreeMeta,
  fetchTreeNode,
  type TreeMeta,
  type TreeNodeInfo,
} from '../treeClient';

defineOptions({
  name: 'SearchTree',
});

const emit = defineEmits<{ focusNode: [id: number | null] }>();

type TreeViewNode = TreeNodeInfo & {
  children: number[];
  loaded: boolean;
};

interface Placed {
  id: number;
  x: number;
  y: number;
  depth: number;
  node: TreeViewNode;
}

interface Edge {
  from: number;
  to: number;
  x1: number;
  y1: number;
  x2: number;
  y2: number;
  onPath: boolean;
}

// Reingold-Tilford node (internal nodes only; leaves are fanned separately).
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

const PAD = 24;
const LEVEL_GAP = 75;
const SIBLING_GAP = 100;
const FAN_RADIUS = 50;
const FAN_SPREAD = Math.PI * 0.6;
const RT_GAP = 1;
const MIN_ZOOM = 0.05;
const MAX_ZOOM = 20;

const meta = ref<TreeMeta | null>(null);
const nodes = reactive(new Map<number, TreeViewNode>());
const inflight = new Set<number>();
const error = ref<string | null>(null);
const hoveredId = ref<number | null>(null);
let leaveTimer: ReturnType<typeof setTimeout> | undefined;

const viewport = ref<HTMLElement | null>(null);
const viewW = ref(0);
const viewH = ref(0);
const zoom = ref(1);
const panX = ref(0);
const panY = ref(0);
const detailX = ref(0);
const detailY = ref(0);
let centered = false;

function fmt(x: number | null | undefined, digits = 2): string {
  if (x === null || x === undefined || !Number.isFinite(x)) {
    return '∞';
  }
  return x.toFixed(digits);
}

function infoOf(n: TreeNodeInfo): TreeNodeInfo {
  return {
    id: n.id,
    parent: n.parent,
    measureIndex: n.measureIndex,
    voiceIndex: n.voiceIndex,
    kind: n.kind,
    nStep: n.nStep,
    cost: n.cost,
    thisCost: n.thisCost,
    debug: n.debug,
    isGoal: n.isGoal,
    nExpanded: n.nExpanded,
    isStart: n.isStart,
    onSolutionPath: n.onSolutionPath,
    bestPriority: n.bestPriority,
    maxDepth: n.maxDepth,
    subtreeSize: n.subtreeSize,
  };
}

function upsert(info: TreeNodeInfo) {
  const existing = nodes.get(info.id);
  if (existing) {
    Object.assign(existing, info);
    return;
  }
  nodes.set(info.id, { ...infoOf(info), children: [], loaded: false });
}

async function expand(id: number) {
  if (nodes.get(id)?.loaded || inflight.has(id)) {
    return;
  }
  inflight.add(id);
  try {
    const data = await fetchTreeNode(id);
    const existing = nodes.get(id);
    if (existing) {
      Object.assign(existing, infoOf(data));
    } else {
      nodes.set(id, { ...infoOf(data), children: [], loaded: false });
    }
    const node = nodes.get(id)!;
    node.children = data.children.map((c) => c.id);
    node.loaded = true;
    for (const child of data.children) {
      upsert(child);
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    inflight.delete(id);
  }
}

function toggle(id: number) {
  const node = nodes.get(id);
  if (!node) {
    return;
  }
  if (node.loaded) {
    node.children = [];
    node.loaded = false;
  } else if (node.nExpanded > 0) {
    void expand(id);
  }
}

function onEnter(id: number) {
  if (leaveTimer) {
    clearTimeout(leaveTimer);
    leaveTimer = undefined;
  }
  hoveredId.value = id;
  emit('focusNode', id);
}

function onLeave() {
  hoveredId.value = null;
  if (leaveTimer) {
    clearTimeout(leaveTimer);
  }
  leaveTimer = setTimeout(() => emit('focusNode', null), 80);
}

function onViewMove(e: MouseEvent) {
  const vp = viewport.value;
  if (!vp) {
    return;
  }
  const rect = vp.getBoundingClientRect();
  detailX.value = e.clientX - rect.left;
  detailY.value = e.clientY - rect.top;
}

async function init() {
  try {
    const m = await fetchTreeMeta();
    meta.value = m;
    const toFetch = new Set<number>([m.start, ...m.solutionPath]);
    await Promise.all([...toFetch].map((id) => expand(id)));
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  }
}

// --- Reingold-Tilford layout (Buchheim et al.) -------------------------------

function buildRT(id: number, parent: RTNode | null, depth: number): RTNode {
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
      n.children.push(buildRT(cid, n, depth + 1));
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

// --- layout ----------------------------------------------------------------

const layout = computed(() => {
  const m = meta.value;
  if (!m || !nodes.has(m.start)) {
    return { placed: [], edges: [], width: 0, height: 0 };
  }

  const root = buildRT(m.start, null, 0);
  rtFirstWalk(root);
  rtSecondWalk(root, 0);
  const rootX = root.x;

  const placed: Placed[] = [];

  const place = (n: RTNode) => {
    const x = n.depth * LEVEL_GAP;
    const y = (n.x - rootX) * SIBLING_GAP;
    placed.push({ id: n.id, x, y, depth: n.depth, node: nodes.get(n.id)! });

    for (const c of n.children) {
      place(c);
    }

    const total = n.leafIds.length;
    const angleOne = total <= 1 ? 0 : FAN_SPREAD / (total - 1);
    n.leafIds.forEach((lid, idx) => {
      const view = nodes.get(lid)!;
      let lx: number;
      let ly: number;
      if (view.onSolutionPath && view.isGoal) {
        lx = (n.depth + 1) * LEVEL_GAP;
        ly = y;
      } else {
        const angle = angleOne * idx - angleOne * ((total - 1) / 2);
        lx = x + Math.cos(angle) * FAN_RADIUS;
        ly = y - Math.sin(angle) * FAN_RADIUS;
      }
      placed.push({ id: lid, x: lx, y: ly, depth: n.depth + 1, node: view });
    });
  };

  place(root);

  // Normalize so the bounding box starts at PAD.
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
  const edges: Edge[] = [];
  for (const p of placed) {
    for (const cid of p.node.children) {
      const c = pos.get(cid);
      if (c) {
        edges.push({
          from: p.id,
          to: cid,
          x1: p.x,
          y1: p.y,
          x2: c.x,
          y2: c.y,
          onPath: p.node.onSolutionPath && c.node.onSolutionPath,
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
});

function isExpandable(node: TreeViewNode): boolean {
  return node.nExpanded > 0 && !node.loaded;
}

function fillColor(node: TreeViewNode): string {
  if (node.isStart) {
    return '#2ecc71';
  }
  if (node.isGoal) {
    return '#3498db';
  }
  if (node.nExpanded === 0) {
    return '#e74c3c';
  }
  return '#555';
}

function importanceColor(node: TreeViewNode): string {
  const total = meta.value?.targetMeasures ?? 1;
  const p = Math.max(0, Math.min(1, node.maxDepth / total));
  const hue = 210 - p * 180;
  return `hsl(${hue}, 70%, ${28 + p * 34}%)`;
}

function importanceRadius(node: TreeViewNode): number {
  return 3 + Math.min(7, Math.log2(node.subtreeSize + 1) * 0.9);
}

function pathD(e: Edge): string {
  const mx = (e.x1 + e.x2) / 2;
  return `M ${e.x1} ${e.y1} C ${mx} ${e.y1}, ${mx} ${e.y2}, ${e.x2} ${e.y2}`;
}

const hoveredNode = computed(() =>
  hoveredId.value === null ? undefined : nodes.get(hoveredId.value),
);

const detailRows = computed<[string, string | number][]>(() => {
  const n = hoveredNode.value;
  if (!n) {
    return [];
  }
  return [
    ['kind', n.kind],
    ['measure', n.measureIndex],
    ['voice', n.voiceIndex ?? '—'],
    ['cost', fmt(n.cost)],
    ['thisCost', fmt(n.thisCost)],
    ['nStep', fmt(n.nStep, 1)],
    ['expanded', n.nExpanded],
    ['subtree', n.subtreeSize],
    ['maxDepth', n.maxDepth],
    ['bestPriority', fmt(n.bestPriority)],
    ['debug', n.debug || '—'],
  ];
});

const svgStyle = computed(() => ({
  transform: `translate(${panX.value}px, ${panY.value}px) scale(${zoom.value})`,
  transformOrigin: '0 0',
}));

function onWheel(e: WheelEvent) {
  const vp = viewport.value;
  if (!vp) {
    return;
  }
  const rect = vp.getBoundingClientRect();
  const px = e.clientX - rect.left;
  const py = e.clientY - rect.top;
  const factor = e.deltaY < 0 ? 1.1 : 0.9;
  const newZoom = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, zoom.value * factor));
  const k = newZoom / zoom.value;
  panX.value = px - (px - panX.value) * k;
  panY.value = py - (py - panY.value) * k;
  zoom.value = newZoom;
}

let panning = false;
let startX = 0;
let startY = 0;
let startPanX = 0;
let startPanY = 0;

function onPanStart(e: PointerEvent) {
  panning = true;
  startX = e.clientX;
  startY = e.clientY;
  startPanX = panX.value;
  startPanY = panY.value;
  window.addEventListener('pointermove', onPanMove);
  window.addEventListener('pointerup', onPanEnd);
}

function onPanMove(e: PointerEvent) {
  if (!panning) {
    return;
  }
  panX.value = startPanX + (e.clientX - startX);
  panY.value = startPanY + (e.clientY - startY);
}

function onPanEnd() {
  panning = false;
  window.removeEventListener('pointermove', onPanMove);
  window.removeEventListener('pointerup', onPanEnd);
}

function updateViewSize() {
  const vp = viewport.value;
  if (!vp) {
    return;
  }
  viewW.value = vp.clientWidth;
  viewH.value = vp.clientHeight;
  maybeCenter();
}

function maybeCenter() {
  if (centered || !meta.value || layout.value.placed.length === 0) {
    return;
  }
  if (viewW.value === 0 || viewH.value === 0) {
    return;
  }
  centered = true;
  const l = layout.value;
  const fit = Math.min(viewW.value / l.width, viewH.value / l.height) * 0.95;
  zoom.value = Math.max(MIN_ZOOM, Math.min(1, fit));
  panX.value = (viewW.value - l.width * zoom.value) / 2;
  panY.value = (viewH.value - l.height * zoom.value) / 2;
}

watch(layout, () => maybeCenter());

let observer: ResizeObserver | undefined;

onMounted(() => {
  void init();
  if (viewport.value) {
    observer = new ResizeObserver(updateViewSize);
    observer.observe(viewport.value);
    updateViewSize();
  }
});

onUnmounted(() => {
  if (leaveTimer) {
    clearTimeout(leaveTimer);
  }
  observer?.disconnect();
  window.removeEventListener('pointermove', onPanMove);
  window.removeEventListener('pointerup', onPanEnd);
});
</script>

<template>
  <div class="search-tree">
    <div v-if="error" class="error">search tree error: {{ error }}</div>
    <div v-else-if="!meta" class="error">loading search tree…</div>
    <div v-else class="search-tree-body">
      <div class="legend">
        <span class="lg"><i class="sw start"></i>start</span>
        <span class="lg"><i class="sw goal"></i>goal</span>
        <span class="lg"><i class="sw leaf"></i>dead-end</span>
        <span class="lg"><i class="sw explored"></i>explored</span>
        <span class="lg"><i class="sw branch"></i>collapsed branch (big = explored, warm = deep)</span>
        <span class="lg"><i class="sw path"></i>accepted path</span>
        <span class="hint">click to expand/collapse · drag to pan · wheel to zoom · hover for details</span>
      </div>
      <div
        ref="viewport"
        class="viewport"
        @wheel.prevent="onWheel"
        @pointerdown="onPanStart"
        @mousemove="onViewMove"
      >
        <svg
          :width="layout.width"
          :height="layout.height"
          :viewBox="`0 0 ${layout.width} ${layout.height}`"
          :style="svgStyle"
        >
          <g v-for="e in layout.edges" :key="`e${e.from}-${e.to}`" class="edge">
            <path
              :d="pathD(e)"
              fill="none"
              :stroke="e.onPath ? '#f1c40f' : '#666'"
              :stroke-width="e.onPath ? 1.6 : 0.8"
              :opacity="e.onPath ? 1 : 0.7"
            />
          </g>
          <g
            v-for="p in layout.placed"
            :key="`n${p.id}`"
            class="node"
            :transform="`translate(${p.x}, ${p.y})`"
            :class="{ hovered: hoveredId === p.id }"
            @pointerdown.stop
            @mouseenter="onEnter(p.id)"
            @mouseleave="onLeave"
            @click.stop="toggle(p.id)"
          >
            <template v-if="isExpandable(p.node)">
              <circle
                :r="importanceRadius(p.node)"
                :fill="importanceColor(p.node)"
                :stroke="p.node.onSolutionPath ? '#f1c40f' : 'none'"
                :stroke-width="1.5"
              />
            </template>
            <template v-else>
              <rect
                x="-5"
                y="-5"
                width="10"
                height="10"
                :fill="fillColor(p.node)"
                :stroke="p.node.onSolutionPath ? '#f1c40f' : 'none'"
                :stroke-width="1.5"
              />
            </template>
          </g>
        </svg>
        <div
          v-if="hoveredNode"
          class="detail"
          :style="{ left: `${detailX + 14}px`, top: `${detailY + 14}px` }"
        >
          <div class="detail-title">#{{ hoveredNode.id }} ({{ hoveredNode.kind }})</div>
          <dl>
            <template v-for="[k, v] in detailRows" :key="k">
              <dt>{{ k }}</dt>
              <dd>{{ v }}</dd>
            </template>
          </dl>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.search-tree {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.search-tree-body {
  flex-grow: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.legend {
  display: flex;
  align-items: center;
  gap: 1em;
  padding: 0.35em 0.75em;
  font-size: 0.8em;
  color: #ccc;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  flex-wrap: wrap;
}

.legend .lg {
  display: inline-flex;
  align-items: center;
  gap: 0.3em;
}

.legend .sw {
  display: inline-block;
  width: 10px;
  height: 10px;
  border-radius: 2px;
}

.legend .sw.start { background: #2ecc71; }
.legend .sw.goal { background: #3498db; }
.legend .sw.leaf { background: #e74c3c; }
.legend .sw.explored { background: #555; }
.legend .sw.branch { border-radius: 50%; background: hsl(30, 70%, 50%); }
.legend .sw.path { outline: 1.5px solid #f1c40f; background: #444; }

.legend .hint {
  margin-left: auto;
  color: #888;
}

.viewport {
  flex-grow: 1;
  min-height: 0;
  position: relative;
  overflow: hidden;
  background: #1e1e1e;
  cursor: grab;
}

.viewport:active {
  cursor: grabbing;
}

.error {
  color: #888;
  padding: 1em;
}

.node {
  cursor: pointer;
}

.node.hovered rect,
.node.hovered circle {
  filter: brightness(1.4);
}

.detail {
  position: absolute;
  pointer-events: none;
  background: rgba(30, 30, 30, 0.95);
  border: 1px solid rgba(255, 255, 255, 0.25);
  border-radius: 6px;
  padding: 0.5em 0.7em;
  font-size: 0.75em;
  color: #ddd;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.5);
  max-width: 240px;
  z-index: 5;
}

.detail-title {
  font-weight: 600;
  margin-bottom: 0.3em;
  color: #fff;
}

.detail dl {
  margin: 0;
  display: grid;
  grid-template-columns: auto auto;
  gap: 0 0.75em;
}

.detail dt {
  color: #999;
}

.detail dd {
  margin: 0;
  text-align: right;
  font-variant-numeric: tabular-nums;
}
</style>
