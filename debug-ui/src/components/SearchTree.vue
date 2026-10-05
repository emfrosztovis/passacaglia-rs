<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref, watch } from 'vue';
import {
  fetchTreeMeta,
  fetchTreeNode,
  type TreeMeta,
  type TreeNodeInfo,
} from '../treeClient';
import { layoutTree, type LayoutEdge } from '../treeLayout';

defineOptions({
  name: 'SearchTree',
});

const emit = defineEmits<{ focusNode: [id: number | null] }>();

type TreeViewNode = TreeNodeInfo & {
  children: number[];
  loaded: boolean;
};

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
    maxMeasureIndex: n.maxMeasureIndex,
    subtreeSize: n.subtreeSize,
  };
}

function upsert(info: TreeNodeInfo) {
  const existing = nodes.get(info.id);
  if (existing) {
    Object.assign(existing, info);
    return;
  }
  nodes.set(info.id, { ...info, children: [], loaded: false });
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
      nodes.set(id, { ...data, children: [], loaded: false });
    }
    const node = nodes.get(id)!;
    node.children = data.children.map((c) => c.id);
    node.loaded = true;
    for (const child of data.children) {
      upsert(child);
    }
    const expandable = data.children.filter((x) => x.nExpanded > 0);
    if (expandable.length == 1) {
      await expand(expandable[0].id);
    } else {
      const total = meta.value?.targetMeasures ?? 1;
      const threshold = (meta.value?.nodeCount ?? Infinity) * 0.01;
      const bigChildren = data.children.filter(
        (x) => x.maxMeasureIndex == total || x.subtreeSize > threshold);
      for (const bigChild of bigChildren) {
        await expand(bigChild.id);
      }
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

const layout = computed(() => {
  const m = meta.value;
  if (!m || !nodes.has(m.start)) {
    return { placed: [], edges: [], width: 0, height: 0 };
  }
  const l = layoutTree(nodes, m.start);
  return {
    placed: l.placed.map((p) => ({ ...p, node: nodes.get(p.id)! })),
    edges: l.edges,
    width: l.width,
    height: l.height,
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
  return importanceColor(node);
}

function importanceColor(node: TreeViewNode): string {
  const total = meta.value?.targetMeasures ?? 1;
  const totalSize = meta.value?.nodeCount ?? 1;
  const p = Math.max(0, Math.min(1, node.maxMeasureIndex / total));
  const hue = 200 - p * 180;
  const sat = 10 + Math.min(80, node.subtreeSize / totalSize * 1000);
  return `hsl(${hue}, ${sat}%, ${30 + p * 30}%)`;
}

function importanceRadius(node: TreeViewNode): number {
  const totalSize = meta.value?.nodeCount ?? 1;
  return 1 + Math.min(7, node.subtreeSize / totalSize * 100 * 5);
}

function importanceSideLength(node: TreeViewNode): number {
  if (node.isGoal) return 10;
  if (node.nExpanded === 0) return 2;
  return 10 + Math.min(10, Math.log2(node.subtreeSize + 1) * 0.5);
}

function pathD(e: LayoutEdge): string {
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
    ['maxMeasureIndex', n.maxMeasureIndex],
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
  const factor = e.deltaY < 0 ? 1.03 : 0.97;
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
                :x="-importanceSideLength(p.node) / 2"
                :y="-importanceSideLength(p.node) / 2"
                :width="importanceSideLength(p.node)"
                :height="importanceSideLength(p.node)"
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
  color: #111;
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
