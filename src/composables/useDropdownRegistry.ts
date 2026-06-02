import { reactive, readonly } from 'vue';

/**
 * useDropdownRegistry — singleton composable for managing open Teleport-mounted
 * dropdowns across the app. Used by App.vue to close all dropdowns gracefully
 * when user navigates between views (M25/P25 fix for page-exit edge case).
 *
 * Why: Teleport moves dropdown DOM to <body>, OUTSIDE the exiting view's tree.
 * When user navigates while a dropdown is open, the dropdown would otherwise
 * linger visually then disappear on unmount with no leave animation.
 *
 * Usage in component:
 *   const dropdown = useDropdownRegistry('device-stats-badge');
 *   function toggle() {
 *     if (!dropdownOpen.value) { dropdownOpen.value = true; dropdown.open(); }
 *     else { dropdownOpen.value = false; dropdown.close(); }
 *   }
 *   onUnmounted(() => dropdown.close()); // cleanup on unmount
 *
 * Usage in App.vue (or any orchestrator):
 *   const dropdowns = useDropdownRegistry();
 *   watch(() => navStore.currentPage, () => dropdowns.closeAll());
 */

interface DropdownRegistryState {
  openDropdowns: Set<string>;
}

const state = reactive<DropdownRegistryState>({
  openDropdowns: new Set<string>(),
});

export function useDropdownRegistry(componentId?: string) {
  function open(id: string = componentId ?? '') {
    if (id) state.openDropdowns.add(id);
  }

  function close(id: string = componentId ?? '') {
    if (id) state.openDropdowns.delete(id);
  }

  function closeAll() {
    state.openDropdowns.clear();
  }

  return {
    state: readonly(state),
    open,
    close,
    closeAll,
  };
}
