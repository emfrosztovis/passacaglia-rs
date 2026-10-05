<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { NProgress } from 'naive-ui';
import MusicScore from './components/MusicScore.vue';
import SearchTree from './components/SearchTree.vue';
import { subscribe, type ConnectionStatus, type DebugServerClient, type ServerEvent } from './debugServerClient';
import { play, type PlayableScore } from './play';
import { fetchNodeScore } from './treeClient';

const status = ref<ConnectionStatus>('connecting');
const source = ref('');
const blob = ref<string | undefined>(undefined);
const noSolution = ref(false);
const playable = ref<PlayableScore | undefined>(undefined);
const progress = ref({
    progress: 0,
    furthest: 0,
    total: 1,
    iteration: 0,
});
const focusedSource = ref<string | undefined>(undefined);

const displaySource = computed(() => focusedSource.value ?? source.value);

const scoreHeight = ref(320);
let resizing = false;
let resizeStartY = 0;
let resizeStartH = 0;

function onResizeStart(e: PointerEvent) {
    resizing = true;
    resizeStartY = e.clientY;
    resizeStartH = scoreHeight.value;
    window.addEventListener('pointermove', onResizeMove);
    window.addEventListener('pointerup', onResizeEnd);
}

function onResizeMove(e: PointerEvent) {
    if (!resizing) {
        return;
    }
    const dy = e.clientY - resizeStartY;
    scoreHeight.value = Math.max(120, Math.min(window.innerHeight - 240, resizeStartH + dy));
}

function onResizeEnd() {
    resizing = false;
    window.removeEventListener('pointermove', onResizeMove);
    window.removeEventListener('pointerup', onResizeEnd);
}

let es: DebugServerClient | undefined;
let focusTimer: ReturnType<typeof setTimeout> | undefined;

function reset() {
    source.value = '';
    blob.value = undefined;
    noSolution.value = false;
    playable.value = undefined;
    focusedSource.value = undefined;
    progress.value = { progress: 0, furthest: 0, total: 1, iteration: 0 };
}

function onEvent(ev: ServerEvent) {
    switch (ev.type) {
        case 'ok':
            source.value = ev.mxl;
            playable.value = ev.playable;
            blob.value = URL.createObjectURL(
                new Blob([ev.mxl], { type: 'application/vnd.recordare.musicxml+xml' }),
            );
            break;
        case 'no-solution':
            noSolution.value = true;
            break;
        case 'progress':
            progress.value = ev;
            break;
    }
}

function onPlay() {
    if (playable.value) {
        play(playable.value.voices, [74, 74, 74, 74, 53], { tempo: 180, synth: true });
    }
}

function onFocusNode(id: number | null) {
    if (focusTimer) {
        clearTimeout(focusTimer);
        focusTimer = undefined;
    }
    if (id === null) {
        focusedSource.value = undefined;
        return;
    }
    focusTimer = setTimeout(async () => {
        try {
            const { mxl } = await fetchNodeScore(id);
            focusedSource.value = mxl;
        } catch {
            // ignore fetch failures for a transient focus
        }
    }, 150);
}

function onStatus(s: ConnectionStatus) {
    const wasStandby = status.value === 'standby';
    status.value = s;
    if (s === 'open' && wasStandby) {
        reset();
    }
}

onMounted(() => {
    es = subscribe(onEvent, onStatus);
});

onUnmounted(() => {
    es?.close();
    if (focusTimer) {
        clearTimeout(focusTimer);
    }
    window.removeEventListener('pointermove', onResizeMove);
    window.removeEventListener('pointerup', onResizeEnd);
});
</script>

<template>
    <div v-if="status !== 'open'" class="standby">
        <p v-if="status === 'connecting'">connecting to debug server…</p>
        <template v-else>
            <p>standby — debug server not running</p>
            <p class="hint">
                start it with <code>cargo run -p passacaglia-debug-server</code>
            </p>
        </template>
    </div>

    <NProgress v-else-if="!source && !noSolution" type="multiple-circle" :percentage="[
        progress.furthest / progress.total * 100,
        progress.progress / progress.total * 100,
    ]">
        {{ progress.progress }} / {{ progress.total }}
        <br>
        {{ (progress.iteration / 1000).toFixed(0) }}k
    </NProgress>

    <div v-else class="container">
        <div class="controls">
            <button v-if="playable" @click="onPlay">play</button>
            <a v-if="blob" :href="blob" download="result.xml">download</a>
            <span v-if="focusedSource !== undefined" class="hint">showing focused node</span>
        </div>
        <div class="content">
            <div class="score-panel" :style="{ height: `${scoreHeight}px` }">
                <MusicScore v-if="displaySource" :file="displaySource" />
                <p v-else-if="noSolution" class="hint">no solution found</p>
            </div>
            <div class="resizer" @pointerdown="onResizeStart"></div>
            <div class="tree-panel">
                <SearchTree @focus-node="onFocusNode" />
            </div>
        </div>
    </div>
</template>

<style scoped>
    .container {
        display: flex;
        flex-direction: column;
        width: 100%;
        height: 100vh;
    }

    .controls {
        position: absolute;
        left: 0;
        top: 0;
        z-index: 10;
        display: flex;
        align-items: center;
        gap: 0.75em;
    }

    .content {
        flex-grow: 1;
        min-height: 0;
        display: flex;
        flex-direction: column;
    }

    .score-panel {
        min-height: 120px;
        overflow: auto;
        flex-shrink: 0;
    }

    .resizer {
        height: 5px;
        flex-shrink: 0;
        cursor: ns-resize;
        background: rgba(255, 255, 255, 0.1);
        user-select: none;
        touch-action: none;
    }

    .resizer:hover {
        background: rgba(255, 255, 255, 0.25);
    }

    .tree-panel {
        flex-grow: 1;
        min-height: 120px;
    }

    .standby {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.5em;
        padding: 2em;
    }

    .hint {
        color: #888;
    }

    code {
        background: rgba(255, 255, 255, 0.1);
        padding: 0.2em 0.4em;
        border-radius: 4px;
    }
</style>
