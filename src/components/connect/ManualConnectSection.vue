<script setup lang="ts">
import { ref } from 'vue';
import { Terminal, ChevronRight } from '@lucide/vue';

const props = defineProps<{
  connecting: boolean;
  expanded: boolean;
}>();

const emit = defineEmits<{
  connect: [ip: string, port: number];
  'update:expanded': [value: boolean];
}>();

// Local buffer — prevents store pollution from abandoned edits (blueprint §2)
const localIp = ref('');
const localPort = ref(5555);

function handleConnect() {
  if (localIp.value.trim()) {
    emit('connect', localIp.value.trim(), localPort.value);
  }
}

function handleClear() {
  localIp.value = '';
  localPort.value = 5555;
}

function toggle() {
  emit('update:expanded', !props.expanded);
}
</script>

<template>
  <div>
    <!-- Collapsible header (FR-4 AC1) -->
    <button
      class="w-full flex items-center justify-between py-2 px-1 text-xs font-medium text-theme-secondary hover:text-theme-primary hover:bg-theme-hover/50 rounded-lg transition-colors"
      :aria-expanded="expanded"
      aria-controls="manual-content"
      @click="toggle"
    >
      <span>Connect via IP address</span>
      <ChevronRight
        :size="14"
        class="transition-transform duration-200"
        :class="{ 'rotate-90': expanded }"
      />
    </button>

    <!-- Collapsible content (FR-4 AC2) -->
    <Transition name="collapse">
      <div
        v-if="expanded"
        id="manual-content"
        role="region"
        aria-labelledby="manual-header"
        class="space-y-4 pt-2"
      >
        <div class="space-y-3">
          <div>
            <label
              class="text-[10px] text-theme-muted uppercase tracking-wider font-semibold mb-1.5 block"
              >IP Address</label
            >
            <input
              v-model="localIp"
              type="text"
              placeholder="e.g., 192.168.1.5"
              aria-label="IP address"
              class="w-full input-terminal py-2 px-3 text-xs text-theme-primary placeholder:text-theme-muted"
              @keyup.enter="handleConnect"
            />
          </div>
          <div>
            <label
              class="text-[10px] text-theme-muted uppercase tracking-wider font-semibold mb-1.5 block"
              >Port</label
            >
            <input
              v-model.number="localPort"
              type="number"
              placeholder="5555"
              aria-label="Port number"
              class="w-full input-terminal py-2 px-3 text-xs text-theme-primary placeholder:text-theme-muted"
              @keyup.enter="handleConnect"
            />
          </div>
        </div>

        <div class="flex gap-2">
          <button
            :disabled="connecting || !localIp.trim()"
            class="flex-1 btn-primary py-2.5 rounded-lg text-xs font-semibold disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-1.5"
            @click="handleConnect"
          >
            <Terminal :size="13" />
            {{ connecting ? 'Connecting...' : 'Connect' }}
          </button>
          <button
            class="bg-theme-btn border border-theme-tertiary text-theme-secondary py-2.5 px-3 rounded-lg text-xs hover:text-theme-primary hover:bg-theme-hover transition-colors"
            @click="handleClear"
          >
            Clear
          </button>
        </div>
      </div>
    </Transition>
  </div>
</template>
