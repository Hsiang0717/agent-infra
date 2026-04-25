# TypeScript / JavaScript (JSDoc Style)

## Example Module
```typescript
/**
 * @module AuthService
 * @see {@link UserRepository} for persistence logic
 */

/**
 * Validates and authenticates a user.
 * @param credentials - Structured user credentials
 * @returns Token or null if failed
 * @modifies SessionStore - sets auth-token cookie
 * @calls IdentityProvider - for OAuth validation
 * @pre credentials must be sanitized
 */
export const authenticate = (credentials: Credentials) => { ... }
```
