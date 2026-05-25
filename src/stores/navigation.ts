import { defineStore } from "pinia";
import { ref } from "vue";

export const useNavigationStore = defineStore("navigation", () => {
  const currentPage = ref<"dashboard" | "settings">("dashboard");
  const focusTerminalRequested = ref(false);

  function navigateTo(page: "dashboard" | "settings") {
    currentPage.value = page;
  }

  function requestTerminalFocus() {
    focusTerminalRequested.value = true;
  }

  function clearTerminalFocusRequest() {
    focusTerminalRequested.value = false;
  }

  return { currentPage, focusTerminalRequested, navigateTo, requestTerminalFocus, clearTerminalFocusRequest };
});