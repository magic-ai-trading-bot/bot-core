# Phase 03: Frontend Integration

**Status**: Pending
**Priority**: High

## Context

- UI exists in Settings.tsx (lines 1004-1168)
- Handlers are stubs: onChange={() => {}}
- Need: Connect to real APIs

## Implementation Steps

### 3.1 Add API Functions (api.ts or new auth-api.ts)

```typescript
// Security API functions
export const authApi = {
  changePassword: (current: string, newPass: string) =>
    axios.post('/api/auth/change-password', { current_password: current, new_password: newPass }),

  updateProfile: (displayName: string) =>
    axios.patch('/api/auth/profile', { display_name: displayName }),

  setup2FA: () =>
    axios.post('/api/auth/2fa/setup'),

  verify2FA: (code: string) =>
    axios.post('/api/auth/2fa/verify', { code }),

  disable2FA: (code: string) =>
    axios.post('/api/auth/2fa/disable', { code }),

  getSessions: () =>
    axios.get('/api/auth/sessions'),

  revokeSession: (sessionId: string) =>
    axios.delete(`/api/auth/sessions/${sessionId}`),

  revokeAllSessions: () =>
    axios.post('/api/auth/sessions/revoke-all'),
};
```

### 3.2 Update Settings.tsx Security Section

#### Profile Update
```typescript
const [displayName, setDisplayName] = useState("Crypto Trader");

const handleUpdateProfile = async () => {
  await authApi.updateProfile(displayName);
  toast({ title: "Profile Updated" });
};
```

#### Change Password
```typescript
const [passwords, setPasswords] = useState({ current: '', new: '', confirm: '' });

const handleChangePassword = async () => {
  if (passwords.new !== passwords.confirm) {
    toast({ title: "Passwords don't match", variant: "destructive" });
    return;
  }
  await authApi.changePassword(passwords.current, passwords.new);
  toast({ title: "Password Changed" });
  setPasswords({ current: '', new: '', confirm: '' });
};
```

#### 2FA Management
```typescript
const [qrCode, setQrCode] = useState<string | null>(null);
const [verifyCode, setVerifyCode] = useState('');

const handleSetup2FA = async () => {
  const { data } = await authApi.setup2FA();
  setQrCode(data.data.qr_code);
};

const handleVerify2FA = async () => {
  await authApi.verify2FA(verifyCode);
  setTwoFactorEnabled(true);
  setQrCode(null);
};
```

#### Sessions Management
```typescript
const [sessions, setSessions] = useState<Session[]>([]);

useEffect(() => {
  authApi.getSessions().then(res => setSessions(res.data.data));
}, []);

const handleRevokeSession = async (sessionId: string) => {
  await authApi.revokeSession(sessionId);
  setSessions(prev => prev.filter(s => s.session_id !== sessionId));
};

const handleRevokeAll = async () => {
  await authApi.revokeAllSessions();
  // Refresh sessions list
  const { data } = await authApi.getSessions();
  setSessions(data.data);
};
```

## Files to Modify

- `nextjs-ui-dashboard/src/services/api.ts` (add auth API)
- `nextjs-ui-dashboard/src/pages/Settings.tsx` (connect handlers)

## UI Components Needed

- 2FA QR Code modal (show QR + verify input)
- Session list with real data binding
- Loading states for async operations

## Success Criteria

- [ ] Profile name updates persist
- [ ] Password change works end-to-end
- [ ] 2FA QR code displays, verify enables 2FA
- [ ] Sessions list shows real device data
- [ ] Revoke buttons work immediately
