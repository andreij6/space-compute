import { describe, expect, it } from 'vitest';
import { runway } from './runway';

describe('runway', () => {
  it('shows no burn instead of u32::MAX', () => expect(runway(4_294_967_295, 'mo.')).toBe('no burn yet'));
  it('shows no burn instead of Infinity', () => expect(runway(Infinity, 'days')).toBe('no burn yet'));
  it('shows real counts with the unit', () => expect(runway(18, 'mo.')).toBe('18 mo.'));
});
