import { describe, it, expect, beforeEach } from 'vitest';
import { useDropdownRegistry } from '../useDropdownRegistry';

describe('useDropdownRegistry', () => {
  beforeEach(() => {
    // Module-level singleton state — reset between tests.
    useDropdownRegistry().closeAll();
  });

  it('open() registers a dropdown id and close() removes it', () => {
    const dd = useDropdownRegistry('a');
    dd.open();
    expect(dd.state.openDropdowns.has('a')).toBe(true);

    dd.close();
    expect(dd.state.openDropdowns.has('a')).toBe(false);
  });

  it('open(id) can be called with an explicit id different from componentId', () => {
    const dd = useDropdownRegistry('a');
    dd.open('b');
    expect(dd.state.openDropdowns.has('b')).toBe(true);
    expect(dd.state.openDropdowns.has('a')).toBe(false);
  });

  it('multiple dropdowns can be open simultaneously (registry is a Set)', () => {
    const d1 = useDropdownRegistry('one');
    const d2 = useDropdownRegistry('two');
    d1.open();
    d2.open();
    expect(d1.state.openDropdowns.has('one')).toBe(true);
    expect(d1.state.openDropdowns.has('two')).toBe(true);
  });

  it('closeAll() clears every open dropdown', () => {
    const d1 = useDropdownRegistry('x');
    const d2 = useDropdownRegistry('y');
    d1.open();
    d2.open();
    d1.closeAll();
    expect(d1.state.openDropdowns.size).toBe(0);
  });

  it('close() on an unknown id is a no-op (no throw)', () => {
    const dd = useDropdownRegistry();
    expect(() => dd.close('not-there')).not.toThrow();
  });
});
