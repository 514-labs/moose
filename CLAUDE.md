# CLAUDE.md - Moose Project Guidelines

## Project-Specific Guidelines

### TypeScript Type Safety
When modifying configuration objects in TypeScript that are passed as parameters, avoid using `(config as any)` as it bypasses type safety. Instead, use object spread to create a new object:

```typescript
// ❌ Avoid this:
(config as any).newProperty = value;

// ✅ Prefer this:
config = { ...config, newProperty: value };
```

This maintains type safety while allowing configuration modifications, especially important for backwards compatibility scenarios.

### Backwards Compatibility Pattern
When implementing backwards compatibility for deprecated parameters:
1. Check if the old parameter exists
2. Show a deprecation warning
3. Only apply the old value if the new parameter is `undefined` (not `false`)
4. Use object spread for type-safe config updates

Example:
```typescript
if (config.oldParam !== undefined) {
  console.warn("⚠️  DEPRECATION WARNING: 'oldParam' is deprecated. Use 'newParam' instead.");
  if (config.newParam === undefined) {
    config = { ...config, newParam: config.oldParam };
  }
}
```