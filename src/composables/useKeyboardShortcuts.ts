import { onMounted, onUnmounted } from 'vue';
import { useNavigationStore } from '../stores/navigation';
import { useDeviceStore } from '../stores/device';

export function useKeyboardShortcuts() {
  const navStore = useNavigationStore();
  const deviceStore = useDeviceStore();

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      if (navStore.connectPanelOpen) {
        navStore.closeConnectPanel();
        e.preventDefault();
        return;
      }
      if (deviceStore.showRebootMenu) {
        deviceStore.showRebootMenu = false;
        e.preventDefault();
      }
    }

    if (e.ctrlKey && e.key === 'k') {
      e.preventDefault();
      navStore.navigateTo('dashboard');
      navStore.requestTerminalFocus();
      return;
    }

    if (e.ctrlKey && e.key.toLowerCase() === 'm') {
      e.preventDefault();
      if (deviceStore.mirroring) {
        deviceStore.stopMirror();
      } else {
        deviceStore.launchMirror();
      }
      return;
    }

    if (e.ctrlKey && e.key === ',') {
      e.preventDefault();
      navStore.navigateTo('settings');
      return;
    }

    if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === 'd') {
      e.preventDefault();
      if (deviceStore.connected) {
        deviceStore.disconnect();
      }
      return;
    }
  }

  onMounted(() => {
    document.addEventListener('keydown', handleKeyDown);
  });

  onUnmounted(() => {
    document.removeEventListener('keydown', handleKeyDown);
  });
}
