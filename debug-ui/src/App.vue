<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';
import { NProgress } from 'naive-ui';
import MusicScore from './components/MusicScore.vue';
import { subscribe, type ConnectionStatus, type DebugServerClient, type ServerEvent } from './debugServerClient';
import { play, type PlayableScore } from './play';

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

let es: DebugServerClient | undefined;

function reset() {
    source.value = '';
    blob.value = undefined;
    noSolution.value = false;
    playable.value = undefined;
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

    <div v-else-if="noSolution" class="standby">
        <p>no solution found</p>
    </div>

    <NProgress v-else-if="!source" type="multiple-circle" :percentage="[
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
        </div>
        <div class="content">
            <div>
                <MusicScore v-if="source" :file="source" />
            </div>
        </div>
    </div>
</template>

<style scoped>
    .container {
        display: flex;
        width: 100%;
        height: 100%;
    }

    .controls {
        position: absolute;
        left: 0;
        top: 0;
    }

    .content {
        flex-grow: 1;
        min-width: 500px;
        height: 100%;

        & > div {
            position: absolute;
            left: 0;
            top: 2em;
            width: 100%;
            height: 100%;
        }
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
