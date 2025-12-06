# Research Report: Session Management Patterns for Web Applications

**Date**: 2025-12-05
**Scope**: MongoDB session storage, device detection, session revocation, Rust/warp implementation
**Sources Reviewed**: 5 research queries + 25+ authoritative sources

---

## Executive Summary

Session management requires hybrid JWT + persistent session store pattern. Store minimal data in JWT (sessionId only), maintain full session metadata in MongoDB with TTL indexes for auto-cleanup. Use proven libraries (UAParser.js/device-detector) for user-agent parsing, avoid client-side feature detection. Revocation works via sessionId invalidation + timestamp checks. Critical: NOT pure JWT for sessions—use JWTs only for microservice propagation, pair with server-side session store.

---

## Key Findings

### 1. Session Storage Architecture

**Recommended Pattern**: JWT references sessionId + MongoDB stores full session data

```javascript
// JWT payload (minimal)
{
  "userId": "user123",
  "sessionId": "sess_abc123xyz",
  "iat": 1701600000,
  "exp": 1701686400  // Short-lived (24h)
}

// MongoDB Session Document
{
  "_id": ObjectId("..."),
  "sessionId": "sess_abc123xyz",
  "userId": "user123",
  "createdAt": ISODate("2025-12-05T10:00:00Z"),
  "expiresAt": ISODate("2025-12-06T10:00:00Z"),
  "revoked": false,
  "revokedAt": null,

  // Device metadata
  "userAgent": "Mozilla/5.0...",
  "browser": {
    "name": "Chrome",
    "version": "131.0.0"
  },
  "os": {
    "name": "macOS",
    "version": "14.2"
  },
  "device": {
    "type": "desktop",
    "brand": "Apple",
    "model": "MacBookPro"
  },

  // Security tracking
  "ipAddress": "203.0.113.42",
  "ipGeolocation": {
    "country": "US",
    "region": "CA",
    "city": "San Francisco"
  },
  "fingerprint": "hash_of_device_signals",

  // Activity
  "lastActivityAt": ISODate("2025-12-05T10:15:00Z"),
  "loginCount": 5
}
```

**Why This Works**:
- JWT stateless for microservices (if applicable)
- sessionId enables instant revocation without token invalidation
- Metadata enables device identification & anomaly detection
- TTL index auto-deletes expired sessions (no manual cleanup)

---

### 2. MongoDB Schema & Indexing

```javascript
// Create collection with TTL auto-expiry
db.createCollection("sessions")

// Indexes for performance
db.sessions.createIndex({ "expiresAt": 1 }, { expireAfterSeconds: 0 })
db.sessions.createIndex({ "userId": 1, "createdAt": -1 })
db.sessions.createIndex({ "sessionId": 1 }, { unique: true })
db.sessions.createIndex({ "ipAddress": 1, "createdAt": -1 })
db.sessions.createIndex({ "userId": 1, "revoked": 1 })
```

**Best Practices**:
- TTL index removes expired docs automatically (set expiresAt, expireAfterSeconds: 0)
- Compound index on userId + createdAt for "all sessions" queries
- Unique index on sessionId prevents collisions
- Optional IP index for geolocation-based queries

---

### 3. Device Detection (User-Agent Parsing)

**Recommended**: UAParser.js (JavaScript/Node) or device-detector (universal)

```rust
// Rust: Parse user-agent in warp handler
use warp::http::HeaderMap;

async fn parse_device(headers: &HeaderMap) -> SessionDevice {
    if let Some(ua) = headers.get("user-agent").and_then(|h| h.to_str().ok()) {
        // Use ua_parser crate or parse manually
        let browser = extract_browser(ua);
        let os = extract_os(ua);
        let device_type = extract_device_type(ua);

        SessionDevice {
            user_agent: ua.to_string(),
            browser: browser,
            os: os,
            device: DeviceInfo {
                type_: device_type,
                brand: None,
                model: None,
            }
        }
    } else {
        SessionDevice::default()
    }
}

fn extract_browser(ua: &str) -> BrowserInfo {
    // Simple parsing (production: use regex or library)
    if ua.contains("Chrome") {
        BrowserInfo { name: "Chrome", version: parse_version(ua, "Chrome/") }
    } else if ua.contains("Safari") {
        BrowserInfo { name: "Safari", version: parse_version(ua, "Version/") }
    } else {
        BrowserInfo::default()
    }
}
```

