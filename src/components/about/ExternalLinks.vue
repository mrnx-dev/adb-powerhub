<script setup lang="ts">
import { open } from '@tauri-apps/plugin-shell';

interface AboutLink {
  key: string;
  label: string;
  url: string;
}

const links: AboutLink[] = [
  {
    key: 'repository',
    label: 'Repository',
    url: import.meta.env.VITE_APP_REPO_URL || '',
  },
  {
    key: 'documentation',
    label: 'Documentation',
    url: import.meta.env.VITE_APP_DOCS_URL || '',
  },
  {
    key: 'issues',
    label: 'Report Issue',
    url: import.meta.env.VITE_APP_ISSUES_URL || '',
  },
  {
    key: 'scrcpy',
    label: 'scrcpy',
    url: 'https://github.com/Genymobile/scrcpy',
  },
].filter((link) => link.url.length > 0);

async function openLink(url: string) {
  try {
    await open(url);
  } catch (e) {
    console.error('Failed to open link:', e);
  }
}
</script>

<template>
  <section class="about-section" aria-labelledby="links-label">
    <h2 id="links-label" class="section-label">Links</h2>
    <div v-if="links.length === 0" class="text-sm text-theme-muted">
      No external links configured.
    </div>
    <div v-else class="about-links flex flex-wrap gap-2">
      <button
        v-for="link in links"
        :key="link.key"
        type="button"
        class="btn btn-pressable"
        @click="openLink(link.url)"
      >
        {{ link.label }}
      </button>
    </div>
  </section>
</template>
