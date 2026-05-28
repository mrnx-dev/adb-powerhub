<script setup lang="ts">
import { ref, nextTick, watch } from "vue";
import { useDeviceStore } from "../stores/device";
import { useNavigationStore } from "../stores/navigation";
import { Terminal, Copy, Check } from "lucide-vue-next";

const store = useDeviceStore();
const navStore = useNavigationStore();
const inputRef = ref<HTMLInputElement | null>(null);
const logContainerRef = ref<HTMLElement | null>(null);
const copied = ref(false);

watch(() => store.logs.length, async () => {
  await nextTick();
  if (logContainerRef.value) {
    logContainerRef.value.scrollTop = logContainerRef.value.scrollHeight;
  }
});

watch(() => navStore.focusTerminalRequested, async (val) => {
  if (val) {
    navStore.clearTerminalFocusRequest();
    await nextTick();
    inputRef.value?.focus();
  }
});

function handleExecute() {
  if (store.commandInput.trim()) {
    store.executeCommand(store.commandInput);
    store.commandInput = "";
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === "Enter") {
    handleExecute();
  }
}

function exportLogs() {
  const content = store.logs.join("\n");
  const blob = new Blob([content], { type: "text/plain" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `adb-powerhub-logs-${Date.now()}.txt`;
  a.click();
  URL.revokeObjectURL(url);
}

async function copyLogs() {
  try {
    const content = store.logs.join("\n");
    await navigator.clipboard.writeText(content);
    copied.value = true;
    setTimeout(() => { copied.value = false; }, 2000);
  } catch {
    copied.value = false;
  }
}
</script>

<template>
  <section class="card-glass p-4 flex-1 min-h-0 flex flex-col">
    <div class="flex items-center justify-between mb-3 shrink-0">
      <div class="flex items-center gap-2">
        <Terminal :size="16" class="text-accent-emerald" />
        <h2 class="font-sans text-xs font-semibold tracking-wider uppercase">Terminal</h2>
      </div>
      <div class="flex gap-2">
        <button @click="copyLogs" :disabled="store.logs.length === 0"
          class="flex items-center gap-1 px-3 py-1 bg-theme-btn border border-theme-secondary rounded-md text-[10px] font-medium hover-accent disabled:opacity-30 disabled:cursor-not-allowed transition-colors">
          <Check v-if="copied" :size="10" class="text-accent-emerald" />
          <Copy v-else :size="10" />
          {{ copied ? 'Copied' : 'Copy' }}
        </button>
        <button @click="store.clearLogs"
          class="px-3 py-1 bg-theme-btn border border-theme-secondary rounded-md text-[10px] font-medium hover-accent">Clear</button>
        <button @click="exportLogs"
          class="px-3 py-1 bg-theme-btn border border-theme-secondary rounded-md text-[10px] font-medium hover-accent">Export</button>
      </div>
    </div>

    <!-- Log Area -->
    <div ref="logContainerRef" class="flex-1 overflow-y-auto mb-3 p-3 font-mono text-xs text-theme-secondary bg-theme-terminal rounded-t-lg border-x border-t border-theme-tertiary select-text">
      <p v-if="store.logs.length === 0" class="mb-1">No logs yet. Click "Auto Connect" to start.</p>
      <p v-for="(log, i) in store.logs" :key="i" class="mb-1 whitespace-pre-wrap break-all terminal-log-line">{{ log }}</p>
      <div v-if="store.logs.length > 0" class="flex items-center">
        <span class="text-accent-emerald font-mono font-bold mr-1">></span>
        <div class="w-2 h-4 bg-accent-emerald animate-pulse"></div>
      </div>
    </div>

    <!-- Input Area -->
    <div class="bg-theme-input backdrop-blur-md border-t border-theme-secondary rounded-b-lg p-2 shrink-0 group focus-within:border-accent-emerald/50">
      <div class="flex items-center gap-3">
        <span class="text-xs font-mono font-bold text-accent-emerald pl-2">$ adb</span>
        <input ref="inputRef" v-model="store.commandInput" @keydown="handleKeydown" type="text"
          placeholder="Enter adb command (e.g., shell dumpsys battery)"
          class="flex-1 bg-transparent border-none text-xs font-mono text-theme-primary placeholder:text-theme-muted focus:ring-0 focus:outline-none py-1 px-0" />
        <button @click="handleExecute" :disabled="!store.connected"
          class="btn-primary text-[10px] font-bold uppercase tracking-wider px-4 py-2 rounded-lg disabled:opacity-50 disabled:cursor-not-allowed">
          Execute
        </button>
      </div>
    </div>
  </section>
</template>
