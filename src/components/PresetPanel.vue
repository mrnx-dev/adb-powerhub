<script setup lang="ts">
import { ref } from 'vue';
import { useDeviceStore } from '../stores/device';
import { usePresetsStore, type Preset } from '../stores/presets';
import PresetForm from './PresetForm.vue';
import { ask } from '@tauri-apps/plugin-dialog';
import { Pencil, Trash2 } from '@lucide/vue';

const deviceStore = useDeviceStore();
const presetsStore = usePresetsStore();

const showForm = ref(false);
const editingPreset = ref<Preset | null>(null);

function startCreate() {
  editingPreset.value = null;
  showForm.value = true;
}

function startEdit(preset: Preset) {
  editingPreset.value = preset;
  showForm.value = true;
}

function closeForm() {
  showForm.value = false;
  editingPreset.value = null;
}

async function confirmDelete(preset: Preset) {
  const confirmed = await ask(`Delete preset "${preset.name}"?`, {
    title: 'Delete Preset',
    kind: 'warning',
    okLabel: 'Delete',
    cancelLabel: 'Cancel',
  });
  if (confirmed) {
    await presetsStore.deletePreset(preset.id);
  }
}
</script>

<template>
  <div class="mb-2">
    <!-- Horizontal Chip Bar -->
    <div class="flex items-center gap-2">
      <!-- Label -->
      <div class="shrink-0 flex items-center gap-1.5">
        <span class="text-[10px] font-semibold tracking-wider uppercase text-theme-secondary"
          >Presets</span
        >
        <span
          v-if="presetsStore.presets.length > 0"
          class="hidden sm:inline text-[9px] text-theme-muted bg-theme-btn border border-theme-tertiary rounded px-1 py-0.5 font-mono"
          title="Keyboard shortcuts for quick-run"
        >
          Alt+1..{{ Math.min(presetsStore.presets.length, 9) }}
        </span>
      </div>

      <!-- Scrollable Chip Area -->
      <div class="flex-1 flex items-center gap-1.5 overflow-x-auto scrollbar-thin">
        <!-- Empty State Inline -->
        <span v-if="presetsStore.presets.length === 0" class="text-[10px] text-theme-muted italic">
          No presets
        </span>

        <!-- Chips -->
        <div
          v-for="(preset, index) in presetsStore.presets"
          :key="preset.id"
          class="group flex items-center gap-1 shrink-0 px-2 py-1 bg-theme-btn/60 border border-theme-secondary/60 rounded-md hover:border-accent-emerald/60 transition-all"
          :class="{ 'opacity-50': !deviceStore.connected }"
        >
          <!-- Run area -->
          <div
            class="flex items-center gap-1 cursor-pointer"
            :title="
              deviceStore.connected
                ? `Run: ${preset.command} (Alt+${index + 1})`
                : 'Connect a device to run'
            "
            @click="deviceStore.connected && presetsStore.runPreset(preset)"
          >
            <span
              class="shrink-0 w-4 h-4 flex items-center justify-center rounded bg-theme-btn border border-theme-tertiary text-[9px] font-mono text-theme-muted"
            >
              {{ index + 1 }}
            </span>
            <span class="text-[10px] font-medium text-theme-primary truncate max-w-[100px]">{{
              preset.name
            }}</span>
          </div>

          <!-- Hover actions -->
          <div class="hidden group-hover:flex items-center gap-0.5 ml-0.5">
            <button
              class="p-0.5 rounded hover:bg-theme-btn transition-colors"
              title="Edit preset"
              @click.stop="startEdit(preset)"
            >
              <Pencil :size="9" class="text-theme-muted hover:text-accent-emerald" />
            </button>
            <button
              class="p-0.5 rounded hover:bg-theme-btn transition-colors"
              title="Delete preset"
              @click.stop="confirmDelete(preset)"
            >
              <Trash2 :size="9" class="text-theme-muted hover:text-red-400" />
            </button>
          </div>
        </div>

        <!-- Add Button Chip -->
        <button
          class="shrink-0 flex items-center justify-center w-6 h-6 bg-theme-btn/60 border border-theme-secondary/60 rounded-md text-[10px] font-bold text-accent-emerald hover:border-accent-emerald/60 transition-colors"
          title="Add new preset"
          @click="startCreate"
        >
          +
        </button>
      </div>
    </div>

    <!-- Dialog Modal Overlay -->
    <div
      v-if="showForm"
      class="fixed inset-0 z-50 flex items-center justify-center rounded-lg bg-black/40 backdrop-blur-md"
      @click.self="closeForm"
    >
      <div
        class="w-full max-w-sm mx-4 bg-theme-card border border-theme-secondary rounded-lg shadow-lg p-4"
      >
        <div class="flex items-center justify-between mb-3">
          <h3 class="text-xs font-semibold text-theme-primary">
            {{ editingPreset ? 'Edit Preset' : 'New Preset' }}
          </h3>
          <button
            class="p-1 rounded hover:bg-theme-btn transition-colors text-theme-muted"
            title="Close"
            @click="closeForm"
          >
            <svg
              class="w-3.5 h-3.5"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </button>
        </div>
        <PresetForm :preset="editingPreset" @save="closeForm" @cancel="closeForm" />
      </div>
    </div>
  </div>
</template>
