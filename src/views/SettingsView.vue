<script setup lang="ts">
import BinaryPathsCard from "../components/settings/BinaryPathsCard.vue";
import GeneralPrefsCard from "../components/settings/GeneralPrefsCard.vue";
import DisplayRecordingCard from "../components/settings/DisplayRecordingCard.vue";
</script>

<template>
  <div class="flex-1 overflow-y-auto bg-glow p-4 space-y-4">
    <BinaryPathsCard />
    <GeneralPrefsCard />
    <DisplayRecordingCard />
  </div>
</template>