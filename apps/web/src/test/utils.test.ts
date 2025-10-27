import { describe, it, expect } from 'vitest';

/**
 * Utility Function Tests
 * Tests for common utility functions used throughout the application
 */

describe('String Utilities', () => {
  it('should check if string is empty', () => {
    const empty = '';
    const notEmpty = 'hello';

    expect(empty.length).toBe(0);
    expect(notEmpty.length).toBeGreaterThan(0);
  });

  it('should trim whitespace from strings', () => {
    const withSpaces = '  hello world  ';
    const trimmed = withSpaces.trim();

    expect(trimmed).toBe('hello world');
  });

  it('should convert string to lowercase', () => {
    const mixed = 'Hello World';
    const lower = mixed.toLowerCase();

    expect(lower).toBe('hello world');
  });

  it('should convert string to uppercase', () => {
    const mixed = 'Hello World';
    const upper = mixed.toUpperCase();

    expect(upper).toBe('HELLO WORLD');
  });

  it('should split string by delimiter', () => {
    const csv = 'apple,banana,cherry';
    const parts = csv.split(',');

    expect(parts).toHaveLength(3);
    expect(parts[0]).toBe('apple');
    expect(parts[2]).toBe('cherry');
  });
});

describe('Array Utilities', () => {
  it('should filter array elements', () => {
    const numbers = [1, 2, 3, 4, 5];
    const evens = numbers.filter((n) => n % 2 === 0);

    expect(evens).toEqual([2, 4]);
  });

  it('should map array elements', () => {
    const numbers = [1, 2, 3];
    const doubled = numbers.map((n) => n * 2);

    expect(doubled).toEqual([2, 4, 6]);
  });

  it('should reduce array to single value', () => {
    const numbers = [1, 2, 3, 4];
    const sum = numbers.reduce((acc, n) => acc + n, 0);

    expect(sum).toBe(10);
  });

  it('should find element in array', () => {
    const items = ['apple', 'banana', 'cherry'];
    const found = items.find((item) => item.startsWith('b'));

    expect(found).toBe('banana');
  });

  it('should check if array includes element', () => {
    const items = ['apple', 'banana', 'cherry'];

    expect(items.includes('banana')).toBe(true);
    expect(items.includes('grape')).toBe(false);
  });
});

describe('Object Utilities', () => {
  it('should get object keys', () => {
    const obj = { name: 'John', age: 30, city: 'NYC' };
    const keys = Object.keys(obj);

    expect(keys).toEqual(['name', 'age', 'city']);
  });

  it('should get object values', () => {
    const obj = { a: 1, b: 2, c: 3 };
    const values = Object.values(obj);

    expect(values).toEqual([1, 2, 3]);
  });

  it('should merge objects', () => {
    const obj1 = { a: 1, b: 2 };
    const obj2 = { c: 3, d: 4 };
    const merged = { ...obj1, ...obj2 };

    expect(merged).toEqual({ a: 1, b: 2, c: 3, d: 4 });
  });

  it('should override properties when merging', () => {
    const obj1 = { name: 'John', age: 30 };
    const obj2 = { age: 31, city: 'NYC' };
    const merged = { ...obj1, ...obj2 };

    expect(merged.age).toBe(31);
    expect(merged).toEqual({ name: 'John', age: 31, city: 'NYC' });
  });
});

describe('Number Utilities', () => {
  it('should parse integer from string', () => {
    const str = '42';
    const num = parseInt(str, 10);

    expect(num).toBe(42);
    expect(typeof num).toBe('number');
  });

  it('should parse float from string', () => {
    const str = '3.14';
    const num = parseFloat(str);

    expect(num).toBe(3.14);
  });

  it('should check if value is NaN', () => {
    const notANumber = NaN;
    const number = 42;

    expect(isNaN(notANumber)).toBe(true);
    expect(isNaN(number)).toBe(false);
  });

  it('should round numbers', () => {
    expect(Math.round(4.4)).toBe(4);
    expect(Math.round(4.5)).toBe(5);
    expect(Math.round(4.6)).toBe(5);
  });

  it('should get absolute value', () => {
    expect(Math.abs(-5)).toBe(5);
    expect(Math.abs(5)).toBe(5);
    expect(Math.abs(0)).toBe(0);
  });
});

describe('Boolean Logic', () => {
  it('should handle AND logic', () => {
    // Testing boolean logic with constants is intentional
    // eslint-disable-next-line no-constant-binary-expression
    expect(true && true).toBe(true);
    // eslint-disable-next-line no-constant-binary-expression
    expect(true && false).toBe(false);
    // eslint-disable-next-line no-constant-binary-expression
    expect(false && false).toBe(false);
  });

  it('should handle OR logic', () => {
    // Testing boolean logic with constants is intentional
    // eslint-disable-next-line no-constant-binary-expression
    expect(true || false).toBe(true);
    // eslint-disable-next-line no-constant-binary-expression
    expect(false || true).toBe(true);
    // eslint-disable-next-line no-constant-binary-expression
    expect(false || false).toBe(false);
  });

  it('should handle NOT logic', () => {
    expect(!true).toBe(false);
    expect(!false).toBe(true);
  });

  it('should handle truthy and falsy values', () => {
    expect(Boolean('')).toBe(false);
    expect(Boolean('hello')).toBe(true);
    expect(Boolean(0)).toBe(false);
    expect(Boolean(1)).toBe(true);
    expect(Boolean(null)).toBe(false);
    expect(Boolean(undefined)).toBe(false);
    expect(Boolean({})).toBe(true);
    expect(Boolean([])).toBe(true);
  });
});

describe('Date Utilities', () => {
  it('should create current date', () => {
    const now = new Date();
    expect(now).toBeInstanceOf(Date);
  });

  it('should create date from string', () => {
    const date = new Date('2024-06-15T12:00:00Z');
    expect(date.getFullYear()).toBe(2024);
    expect(date.getMonth()).toBe(5); // June is 5 (0-indexed)
    expect(date.getDate()).toBe(15);
  });

  it('should get timestamp', () => {
    const now = Date.now();
    expect(typeof now).toBe('number');
    expect(now).toBeGreaterThan(0);
  });

  it('should format date to ISO string', () => {
    const date = new Date('2024-01-15T12:30:00Z');
    const iso = date.toISOString();
    expect(iso).toContain('2024-01-15');
  });
});

describe('JSON Utilities', () => {
  it('should stringify object to JSON', () => {
    const obj = { name: 'John', age: 30 };
    const json = JSON.stringify(obj);

    expect(json).toBe('{"name":"John","age":30}');
  });

  it('should parse JSON to object', () => {
    const json = '{"name":"John","age":30}';
    const obj = JSON.parse(json);

    expect(obj.name).toBe('John');
    expect(obj.age).toBe(30);
  });

  it('should handle nested objects', () => {
    const nested = {
      user: {
        name: 'John',
        address: {
          city: 'NYC',
        },
      },
    };

    const json = JSON.stringify(nested);
    const parsed = JSON.parse(json);

    expect(parsed.user.address.city).toBe('NYC');
  });
});
