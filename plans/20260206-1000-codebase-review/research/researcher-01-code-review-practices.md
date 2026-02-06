# Code Review Best Practices & Code Smells Detection
**Date**: 2026-02-06 | **Status**: Complete | **Research Scope**: Rust, Python, TypeScript/React

---

## 1. RUST CODE SMELLS & ANTI-PATTERNS

### Critical Issues
- **Unwrap Abuse**: Use `?` operator, `match`, `if let`, or `.unwrap_or()` instead of bare `unwrap()`/`expect()`
- **Unsafe Block Misuse**: Unsafe doesn't mean "forbidden code allowed" — it means "compiler cannot verify safety"
- **Excessive Cloning**: Clone only when necessary; prefer references and borrowing
- **Blocking I/O in Async**: Use `.await` properly; don't block executor with sync operations
- **Mutex Misuse**: Lock duration should be minimal; avoid nested locks (deadlock risk)

### Reviewer Checklist
- [ ] Zero production `unwrap()` calls (use Clippy: `clippy::unwrap_used`)
- [ ] Each `unsafe` block justified with comment explaining invariants
- [ ] No `clone()` in hot loops
- [ ] Async tasks don't call blocking functions (fs, network sync)
- [ ] Mutex guards released before awaits

---

## 2. PYTHON CODE SMELLS & ANTI-PATTERNS

### Critical Issues
- **Type Hint Gaps**: Async functions, decorators, and callbacks need explicit type hints
- **Async/Await Antipatterns**:
  - Not awaiting tasks (fire-and-forget without tracking)
  - Mixing sync/async (deadlock risk with asyncio)
  - Missing exception handling in async context
- **Memory Leaks**: Circular references, unclosed resources, lingering handlers
- **State Management**: Mutable shared state without proper synchronization

### Reviewer Checklist
- [ ] Async functions typed: `async def func() -> Awaitable[Type]:`
- [ ] Decorators preserve async type hints: `Callable[[P], Awaitable[T]]`
- [ ] All `asyncio.create_task()` calls tracked/awaited
- [ ] Context managers used for file/connection handling
- [ ] No circular imports or module-level mutable state
- [ ] Exception handling in async code (not just try/except)

---

## 3. TYPESCRIPT/REACT CODE SMELLS & ANTI-PATTERNS

### Critical Issues: useEffect
- **Missing Cleanup Functions**: Event listeners, timers, subscriptions must be cleaned up
- **Stale Closures**: useEffect dependencies incomplete → race conditions
- **Unhandled Promises**: Fetch/async operations completing after unmount
- **Memory Leaks from State Updates**: State set on unmounted components

### Critical Issues: State Management
- **Excessive Data in State**: Component accumulates data over time
- **No AbortController**: Fetch requests not cancelled on unmount
- **Event Listeners Never Removed**: Attached to window/document without cleanup
- **Interval/Timeout Leaks**: clearInterval/clearTimeout missing

### Reviewer Checklist
- [ ] Every useEffect has cleanup function (return callback)
- [ ] Dependency array includes all used variables
- [ ] Async calls wrapped in AbortController
- [ ] Event listeners removed in cleanup: `element.removeEventListener()`
- [ ] Timers cleared: `clearTimeout/clearInterval` in cleanup
- [ ] No `setState` after component unmount (check isMounted flag)
- [ ] useCallback/useMemo optimizations where needed
- [ ] No object/array literals in useEffect deps (causes infinite loops)

---

## 4. CROSS-LANGUAGE: RACE CONDITIONS

### Signs to Look For
- **Unprotected Shared State**: Multiple threads/tasks reading/writing same variable
- **Check-Then-Act Gaps**: State checked, then action taken (non-atomic)
- **Concurrent Map/Array Access**: No locks in Rust/Python concurrent code
- **WebSocket Race**: Message handlers with shared state (async JS)

### Reviewer Checklist
- [ ] Shared state access protected (mutex/lock/atomic)
- [ ] No time gap between check and action
- [ ] Concurrent collections used (Rust: Arc<Mutex<T>>, Python: thread-safe queues)
- [ ] WebSocket handlers don't mutate shared state directly

---

## 5. CROSS-LANGUAGE: MEMORY LEAKS

### Detection Pattern
- **Unhandled Resources**: File handles, network connections, timers not closed
- **Circular References**: Objects holding references to each other
- **Event Handler Retention**: Listeners attached but never removed
- **Stuck Async Operations**: Promises/tasks that never complete

### Reviewer Checklist
- [ ] All resources properly closed (using `with`, try/finally, or cleanup functions)
- [ ] No circular object references
- [ ] Event listeners registered & unregistered in pairs
- [ ] Async operations have completion/error handling
- [ ] No global caches without eviction policy
- [ ] Timers/intervals always cleared

---

## 6. SECURITY VULNERABILITIES

### High-Priority Checks
- **Injection Attacks**: User input passed to SQL/shell/eval without sanitization
- **Authentication Bypass**: Token validation missing or weak
- **Credential Exposure**: API keys, secrets in code or logs
- **Data Exposure**: Sensitive data logged, sent unencrypted, or stored in plain text

### Reviewer Checklist
- [ ] User input validated/sanitized before use
- [ ] Authentication checks on all protected endpoints
- [ ] No hardcoded credentials (check .env, secrets)
- [ ] Sensitive data not logged (passwords, tokens, PII)
- [ ] Error messages don't reveal internal details
- [ ] Database queries use parameterized statements (no string concat)

---

## 7. REFACTORING INDICATORS

### Red Flags
- **God Functions**: >50 lines doing multiple things (split into smaller functions)
- **Duplicate Code**: Same logic in 2+ places (extract to shared function)
- **Poor Naming**: Variables/functions with cryptic names (rename to intent)
- **Inconsistent Style**: Mixing naming conventions, indentation, formatting
- **Low Cohesion**: Class/module has unrelated responsibilities (split it)
- **High Coupling**: Changes to A break B (decouple with interfaces/traits)

### Reviewer Checklist
- [ ] Functions <40 lines (max 60 for complex logic)
- [ ] No copy-paste code (extract DRY violations)
- [ ] Names describe intent (no x, temp, data, result)
- [ ] Consistent formatting throughout (run linters)
- [ ] Single responsibility per class/module
- [ ] External dependencies injected (testable, decoupled)

---

## QUICK REVIEW WORKFLOW

**Before merging:**
1. Run linters (Clippy, Black, ESLint) — must be clean
2. Run tests — >90% coverage required
3. Check for code smells above
4. Verify security checklist
5. Confirm no performance regressions

**Tools to Enforce:**
- **Rust**: Clippy (clippy::unwrap_used, clippy::missing_docs)
- **Python**: Black, Flake8, MyPy (strict mode)
- **TypeScript**: ESLint, strict mode, Prettier

---

## SOURCES
- [Rust Anti-Patterns 2025](https://medium.com/solo-devs/the-7-rust-anti-patterns-that-are-secretly-killing-your-performance-and-how-to-fix-them-in-2025-dcebfdef7b54)
- [Unsafe Rust Guide](https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html)
- [React Memory Leak Prevention](https://www.wisdomgeek.com/development/web-development/react/avoiding-race-conditions-memory-leaks-react-useeffect/)
- [Security Code Review Checklist](https://redwerk.com/blog/security-code-review-checklist/)
- [OWASP Code Review Guide](https://owasp.org/www-project-code-review-guide/assets/OWASP_Code_Review_Guide_v2.pdf)
