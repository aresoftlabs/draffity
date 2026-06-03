<script setup lang="ts">
import { computed, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import Dialog from 'primevue/dialog';
import Button from 'primevue/button';
import { useUiStore } from '@/stores/ui';

const STORAGE_KEY = 'draffity.onboarded';

const { t } = useI18n();
const uiStore = useUiStore();

const visible = ref(false);
const step = ref(0);

if (typeof localStorage !== 'undefined' && localStorage.getItem(STORAGE_KEY) !== '1') {
  visible.value = true;
}

const isLastSlide = computed(() => step.value === slides.length - 1);

const slides = [
  {
    icon: 'pi pi-bolt',
    titleKey: 'onboarding.welcomeTitle',
    bodyKey: 'onboarding.welcomeBody',
  },
  {
    icon: 'pi pi-sitemap',
    titleKey: 'onboarding.binderTitle',
    bodyKey: 'onboarding.binderBody',
  },
  {
    icon: 'pi pi-pencil',
    titleKey: 'onboarding.startTitle',
    bodyKey: 'onboarding.startBody',
  },
];

function next() {
  if (step.value < slides.length - 1) {
    step.value += 1;
  } else {
    finish({ startProject: true });
  }
}

function back() {
  if (step.value > 0) step.value -= 1;
}

function finish(opts: { startProject?: boolean } = {}) {
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem(STORAGE_KEY, '1');
  }
  visible.value = false;
  if (opts.startProject) uiStore.requestNewProject();
}
</script>

<template>
  <Dialog
    :visible="visible"
    modal
    :closable="false"
    :show-header="false"
    :style="{ width: '32rem' }"
    @update:visible="(v: boolean) => (visible = v)"
  >
    <div class="flex flex-col items-center text-center gap-4 p-4">
      <i :class="slides[step].icon + ' text-5xl text-primary-500 mt-4'" />
      <h2 class="text-2xl font-display font-bold">
        {{ t(slides[step].titleKey) }}
      </h2>
      <p class="text-sm leading-relaxed opacity-80 max-w-sm">
        {{ t(slides[step].bodyKey) }}
      </p>

      <div class="flex items-center justify-center gap-1 mt-2">
        <span
          v-for="i in slides.length"
          :key="i"
          class="w-2 h-2 rounded-full transition-colors"
          :class="i - 1 === step ? 'bg-primary-500' : 'bg-surface-300 dark:bg-surface-600'"
        />
      </div>
    </div>

    <template #footer>
      <Button
        :label="t('onboarding.skip')"
        text
        severity="secondary"
        :disabled="isLastSlide"
        @click="finish()"
      />
      <span class="flex-1" />
      <Button
        v-if="step > 0"
        :label="t('onboarding.back')"
        icon="pi pi-arrow-left"
        text
        severity="secondary"
        @click="back"
      />
      <Button
        :label="isLastSlide ? t('onboarding.createFirstProject') : t('onboarding.next')"
        :icon="isLastSlide ? 'pi pi-plus' : 'pi pi-arrow-right'"
        icon-pos="right"
        @click="next"
      />
    </template>
  </Dialog>
</template>
