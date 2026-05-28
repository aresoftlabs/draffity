<script setup lang="ts">
import { RouterView } from 'vue-router';
import Toast from 'primevue/toast';
import ConfirmDialog from 'primevue/confirmdialog';
import AppShell from '@/components/AppShell.vue';
import OnboardingDialog from '@/components/OnboardingDialog.vue';
</script>

<template>
  <div
    class="draffity-app flex flex-col min-h-screen bg-surface-50 dark:bg-surface-950 text-surface-900 dark:text-surface-50"
  >
    <AppShell />
    <main class="flex-1 flex flex-col min-h-0">
      <RouterView />
    </main>
    <OnboardingDialog />
    <Toast position="bottom-right" />
    <ConfirmDialog />
  </div>
</template>
