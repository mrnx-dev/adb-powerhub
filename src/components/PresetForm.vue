<script setup lang="ts">
import { ref, watch, computed } from 'vue';
import { usePresetsStore, type Preset } from '../stores/presets';

interface Props {
  preset?: Preset | null;
}

const props = defineProps<Props>();
const emit = defineEmits<{ (e: 'save'): void; (e: 'cancel'): void }>();

const presetsStore = usePresetsStore();

const name = ref('');
const command = ref('');
const errors = ref({ name: '', command: '' });

const isEditMode = computed(() => !!props.preset);
const isValid = computed(() => {
  return (
    name.value.trim().length > 0 &&
    name.value.trim().length <= 50 &&
    command.value.trim().length > 0 &&
    command.value.trim().length <= 500
  );
});

function syncFromPreset() {
  if (props.preset) {
    name.value = props.preset.name;
    command.value = props.preset.command;
  } else {
    name.value = '';
    command.value = '';
  }
  errors.value = { name: '', command: '' };
}

watch(() => props.preset, syncFromPreset, { immediate: true });

function validate(): boolean {
  const n = name.value.trim();
  const c = command.value.trim();
  errors.value.name = '';
  errors.value.command = '';

  if (!n) errors.value.name = 'Name is required';
  else if (n.length > 50) errors.value.name = 'Name must be 50 characters or less';

  if (!c) errors.value.command = 'Command is required';
  else if (c.length > 500) errors.value.command = 'Command must be 500 characters or less';

  return !errors.value.name && !errors.value.command;
}

async function handleSave() {
  if (!validate()) return;

  if (isEditMode.value && props.preset) {
    const error = await presetsStore.updatePreset(props.preset.id, name.value, command.value);
    if (error) {
      errors.value.name = error;
      return;
    }
  } else {
    const error = await presetsStore.addPreset(name.value, command.value);
    if (error) {
      errors.value.name = error;
      return;
    }
  }

  emit('save');
}

function handleCancel() {
  emit('cancel');
}
</script>

<template>
  <div class="flex flex-col gap-2">
    <!-- Name -->
    <div class="flex flex-col gap-0.5">
      <label class="text-[10px] font-medium text-theme-secondary">Name</label>
      <input
        ref="nameInput"
        v-model="name"
        type="text"
        placeholder="e.g., List Packages"
        class="bg-theme-input border border-theme-secondary rounded-md px-2 py-1.5 text-xs text-theme-primary placeholder:text-theme-muted focus:outline-none focus-border-accent transition-colors"
        @keydown.enter="handleSave"
      />
      <span v-if="errors.name" class="text-[10px] text-color-error">{{ errors.name }}</span>
    </div>

    <!-- Command -->
    <div class="flex flex-col gap-0.5">
      <div class="flex items-center justify-between">
        <label class="text-[10px] font-medium text-theme-secondary">Command</label>
        <span class="text-[9px] text-theme-muted">Do not include "adb"</span>
      </div>
      <input
        v-model="command"
        type="text"
        placeholder="e.g., shell pm list packages -3"
        class="bg-theme-input border border-theme-secondary rounded-md px-2 py-1.5 text-xs font-mono text-theme-primary placeholder:text-theme-muted focus:outline-none focus-border-accent transition-colors"
        @keydown.enter="handleSave"
      />
      <span v-if="errors.command" class="text-[10px] text-color-error">{{ errors.command }}</span>
    </div>

    <!-- Actions -->
    <div class="flex items-center justify-end gap-2 mt-1">
      <button
        class="px-3 py-1.5 bg-theme-btn border border-theme-tertiary rounded-md text-[10px] font-medium text-theme-secondary hover-accent transition-colors"
        @click="handleCancel"
      >
        Cancel
      </button>
      <button
        :disabled="!isValid"
        class="btn-primary text-[10px] font-bold uppercase tracking-wider px-4 py-1.5 rounded-lg disabled:opacity-50 disabled:cursor-not-allowed"
        @click="handleSave"
      >
        {{ isEditMode ? 'Update' : 'Save' }}
      </button>
    </div>
  </div>
</template>
