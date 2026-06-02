<script setup lang="ts">
import DeviceStatsCard from '../components/DeviceStatsCard.vue';
import MirrorCard from '../components/MirrorCard.vue';
import TerminalCard from '../components/TerminalCard.vue';
</script>

<template>
  <div tabindex="-1" class="flex-1 min-h-0 overflow-hidden bg-glow p-6 flex flex-col gap-6">
    <TransitionGroup
      tag="div"
      name="card-stagger"
      appear
      class="flex flex-col gap-6 flex-1 min-h-0"
    >
      <DeviceStatsCard key="device-stats" :style="{ '--stagger-index': 0 }" />
      <MirrorCard key="mirror" :style="{ '--stagger-index': 1 }" />
      <TerminalCard key="terminal" :style="{ '--stagger-index': 2 }" />
    </TransitionGroup>
  </div>
</template>