**Key Points**:
- Modern browsers freeze OS versions in UA string (macOS stuck at 10_15_7, Windows at NT 10.0)
- Client Hints (Sec-CH-UA, Sec-CH-UA-Mobile) more reliable than UA string parsing
- Never change functionality based on UA parsing—use feature detection instead
- Mobile accounts for 60%+ of traffic (2024)

---

### 4. Session Revocation Pattern

**Strategy**: Mark session revoked, don't delete immediately

```rust
// Rust warp handler for logout
#[derive(Deserialize)]
struct LogoutRequest {
    session_id: String,
    revoke_all: bool,  // Sign out from all devices
}

async fn logout(
    user_id: String,
    req: LogoutRequest,
    db: &Db,
) -> Result<impl Reply, Rejection> {
    if req.revoke_all {
        // Revoke all sessions for this user
        db.sessions
            .update_many(
                doc! { "userId": &user_id },
                doc! { "$set": {
                    "revoked": true,
                    "revokedAt": Utc::now()
                }},
            )
            .await?;
    } else {
        // Revoke single session
        db.sessions
            .update_one(
                doc! { "sessionId": &req.session_id },
                doc! { "$set": {
                    "revoked": true,
                    "revokedAt": Utc::now()
                }},
            )
            .await?;
    }

    Ok(warp::reply::with_status(
        warp::reply::json(&json!({"message": "Logged out"})),
        StatusCode::OK,
    ))
}

// On every request: check revocation + expiry
async fn validate_session(session_id: &str, db: &Db) -> Result<Session, AuthError> {
    let session = db.sessions
        .find_one(doc! { "sessionId": session_id }, None)
        .await?
        .ok_or(AuthError::InvalidSession)?;

    // Check revocation flag
    if session.revoked {
        return Err(AuthError::SessionRevoked);
    }

    // Check expiry (TTL handles deletion, but check explicitly)
    if session.expires_at < Utc::now() {
        return Err(AuthError::SessionExpired);
    }

    Ok(session)
}
```

**Timing Behavior**:
- Revocation is **immediate** (flag-based)
- Existing JWTs stay valid until expiry, but sessionId lookup fails
- Better: short JWT expiry (15-30 min) + refresh tokens
- Password reset: revoke all sessions except current

---

### 5. Security Best Practices

