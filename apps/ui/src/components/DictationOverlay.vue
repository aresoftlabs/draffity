<script setup lang="ts">
import VoiceRecorderControl from './VoiceRecorderControl.vue';
import type { DictationPhase } from '@/composables/useDictation';

const props = defineProps<{
  phase: DictationPhase;
  waveform: Uint8Array;
  elapsedMs: number;
  isSilent: boolean;
  progress?: number | null;
}>();
defineEmits<{ stop: []; cancel: [] }>();
</script>

<template>
  <div v-if="props.phase !== 'idle'" class="fixed bottom-6 left-1/2 -translate-x-1/2 z-50">
    <VoiceRecorderControl
      :state="props.phase === 'recording' ? 'recording' : 'transcribing'"
      :waveform="props.waveform"
      :elapsed-ms="props.elapsedMs"
      :is-silent="props.isSilent"
      :progress="props.progress ?? null"
      @stop="$emit('stop')"
      @cancel="$emit('cancel')"
    />
  </div>
</template>
