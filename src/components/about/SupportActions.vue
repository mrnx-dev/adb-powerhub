<script setup lang="ts">
import { ref } from 'vue';
import { useAboutStore } from '../../stores/about';
import { useToastStore } from '../../stores/toast';
import DebugInfoModal from './DebugInfoModal.vue';

const aboutStore = useAboutStore();
const toastStore = useToastStore();
const showDebugModal = ref(false);

async function handleCheckForUpdates() {
  const result = await aboutStore.checkForUpdates();
  if (result) {
    const type = result.available ? 'success' : 'info';
    toastStore.show(result.message, type, 4000);
  }
}
</script>

<template>
  <div aria-labelledby="actions-label">
    <h2 id="actions-label" class="font-sans text-xs font-semibold tracking-wider uppercase mb-4">
      Support
    </h2>
    <div class="about-actions flex flex-wrap gap-3">
      <button
        type="button"
        class="btn-pressable px-4 py-2 rounded-lg btn-primary text-xs font-semibold"
        @click="showDebugModal = true"
      >
        Copy Debug Info
      </button>
      <button
        type="button"
        class="btn-pressable px-4 py-2 rounded-lg bg-theme-btn border border-theme-tertiary text-sm hover-accent"
        :disabled="aboutStore.checkingForUpdates"
        @click="handleCheckForUpdates"
      >
        <span v-if="aboutStore.checkingForUpdates">Checking...</span>
        <span v-else>Check for Updates</span>
      </button>
    </div>

    <DebugInfoModal v-model:open="showDebugModal" />
  </div>
</template>
