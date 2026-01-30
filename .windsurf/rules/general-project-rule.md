---
trigger: always_on
---

---

## description: General Development guidelines for this repo

# Development Standards

This repo follows strict development standards for TypeScript, DrizzleORM, and Cloudflare developer tools.

## Technology Stack

### Core Technologies

- **TypeScript**: All code must be written in TypeScript
- **DrizzleORM**: Database ORM to interact with Cloudflare SQLites Durable Objects
- **PrismaORM**: Database ORM to interact with Cloudflare D1 Database storage

### Cloudflare Tools

- **DO**: Durable Objects Store
- **D1**: Serverless database
- **KV**: Key-value storage
- **R2**: File storage
- **AI**: Multimodal inference
- **Workers**: Edge computing platform

### Programming Patterns

1. Use functional and declarative programming patterns
2. Prefer iteration and modularization over code duplication
3. Use descriptive variable names with auxiliary verbs (isLoading, hasError, canSubmit)

### Key Mindsets

- **Simplicity**: Write simple and straightforward code
- **Readability**: Ensure your code is easy to read and understand
- **Performance**: Keep performance in mind but do not over-optimize at the cost of readability
- **Maintainability**: Write code that is easy to maintain and update
- **Testability**: Ensure your code is easy to test, that it has tests that prove its working, and that the tests pass
- **Reusability**: Write reusable components and functions that are DRY
- **Modularity**: Break logic into small, independent modules

## Code Style and Structure

### Basic Principles

- Always declare the type of each variable and function (parameters and return value).
- Avoid using any.
- Create necessary types (interfaces/types, discriminated unions) and reuse them. When it makes sense, define those types in another type specific file or folder. 
- Use JSDoc to document public classes, methods, and exported functions.

### TypeScript Standards

- Strong typing: Prefer precise types over `any`.
- Explicit return types: Declare explicit return types for all exported functions, public class methods, and any non-trivial functions.
- Type guards: Narrow values with user-defined type guards and predicate functions instead of loosening types.
- API stability: Keep exported types and function signatures stable; prefer additive, backward-compatible changes.
- Nullability: Prefer `undefined` for optional fields; be explicit with `null` only when semantically meaningful.
- Error types: Model domain errors with discriminated unions rather than overloading exceptions.

## API Standards

- **Response Format**: All APIs must adhere to using `JSONV4Response` type for consistent response formatting, and API error responses should use ApiError
- **Helper Location**: API response helpers and utilities can be found in `src/lib/api/v4.ts`
- **Implementation**: Use the provided helper functions to ensure standardized error handling and success responses
- **Durable Objects: Durable objects should use RPC, and not REST/HTTP APIs

## Git Standards
- **Moving/Renaming files: Use git mv instead of moving

## General Principles

```typescript
// Good: Concise, technical TypeScript with accurate examples
function processUserData(userData: UserProfile): ProcessedData {
  return {
    isActive: userData.status === "active",
    hasPermissions: userData.permissions.length > 0,
    displayName: userData.firstName + " " + userData.lastName,
  };
}

// Bad: Verbose, unclear naming
class UserProcessor {
  process(data: any) {
    // Implementation
  }
}
```

## Testing Standards
- Use direct endpoint class testing instead of unstable_dev. Use new MyEndpoint() + mock dependencies. Avoid unstable_dev() (spawns full workers, slow, complex)
- Add tests for all code we add, with tests being unit tests, integration tests that use vitest-pool-worker, and E2E tests that use http calls
- Run relevant tests after making changes to code