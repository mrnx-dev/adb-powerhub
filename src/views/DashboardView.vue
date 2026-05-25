<script setup lang="ts">
import ConnectCard from "../components/ConnectCard.vue";
import DeviceStatsCard from "../components/DeviceStatsCard.vue";
import MirrorCard from "../components/MirrorCard.vue";
import TerminalCard from "../components/TerminalCard.vue";
</script>

<template>
  <div class="flex-1 overflow-hidden bg-glow p-4 flex flex-col gap-4">
    <DeviceStatsCard />
    <div class="grid grid-cols-2 gap-4">
      <ConnectCard />
      <MirrorCard />
    </div>
    <TerminalCard />
  </div>
</template>