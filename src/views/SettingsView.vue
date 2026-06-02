<script setup lang="ts">
import BinaryPathsCard from '../components/settings/BinaryPathsCard.vue';
import GeneralPrefsCard from '../components/settings/GeneralPrefsCard.vue';
import DisplayRecordingCard from '../components/settings/DisplayRecordingCard.vue';
</script>

<template>
  <div tabindex="-1" class="flex-1 min-h-0 overflow-y-auto bg-glow p-6 space-y-6">
    <BinaryPathsCard />
    <GeneralPrefsCard />
    <DisplayRecordingCard />
  </div>
</template>
