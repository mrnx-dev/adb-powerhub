import { describe, it, expect } from 'vitest';
import { useLogcatStore, type LogEntry } from '../logcat';

function makeEntry(partial: Partial<LogEntry>): LogEntry {
  return {
    id: 0,
    timestamp: '06-18 10:00:00.000',
    pid: '1234',
    tid: '1234',
    level: 'I',
    tag: 'TestTag',
    message: 'hello',
    firstLine: 'hello',
    contLines: [],
    hasCont: false,
    displayMessage: 'hello',
    ...partial,
  };
}

describe('useLogcatStore — filteredEntries', () => {
  it('filterLevel narrows to the selected level', () => {
    const store = useLogcatStore();
    store.appendEntry(makeEntry({ id: 1, level: 'V', tag: 'A', message: 'verbose' }));
    store.appendEntry(makeEntry({ id: 2, level: 'E', tag: 'B', message: 'error' }));
    store.appendEntry(makeEntry({ id: 3, level: 'W', tag: 'C', message: 'warn' }));

    store.filterLevel = 'E';
    expect(store.filteredEntries.map((e) => e.level)).toEqual(['E']);

    store.filterLevel = 'ALL';
    expect(store.filteredEntries).toHaveLength(3);
  });

  it('tag include filter matches case-insensitively', () => {
    const store = useLogcatStore();
    store.appendEntry(makeEntry({ id: 1, tag: 'ActivityManager', message: 'am' }));
    store.appendEntry(makeEntry({ id: 2, tag: 'WiFiService', message: 'wifi' }));

    store.addTagFilter('wifi');
    expect(store.tagFilters).toHaveLength(1);
    expect(store.tagFilters[0].mode).toBe('include');
    expect(store.filteredEntries.map((e) => e.tag)).toEqual(['WiFiService']);
  });

  it('tag exclude filter removes matching tags', () => {
    const store = useLogcatStore();
    store.appendEntry(makeEntry({ id: 1, tag: 'ActivityManager', message: 'am' }));
    store.appendEntry(makeEntry({ id: 2, tag: 'WiFiService', message: 'wifi' }));

    store.addTagFilter('wifi');
    store.toggleTagMode(0); // include -> exclude
    expect(store.tagFilters[0].mode).toBe('exclude');
    expect(store.filteredEntries.map((e) => e.tag)).toEqual(['ActivityManager']);
  });

  it('multiple include filters use OR semantics', () => {
    const store = useLogcatStore();
    store.appendEntry(makeEntry({ id: 1, tag: 'Alpha', message: 'a' }));
    store.appendEntry(makeEntry({ id: 2, tag: 'Beta', message: 'b' }));
    store.appendEntry(makeEntry({ id: 3, tag: 'Gamma', message: 'g' }));

    store.addTagFilter('alpha');
    store.addTagFilter('beta');
    expect(store.filteredEntries.map((e) => e.tag).sort()).toEqual(['Alpha', 'Beta']);
  });

  it('searchQuery filters across the full haystack', () => {
    const store = useLogcatStore();
    store.appendEntry(
      makeEntry({ id: 1, tag: 'Net', message: 'connection established', pid: '100' })
    );
    store.appendEntry(makeEntry({ id: 2, tag: 'Net', message: 'connection dropped', pid: '200' }));

    store.searchQuery = '100';
    expect(store.filteredEntries.map((e) => e.pid)).toEqual(['100']);
  });

  it('clearTagFilters resets to show all', () => {
    const store = useLogcatStore();
    store.appendEntry(makeEntry({ id: 1, tag: 'Alpha' }));
    store.appendEntry(makeEntry({ id: 2, tag: 'Beta' }));
    store.addTagFilter('alpha');
    expect(store.filteredEntries).toHaveLength(1);

    store.clearTagFilters();
    expect(store.tagFilters).toHaveLength(0);
    expect(store.filteredEntries).toHaveLength(2);
  });

  it('addTagFilter ignores empty and duplicate values', () => {
    const store = useLogcatStore();
    store.addTagFilter('   ');
    expect(store.tagFilters).toHaveLength(0);
    store.addTagFilter('wifi');
    store.addTagFilter('WIFI'); // duplicate, case-insensitive
    expect(store.tagFilters).toHaveLength(1);
  });

  it('setPaused flips status between PAUSED and LIVE/IDLE', () => {
    const store = useLogcatStore();
    expect(store.status).toBe('IDLE');
    store.streaming = true;
    store.setPaused(true);
    expect(store.paused).toBe(true);
    expect(store.status).toBe('PAUSED');
    store.setPaused(false);
    expect(store.status).toBe('LIVE');
  });
});
