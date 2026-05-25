import { onMounted, onUnmounted } from "vue";
import { useNavigationStore } from "../stores/navigation";
import { useDeviceStore } from "../stores/device";

export function useKeyboardShortcuts() {
  const navStore = useNavigationStore();
  const deviceStore = useDeviceStore();

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      if (deviceStore.showRebootMenu) {
        deviceStore.showRebootMenu = false;
        e.preventDefault();
      }
    }

    if (e.ctrlKey && e.key === "k") {
      e.preventDefault();
      navStore.navigateTo("dashboard");
      navStore.requestTerminalFocus();
    }
  }

  onMounted(() => {
    document.addEventListener("keydown", handleKeyDown);
  });

  onUnmounted(() => {
    document.removeEventListener("keydown", handleKeyDown);
  });
}