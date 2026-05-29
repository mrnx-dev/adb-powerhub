<script setup lang="ts">
import DeviceStatsCard from '../components/DeviceStatsCard.vue';
import MirrorCard from '../components/MirrorCard.vue';
import TerminalCard from '../components/TerminalCard.vue';
</script>

<template>
  <div class="flex-1 overflow-hidden bg-glow p-6 flex flex-col gap-6">
    <DeviceStatsCard />
    <MirrorCard />
    <TerminalCard />
  </div>
</template>
