import {describe,expect,it} from 'vitest';
import {escapeHtml,parseStrategies,statusLabel} from './types';
describe('board helpers',()=>{
  it('parses only strategy arrays',()=>{expect(parseStrategies('["diagram"]')).toEqual(['diagram']);expect(parseStrategies('{"x":1}')).toEqual([])});
  it('escapes user content',()=>expect(escapeHtml('<img onerror="x">')).toBe('&lt;img onerror=&quot;x&quot;&gt;'));
  it('uses words as well as symbols for status',()=>expect(statusLabel('exploring')).toContain('Exploring'));
});
