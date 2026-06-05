<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { getVersion } from '@tauri-apps/api/app';

// Self-contained badge: reads the installed app version from Tauri. Outside
// Tauri (browser dev / unit tests) getVersion throws and we render nothing.
const version = ref('');
const label = computed(() => (version.value ? `v${version.value}` : ''));

onMounted(async () => {
  try {
    version.value = await getVersion();
  } catch {
    // not running inside Tauri — leave blank
  }
});
</script>

<template>
  <span
    v-if="label"
    class="font-mono tabular-nums opacity-50"
    data-test="app-version"
    :title="label"
  >
    {{ label }}
  </span>
</template>
