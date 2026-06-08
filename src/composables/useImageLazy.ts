import { ref, onMounted, onBeforeUnmount, type Ref } from 'vue';

/**
 * IntersectionObserver-based lazy loading for thumbnail images.
 * Only mounts <img> elements when the card container scrolls near the viewport.
 *
 * @param elRef - Template ref to the container element
 * @param rootMargin - Preload margin (default '200%' = 2 viewport heights ahead)
 * @returns shouldLoad ref — when true, mount the <img>
 */
export function useImageLazy(elRef: Ref<HTMLElement | null>, rootMargin = '200%') {
  const shouldLoad = ref(false);
  let observer: IntersectionObserver | null = null;

  onMounted(() => {
    if (!elRef.value) return;
    observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          shouldLoad.value = true;
          observer?.disconnect();
        }
      },
      { rootMargin }
    );
    observer.observe(elRef.value);
  });

  onBeforeUnmount(() => observer?.disconnect());

  return { shouldLoad };
}
