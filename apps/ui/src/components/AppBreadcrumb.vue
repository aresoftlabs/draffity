<script setup lang="ts">
import { useI18n } from 'vue-i18n';

/** Migas de pan presentacionales del contexto actual. Sin lógica de negocio:
 *  el AppTopBar le pasa el nombre del proyecto y el título del documento. */
defineProps<{
  projectName?: string | null;
  docTitle?: string | null;
}>();

const { t } = useI18n();
</script>

<template>
  <nav
    v-if="projectName"
    class="flex items-center gap-2 text-sm min-w-0"
    :aria-label="t('topBar.breadcrumb')"
  >
    <span class="font-medium text-surface-700 dark:text-surface-200 truncate">
      {{ projectName }}
    </span>
    <template v-if="docTitle">
      <i class="pi pi-angle-right text-xs opacity-40 shrink-0" aria-hidden="true" />
      <span data-test="crumb-doc" class="text-surface-500 dark:text-surface-400 truncate">
        {{ docTitle }}
      </span>
    </template>
  </nav>
</template>
