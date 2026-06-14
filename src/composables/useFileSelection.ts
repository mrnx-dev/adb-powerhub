import { useFileExplorerStore } from '../stores/fileExplorer';

export function useFileSelection() {
  const store = useFileExplorerStore();

  function toggle(path: string) {
    store.toggleSelected(path);
  }

  function selectSingle(path: string) {
    store.clearSelection();
    store.setSelected(path, true);
  }

  function selectRange(from: string, to: string) {
    store.selectRange(from, to);
  }

  function clear() {
    store.clearSelection();
  }

  return {
    selectedPaths: store.selectedPaths,
    toggle,
    selectSingle,
    selectRange,
    clear,
  };
}
