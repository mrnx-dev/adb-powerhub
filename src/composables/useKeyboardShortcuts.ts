import { onMounted, onUnmounted } from 'vue';
import { useNavigationStore } from '../stores/navigation';
import { useDeviceStore } from '../stores/device';
import { usePresetsStore } from '../stores/presets';

export function useKeyboardShortcuts() {
  const navStore = useNavigationStore();
  const deviceStore = useDeviceStore();
  const presetsStore = usePresetsStore();

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

    if (e.altKey && !e.ctrlKey && !e.metaKey) {
      const num = parseInt(e.key, 10);
      if (num >= 1 && num <= 9) {
        const target = e.target as HTMLElement;
        if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') return;

        const preset = presetsStore.getPresetByIndex(num - 1);
        if (preset && deviceStore.connected) {
          e.preventDefault();
          presetsStore.runPreset(preset);
        }
      }
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
