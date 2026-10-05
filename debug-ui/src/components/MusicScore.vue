<template>
  <div class="osmd-component">
    <div ref="container"></div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, useTemplateRef, watch } from 'vue';
import { OpenSheetMusicDisplay as OSMD } from 'opensheetmusicdisplay';

defineOptions({
  name: 'OsmdComponent',
});

const props = defineProps({
  file: {
    type: String,
    required: true,
  },
  autoResize: {
    type: Boolean,
    required: false,
    default: true,
  },
});

const container = useTemplateRef('container');
let osmd: OSMD | null = null;

onMounted(() => {
  if (container.value) {
    osmd = new OSMD(container.value, {
        backend: "canvas",
        drawTitle: false,
        autoResize: props.autoResize
    });
    if (props.file)
        void render();
  }
});

watch(props, () => {
  void render();
});

async function render() {
  if (!osmd) return;
  await osmd.load(props.file);
  osmd.zoom = 0.75;
  osmd.render();
}

</script>

<style lang="css" scoped>
    .osmd-component {
        width: 100vw;
        margin: 0;
        padding: 0;
    }
</style>
