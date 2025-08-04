
<template>
  <div ref="excalidrawContainer" style="height: 100%; width: 100%;"></div>
</template>

<script setup>
import { onMounted, onUnmounted, ref } from 'vue';
import { createRoot } from 'react-dom/client';
import { createElement } from 'react';
import { Excalidraw } from '@excalidraw/excalidraw';

const props = defineProps({
  initialData: {
    type: Object,
    default: () => ({ elements: [], appState: {} }),
  },
});

const emit = defineEmits(['change']);

const excalidrawContainer = ref(null);
let root = null; // To store the React root

const handleChange = (elements, appState, files) => {
  emit('change', { elements, appState, files });
};

onMounted(() => {
  if (excalidrawContainer.value) {
    root = createRoot(excalidrawContainer.value);
    
    const excalidrawElement = createElement(Excalidraw, {
      initialData: props.initialData,
      onChange: handleChange,
    });

    root.render(excalidrawElement);
  }
});

onUnmounted(() => {
  if (root) {
    root.unmount();
  }
});
</script>
