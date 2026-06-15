<script setup lang="ts">
import type { AppInfo, DependencyStatus } from '../../types/about';
import AppIdentityHeader from './AppIdentityHeader.vue';
import DependencyList from './DependencyList.vue';
import ExternalLinks from './ExternalLinks.vue';
import LicenseNotice from './LicenseNotice.vue';
import SupportActions from './SupportActions.vue';

defineProps<{
  appInfo: AppInfo | null;
  dependencies: DependencyStatus[];
}>();
</script>

<template>
  <article class="about-card card-glass group max-w-[640px] w-full p-6">
    <AppIdentityHeader :app-info="appInfo" />
    <DependencyList :dependencies="dependencies" />
    <ExternalLinks />
    <LicenseNotice />
    <SupportActions />
  </article>
</template>