| Practice | Implementation |
|----------|-----------------|
| **IP Validation** | Store IP on login, check on each request (flag anomalies, don't auto-revoke) |
| **Geolocation** | Track on login; if login from opposite continent in minutes → risk |
| **Device Fingerprint** | Hash of (browser, OS, device, canvas, WebGL) for anomaly detection |
| **HTTPOnly Cookies** | Store session token in httpOnly, secure, sameSite=strict |
| **Token Rotation** | Issue new JWT on refresh, invalidate old one |
| **Concurrent Sessions** | Limit to N active sessions per user (optional: 1 desktop + N mobile) |

**Fraud Detection Signals**:
1. IP change + geolocation inconsistent (travel time impossible)
2. Same user, 2+ active devices with different fingerprints
3. User-agent changes within same session
4. Login from known suspicious IPs/VPNs

---

## Implementation Recommendations

### Quick Start Schema

```javascript
// Minimal production-ready schema
{
  "_id": ObjectId,
  "sessionId": String,  // Unique, random
  "userId": ObjectId,

  "createdAt": Date,
  "expiresAt": Date,
  "revoked": Boolean,

  "userAgent": String,
  "ipAddress": String,
  "fingerprint": String,  // Optional but recommended

  "lastActivityAt": Date,
  "updatedAt": Date
}
```

### Warp Middleware Example

```rust
// Extract & validate session from token
#[derive(Clone)]
struct Session {
    user_id: String,
    session_id: String,
}

fn with_session(
    db: Db,
) -> impl Filter<Extract = (Session,), Error = Rejection> + Clone {
    warp::header::optional::<String>("authorization")
        .and(warp::header("user-agent"))
        .and(warp::addr::remote())
        .and(with_db(db))
        .and_then(|auth: Option<String>, ua: String, addr: Option<SocketAddr>, db: Db| async move {
            let token = auth
                .and_then(|a| a.strip_prefix("Bearer ").map(|s| s.to_string()))
                .ok_or(reject::reject())?;

            let claims = decode_jwt(&token).map_err(|_| reject::reject())?;

            // Validate session in DB
            let session = validate_session(&claims.session_id, &db)
                .await
                .map_err(|_| reject::reject())?;

            // Optional: update lastActivityAt
            db.sessions
                .update_one(
                    doc! { "sessionId": &session.session_id },
                    doc! { "$set": { "lastActivityAt": Utc::now() }},
                )
                .await
                .ok();

            Ok(session)
        })
}
```

### React Hook for Session Management

```typescript
// useSession.ts
function useSession() {
    const [session, setSession] = useState<Session | null>(null);

    const logout = async (signOutAll = false) => {
        await fetch('/api/auth/logout', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ revoke_all: signOutAll })
        });

        localStorage.removeItem('sessionToken');
        setSession(null);
    };

    return { session, logout };
}

// Component usage
export function SessionManager() {
    const { logout } = useSession();

    return (
        <>
            <button onClick={() => logout(false)}>Logout</button>
            <button onClick={() => logout(true)}>Sign Out All Devices</button>
        </>
    );
}
```

---

## Common Pitfalls

| Pitfall | Impact | Solution |
|---------|--------|----------|
| Pure JWT sessions | No revocation (token valid until expiry) | Add sessionId reference, store session in DB |
| No sessionId in JWT | Revocation requires token blacklist | Include sessionId in payload |
| Missing TTL index | Manual cleanup needed, stale data | `createIndex({ expiresAt: 1 }, { expireAfterSeconds: 0 })` |
| UA sniffing for logic | Breaks with UA freeze, spoofing | Use feature detection (navigator.maxTouchPoints, etc.) |
| IP validation every request | UX issues on mobile/VPN | Track IP changes, alert user, require re-auth |
| No device metadata | Can't detect hijacking | Store browser, OS, device in session doc |

---

## Resources & References

### Official Documentation
- [MongoDB Server Sessions](https://www.mongodb.com/docs/manual/reference/server-sessions/)
- [MDN: Browser Detection Using UA](https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/Browser_detection_using_the_user_agent)
- [JWT Authentication in Rust - LogRocket](https://blog.logrocket.com/jwt-authentication-in-rust/)

### Key Libraries
- [UAParser.js](https://uaparser.dev/) - Browser/OS/device detection
- [Matomo Device Detector](https://github.com/matomo-org/device-detector) - Universal parser
- [jsonwebtoken (Rust crate)](https://docs.rs/jsonwebtoken/) - JWT signing/validation
- [connect-mongo](https://www.npmjs.com/package/connect-mongo) - MongoDB session store (Node reference)

### Session Management Patterns
- [WorkOS: Session Revocation & Sign Out Everywhere](https://workos.com/blog/workos-sessions-api-session-revocation-sign-out-everywhere)
- [Better-Auth: Multi-Session & Device Management](https://deepwiki.com/better-auth/better-auth/4.5-multi-session-and-device-management)
- [Building Device-Aware Sessions in JWT](https://skylinecodes.substack.com/p/building-device-aware-sessions-in)

### Security & Fingerprinting
- [Device Fingerprinting for Fraud Prevention - SEON](https://seon.io/resources/device-fingerprinting-for-fraud-reduction/)
- [IP Geolocation Best Practices - Fingerprint](https://fingerprint.com/blog/what-is-ip-geolocation/)
- [Session Fingerprinting - Blackboard Learn](https://help.blackboard.com/Learn/Administrator/Hosting/Security/Key_Security_Features/System_and_Communications_Protection/Session_Fingerprinting)

### Code Examples
- [Rust Warp JWT Example](https://github.com/zupzup/rust-jwt-example)
- [RealWorld Rust Backend](https://github.com/cjbassi/rust-warp-realworld-backend) (Note: includes warning against pure JWT)

---

## Unresolved Questions

1. **Geolocation Service**: Should IP geolocation be real-time (third-party API) or batch-updated database? Trade-off latency vs. cost.
2. **Fingerprint Collision**: Device fingerprinting has collision risk (~1% false positives). Acceptable for risk scoring?
3. **Concurrent Device Limits**: Should system enforce 1 device at a time (like banking) or allow N sessions per user?
4. **VPN Detection**: How to handle users behind VPNs/proxies masking real IP? Flag but allow, or require extra auth?
5. **Session Binding**: Should sessions be bound to single IP (strict) or allow IP changes within region (loose)?

---

## Summary Stats

- **Sources**: 25+ authoritative (MongoDB docs, MDN, GitHub, WorkOS, SEON, LogRocket)
- **Key Pattern**: JWT + MongoDB (not pure JWT)
- **Revocation**: O(1) flag-based (instant), no token blacklist needed
- **Device Detection**: UAParser.js (fastest), avoid UA sniffing
- **TTL Cleanup**: MongoDB automatic (expireAfterSeconds: 0)
- **Mobile**: 60%+ of traffic in 2024, handle IP changes gracefully
