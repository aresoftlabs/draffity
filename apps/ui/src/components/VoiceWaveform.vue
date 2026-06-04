<script setup lang="ts">
import { onMounted, ref, watch } from 'vue';

const props = withDefaults(
  defineProps<{ data: Uint8Array; width?: number; height?: number; color?: string }>(),
  { width: 120, height: 24, color: '#5fa8e0' },
);

const canvas = ref<HTMLCanvasElement | null>(null);

function draw() {
  const el = canvas.value;
  if (!el) return;
  const ctx = el.getContext('2d');
  if (!ctx) return;
  const w = el.width;
  const h = el.height;
  ctx.clearRect(0, 0, w, h);
  const n = props.data.length;
  if (n === 0) return;
  ctx.beginPath();
  ctx.lineWidth = 2;
  ctx.strokeStyle = props.color;
  for (let i = 0; i < n; i++) {
    const x = (i / (n - 1)) * w;
    const y = (props.data[i] / 255) * h;
    if (i === 0) ctx.moveTo(x, y);
    else ctx.lineTo(x, y);
  }
  ctx.stroke();
}

onMounted(draw);
watch(() => props.data, draw);
</script>

<template>
  <canvas ref="canvas" :width="width" :height="height" aria-hidden="true" class="block" />
</template>
