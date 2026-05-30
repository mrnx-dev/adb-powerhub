import { defineStore } from 'pinia';
import { ref } from 'vue';

export const useNavigationStore = defineStore('navigation', () => {
  const currentPage = ref<'dashboard' | 'logcat' | 'settings'>('dashboard');
  const focusTerminalRequested = ref(false);
  const connectPanelOpen = ref(false);

  function navigateTo(page: 'dashboard' | 'logcat' | 'settings') {
    currentPage.value = page;
  }

  function requestTerminalFocus() {
    focusTerminalRequested.value = true;
  }

  function clearTerminalFocusRequest() {
    focusTerminalRequested.value = false;
  }

  function openConnectPanel() {
    connectPanelOpen.value = true;
  }

  function closeConnectPanel() {
    connectPanelOpen.value = false;
  }

  function toggleConnectPanel() {
    connectPanelOpen.value = !connectPanelOpen.value;
  }

  return {
    currentPage,
    focusTerminalRequested,
    connectPanelOpen,
    navigateTo,
    requestTerminalFocus,
    clearTerminalFocusRequest,
    openConnectPanel,
    closeConnectPanel,
    toggleConnectPanel,
  };
});
