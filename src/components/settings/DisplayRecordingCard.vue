<script setup lang="ts">
import { useSettingsStore } from "../../stores/settings";
import { MonitorPlay } from "lucide-vue-next";

const store = useSettingsStore();

const qualityPresets: { key: "low" | "medium" | "high" | "custom"; label: string; desc: string }[] = [
  { key: "low", label: "Low", desc: "720p / 2Mbps" },
  { key: "medium", label: "Medium", desc: "1080p / 4Mbps" },
  { key: "high", label: "High", desc: "Native / 8Mbps" },
  { key: "custom", label: "Custom", desc: "Set your own" },
];

const formatOptions: { key: "mp4" | "mkv"; label: string }[] = [
  { key: "mp4", label: "MP4" },
  { key: "mkv", label: "MKV" },
];
</script>

<template>
  <section class="card-glass border border-card-border rounded-2xl p-4">
    <div class="flex items-center gap-2 mb-4">
      <MonitorPlay :size="16" class="text-accent-emerald" />
      <h2 class="text-xs font-bold uppercase tracking-widest">Display & Recording</h2>
    </div>

    <!-- Video Quality -->
    <div class="mb-6">
      <label class="text-sm block mb-3">Video Quality</label>
      <div class="grid grid-cols-2 sm:grid-cols-4 gap-2">
        <button v-for="preset in qualityPresets" :key="preset.key"
          @click="store.setVideoQuality(preset.key)"
          class="flex flex-col items-center px-3 py-3 rounded-xl border transition-all"
          :class="store.videoQuality === preset.key
            ? 'bg-accent-emerald/10 border-accent-emerald/50 text-accent-emerald'
            : 'bg-white/5 border-white/5 text-gray-300 hover:border-accent-emerald/50 hover:bg-white/10'">
          <span class="text-xs font-semibold">{{ preset.label }}</span>
          <span class="text-[10px] opacity-60">{{ preset.desc }}</span>
        </button>
      </div>

      <!-- Custom settings -->
      <div v-if="store.videoQuality === 'custom'" class="mt-4 bg-black/30 border border-white/5 rounded-xl p-4 space-y-4">
        <div>
          <label class="text-xs font-medium text-gray-400 mb-2 block">Bit Rate (Mbps)</label>
          <input v-model.number="store.customBitRate" @change="store.setCustomBitRate(store.customBitRate)" type="number" min="1" max="100"
            class="w-full bg-black/40 border border-white/10 rounded-lg py-2 px-4 text-sm focus:outline-none focus:border-accent-emerald/50" />
        </div>
        <div>
          <label class="text-xs font-medium text-gray-400 mb-2 block">Max Resolution (px)</label>
          <input v-model.number="store.customMaxSize" @change="store.setCustomMaxSize(store.customMaxSize)" type="number" min="0" max="7680"
            class="w-full bg-black/40 border border-white/10 rounded-lg py-2 px-4 text-sm focus:outline-none focus:border-accent-emerald/50" />
          <p class="text-[10px] text-gray-500 mt-1">0 = unlimited (native resolution)</p>
        </div>
      </div>
    </div>

    <!-- Recording Format -->
    <div class="mb-6">
      <label class="text-sm block mb-3">Recording Format</label>
      <div class="grid grid-cols-2 gap-2">
        <button v-for="fmt in formatOptions" :key="fmt.key"
          @click="store.setRecordingFormat(fmt.key)"
          class="flex items-center justify-center px-4 py-3 rounded-xl border transition-all text-sm font-semibold"
          :class="store.recordingFormat === fmt.key
            ? 'bg-accent-emerald/10 border-accent-emerald/50 text-accent-emerald'
            : 'bg-white/5 border-white/5 text-gray-300 hover:border-accent-emerald/50 hover:bg-white/10'">
          {{ fmt.label }}
        </button>
      </div>
      <p v-if="store.recordingFormat === 'mp4'" class="text-xs text-yellow-500 mt-2">
        MP4 files may become corrupted if recording is interrupted. MKV is recommended for reliability.
      </p>
    </div>

    <!-- Screenshot Save Dir -->
    <div class="mb-4">
      <label class="text-xs font-medium text-gray-400 mb-2 block">Screenshot Save Location</label>
      <div class="flex items-center gap-3">
        <input v-model="store.screenshotSaveDir" @blur="store.saveSetting('screenshotSaveDir', store.screenshotSaveDir)" type="text"
          placeholder="Default: ~/Pictures/adb-powerhub"
          class="flex-1 bg-black/40 border border-white/10 rounded-lg py-2 px-4 text-sm focus:outline-none focus:border-accent-emerald/50" />
        <button @click="store.browseScreenshotDir"
          class="px-4 py-2 rounded-lg bg-white/5 border border-white/10 text-sm hover:bg-white/10 hover:border-accent-emerald/50 transition-all">
          Browse
        </button>
      </div>
    </div>

    <!-- Recording Save Dir -->
    <div>
      <label class="text-xs font-medium text-gray-400 mb-2 block">Recording Save Location</label>
      <div class="flex items-center gap-3">
        <input v-model="store.recordingSaveDir" @blur="store.saveSetting('recordingSaveDir', store.recordingSaveDir)" type="text"
          placeholder="Default: ~/Videos/adb-powerhub"
          class="flex-1 bg-black/40 border border-white/10 rounded-lg py-2 px-4 text-sm focus:outline-none focus:border-accent-emerald/50" />
        <button @click="store.browseRecordingDir"
          class="px-4 py-2 rounded-lg bg-white/5 border border-white/10 text-sm hover:bg-white/10 hover:border-accent-emerald/50 transition-all">
          Browse
        </button>
      </div>
    </div>
  </section>
</template>