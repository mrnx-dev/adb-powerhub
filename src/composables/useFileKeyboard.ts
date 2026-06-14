import type { Ref } from 'vue';
import { useFileExplorerStore } from '../stores/fileExplorer';

export function useFileKeyboard(containerRef: Ref<HTMLElement | null>) {
  const store = useFileExplorerStore();

  function scrollToFocused(path: string | null) {
    if (!path) return;
    const el = containerRef.value?.querySelector(`[data-path="${CSS.escape(path)}"]`);
    el?.scrollIntoView({ block: 'nearest' });
  }

  function handleKeydown(e: KeyboardEvent) {
    const visible = store.filteredEntries;
    const currentIdx = visible.findIndex((n) => n.path === store.focusedPath);

    if (e.key === 'ArrowDown') {
      e.preventDefault();
      const idx = Math.min(currentIdx + 1, visible.length - 1);
      store.setFocus(visible[idx]?.path ?? null);
      scrollToFocused(store.focusedPath);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      const idx = Math.max(currentIdx - 1, 0);
      store.setFocus(visible[idx]?.path ?? null);
      scrollToFocused(store.focusedPath);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      const node = visible[currentIdx];
      if (node?.entryType === 'folder') {
        store.navigateTo(node.path);
      } else if (node) {
        store.pull([node.path]);
      }
    } else if (e.key === ' ') {
      e.preventDefault();
      const node = visible[currentIdx];
      if (node) store.toggleSelected(node.path);
    } else if (e.key === 'Delete') {
      e.preventDefault();
      if (store.selectedPaths.size > 0) {
        store.deletePaths(Array.from(store.selectedPaths));
      }
    } else if (e.key === 'Escape') {
      store.clearSelection();
    }
  }

  return { handleKeydown };
}
