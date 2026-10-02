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
        osmd.load(props.file).then(() => osmd!.render());
  }
});

watch(props, (x) => {
    if (osmd) {
        osmd.load(x.file).then(() => osmd!.render());
    }
})
</script>

<style lang="css" scoped>
    .osmd-component {
        width: 100vw;
        margin: 0;
        padding: 0;
    }
</style>
