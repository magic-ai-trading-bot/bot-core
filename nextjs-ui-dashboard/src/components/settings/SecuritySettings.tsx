/**
 * SecuritySettings Component
 * @spec:FR-AUTH-012 through FR-AUTH-016 - Account Security Management
 * @ref:specs/02-design/2.5-components/COMP-FRONTEND-DASHBOARD.md
 * @test:TC-AUTH-020 through TC-AUTH-035
 *
 * Password management, 2FA setup, and active sessions.
 * Security-focused with confirmation dialogs for sensitive actions.
 */

import { useState } from 'react';
import { GlassCard } from '@/components/ui/GlassCard';
import { Label } from '@/components/ui/label';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '@/components/ui/alert-dialog';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Shield, Key, Smartphone, LogOut, CheckCircle2, XCircle, Loader2, Copy } from 'lucide-react';
import { useSecurity } from '@/hooks/useSecurity';
import { toast } from 'sonner';

export function SecuritySettings() {
  const {
    sessions,
    twoFactorEnabled,
    setup2FAData,
    isLoading,
    isChangingPassword,
    isSettingUp2FA,
    isLoadingSessions,
    changePassword,
    setup2FA,
    verify2FA,
    disable2FA,
    cancelSetup2FA,
    revokeSession,
    revokeAllSessions,
  } = useSecurity();

  const [showPasswordForm, setShowPasswordForm] = useState(false);
  const [passwordData, setPasswordData] = useState({
    current: '',
    new: '',
    confirm: '',
  });
  const [verificationCode, setVerificationCode] = useState('');
  const [disableCode, setDisableCode] = useState('');
  const [showDisable2FADialog, setShowDisable2FADialog] = useState(false);

  const handleChangePassword = async (e: React.FormEvent) => {
    e.preventDefault();

    if (passwordData.new !== passwordData.confirm) {
      toast.error('New passwords do not match');
      return;
    }

    if (passwordData.new.length < 8) {
      toast.error('Password must be at least 8 characters');
      return;
    }

    const success = await changePassword(passwordData.current, passwordData.new);
    if (success) {
      setShowPasswordForm(false);
      setPasswordData({ current: '', new: '', confirm: '' });
    }
  };

  const handleEnable2FA = async () => {
    await setup2FA();
  };

  const handleVerify2FA = async () => {
    if (verificationCode.length !== 6) {
      toast.error('Please enter a 6-digit code');
      return;
    }
    const success = await verify2FA(verificationCode);
    if (success) {
      setVerificationCode('');
    }
  };

  const handleDisable2FA = async () => {
    if (disableCode.length !== 6) {
      toast.error('Please enter a 6-digit code');
      return;
    }
    const success = await disable2FA(disableCode);
    if (success) {
      setDisableCode('');
      setShowDisable2FADialog(false);
    }
  };

  const handleLogoutSession = async (sessionId: string) => {
    await revokeSession(sessionId);
  };

  const handleLogoutAllSessions = async () => {
    await revokeAllSessions();
  };

  const copySecretToClipboard = () => {
    if (setup2FAData?.secret) {
      navigator.clipboard.writeText(setup2FAData.secret);
      toast.success('Secret copied to clipboard');
    }
  };

  const formatLastActive = (dateStr: string) => {
    const date = new Date(dateStr);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins} minute${diffMins > 1 ? 's' : ''} ago`;
    if (diffHours < 24) return `${diffHours} hour${diffHours > 1 ? 's' : ''} ago`;
    return `${diffDays} day${diffDays > 1 ? 's' : ''} ago`;
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-gray-100">Security</h2>
        <p className="text-sm text-gray-400 mt-1">
          Manage your account security and privacy settings
        </p>
      </div>

      {/* Password Change */}
      <GlassCard>
        <div className="space-y-4">
          <div className="flex items-center gap-2 pb-2 border-b border-slate-700">
            <Key className="w-5 h-5 text-sky-500" />
            <div className="flex-1">
              <Label className="text-gray-100 text-base font-semibold">Password</Label>
              <p className="text-xs text-gray-400 mt-1">
                Change your account password
              </p>
            </div>
            <Button
              onClick={() => setShowPasswordForm(!showPasswordForm)}
              variant="outline"
              size="sm"
              className="border-slate-700"
              disabled={isChangingPassword}
            >
              {showPasswordForm ? 'Cancel' : 'Change Password'}
            </Button>
          </div>

          {showPasswordForm && (
            <form onSubmit={handleChangePassword} className="space-y-4 pt-2">
              <div className="space-y-2">
                <Label htmlFor="currentPassword" className="text-gray-300">
                  Current Password
                </Label>
                <Input
                  id="currentPassword"
                  type="password"
                  value={passwordData.current}
                  onChange={(e) =>
                    setPasswordData({ ...passwordData, current: e.target.value })
                  }
                  className="bg-slate-800 border-slate-700 text-gray-100"
                  required
                  disabled={isChangingPassword}
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="newPassword" className="text-gray-300">
                  New Password
                </Label>
                <Input
                  id="newPassword"
                  type="password"
                  value={passwordData.new}
                  onChange={(e) => setPasswordData({ ...passwordData, new: e.target.value })}
                  className="bg-slate-800 border-slate-700 text-gray-100"
                  required
                  minLength={8}
                  disabled={isChangingPassword}
                />
                <p className="text-xs text-gray-500">
                  At least 8 characters with letters, numbers, and symbols
                </p>
              </div>

              <div className="space-y-2">
                <Label htmlFor="confirmPassword" className="text-gray-300">
                  Confirm New Password
                </Label>
                <Input
                  id="confirmPassword"
                  type="password"
                  value={passwordData.confirm}
                  onChange={(e) =>
                    setPasswordData({ ...passwordData, confirm: e.target.value })
                  }
                  className="bg-slate-800 border-slate-700 text-gray-100"
                  required
                  disabled={isChangingPassword}
                />
              </div>

              <Button
                type="submit"
                className="w-full bg-sky-600 hover:bg-sky-700"
                disabled={isChangingPassword}
              >
                {isChangingPassword ? (
                  <>
                    <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                    Updating...
                  </>
                ) : (
                  'Update Password'
                )}
              </Button>
            </form>
          )}
        </div>
      </GlassCard>

      {/* Two-Factor Authentication */}
      <GlassCard>
        <div className="space-y-4">
          <div className="flex items-center gap-2 pb-2 border-b border-slate-700">
            <Smartphone className="w-5 h-5 text-sky-500" />
            <div className="flex-1">
              <Label className="text-gray-100 text-base font-semibold">
                Two-Factor Authentication
              </Label>
              <p className="text-xs text-gray-400 mt-1">
                Add an extra layer of security to your account
              </p>
            </div>
            {twoFactorEnabled ? (
              <div className="flex items-center gap-2 text-green-500">
                <CheckCircle2 className="w-4 h-4" />
                <span className="text-sm">Enabled</span>
              </div>
            ) : (
              <div className="flex items-center gap-2 text-gray-500">
                <XCircle className="w-4 h-4" />
                <span className="text-sm">Disabled</span>
              </div>
            )}
          </div>

          {!twoFactorEnabled ? (
            <div className="space-y-4">
              <p className="text-sm text-gray-400">
                Secure your account with 2FA using an authenticator app like Google Authenticator
                or Authy.
              </p>
              <Button
                onClick={handleEnable2FA}
                className="w-full bg-green-600 hover:bg-green-700"
                disabled={isSettingUp2FA}
              >
                {isSettingUp2FA ? (
                  <>
                    <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                    Setting up...
                  </>
                ) : (
                  'Enable 2FA'
                )}
              </Button>
            </div>
          ) : (
            <div className="space-y-4">
              <div className="p-4 bg-green-500/10 border border-green-500/30 rounded-lg">
                <p className="text-sm text-green-400">
                  Two-factor authentication is active on your account.
                </p>
              </div>
              <Button
                variant="outline"
                className="w-full border-slate-700 text-gray-300"
                onClick={() => setShowDisable2FADialog(true)}
              >
                Disable 2FA
              </Button>
            </div>
          )}
        </div>
      </GlassCard>

      {/* 2FA Setup Dialog */}
      <Dialog open={!!setup2FAData} onOpenChange={(open) => !open && cancelSetup2FA()}>
        <DialogContent className="bg-slate-900 border-slate-700 max-w-md">
          <DialogHeader>
            <DialogTitle className="text-gray-100">Set Up Two-Factor Authentication</DialogTitle>
            <DialogDescription className="text-gray-400">
              Scan this QR code with your authenticator app, then enter the verification code.
            </DialogDescription>
          </DialogHeader>

          {setup2FAData && (
            <div className="space-y-4">
              {/* QR Code */}
              <div className="flex justify-center p-4 bg-white rounded-lg">
                <img
                  src={`data:image/png;base64,${setup2FAData.qr_code}`}
                  alt="2FA QR Code"
                  className="w-48 h-48"
                />
              </div>

              {/* Manual Entry Secret */}
              <div className="space-y-2">
                <Label className="text-gray-300">Manual Entry Code</Label>
                <div className="flex gap-2">
                  <Input
                    readOnly
                    value={setup2FAData.secret}
                    className="bg-slate-800 border-slate-700 text-gray-100 font-mono text-sm"
                  />
                  <Button
                    variant="outline"
                    size="icon"
                    className="border-slate-700"
                    onClick={copySecretToClipboard}
                  >
                    <Copy className="w-4 h-4" />
                  </Button>
                </div>
                <p className="text-xs text-gray-500">
                  If you can't scan the QR code, enter this secret manually in your authenticator app.
                </p>
              </div>

              {/* Verification Code Input */}
              <div className="space-y-2">
                <Label htmlFor="verificationCode" className="text-gray-300">
                  Verification Code
                </Label>
                <Input
                  id="verificationCode"
                  type="text"
                  inputMode="numeric"
                  pattern="[0-9]*"
                  maxLength={6}
                  placeholder="000000"
                  value={verificationCode}
                  onChange={(e) => setVerificationCode(e.target.value.replace(/\D/g, ''))}
                  className="bg-slate-800 border-slate-700 text-gray-100 text-center text-2xl tracking-widest"
                />
              </div>
            </div>
          )}

          <DialogFooter>
            <Button variant="outline" className="border-slate-700" onClick={cancelSetup2FA}>
              Cancel
            </Button>
            <Button
              className="bg-green-600 hover:bg-green-700"
              onClick={handleVerify2FA}
              disabled={isLoading || verificationCode.length !== 6}
            >
              {isLoading ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  Verifying...
                </>
              ) : (
                'Enable 2FA'
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Disable 2FA Dialog */}
      <Dialog open={showDisable2FADialog} onOpenChange={setShowDisable2FADialog}>
        <DialogContent className="bg-slate-900 border-slate-700">
          <DialogHeader>
            <DialogTitle className="text-gray-100">Disable Two-Factor Authentication?</DialogTitle>
            <DialogDescription className="text-gray-400">
              This will make your account less secure. Enter your 2FA code to confirm.
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-2">
            <Label htmlFor="disableCode" className="text-gray-300">
              Current 2FA Code
            </Label>
            <Input
              id="disableCode"
              type="text"
              inputMode="numeric"
              pattern="[0-9]*"
              maxLength={6}
              placeholder="000000"
              value={disableCode}
              onChange={(e) => setDisableCode(e.target.value.replace(/\D/g, ''))}
              className="bg-slate-800 border-slate-700 text-gray-100 text-center text-2xl tracking-widest"
            />
          </div>

          <DialogFooter>
            <Button
              variant="outline"
              className="border-slate-700"
              onClick={() => {
                setShowDisable2FADialog(false);
                setDisableCode('');
              }}
            >
              Cancel
            </Button>
            <Button
              className="bg-red-600 hover:bg-red-700"
              onClick={handleDisable2FA}
              disabled={isLoading || disableCode.length !== 6}
            >
              {isLoading ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  Disabling...
                </>
              ) : (
                'Disable 2FA'
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Active Sessions */}
      <GlassCard>
        <div className="space-y-4">
          <div className="flex items-center justify-between pb-2 border-b border-slate-700">
            <div className="flex items-center gap-2">
              <Shield className="w-5 h-5 text-sky-500" />
              <div>
                <Label className="text-gray-100 text-base font-semibold">Active Sessions</Label>
                <p className="text-xs text-gray-400 mt-1">
                  Manage devices currently signed into your account
                </p>
              </div>
            </div>
            <AlertDialog>
              <AlertDialogTrigger asChild>
                <Button
                  variant="outline"
                  size="sm"
                  className="gap-2 border-slate-700 text-gray-300"
                  disabled={sessions.length <= 1}
                >
                  <LogOut className="w-4 h-4" />
                  Log Out Everywhere
                </Button>
              </AlertDialogTrigger>
              <AlertDialogContent className="bg-slate-900 border-slate-700">
                <AlertDialogHeader>
                  <AlertDialogTitle className="text-gray-100">
                    Log Out All Sessions?
                  </AlertDialogTitle>
                  <AlertDialogDescription className="text-gray-400">
                    This will log you out from all devices except this one.
                  </AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                  <AlertDialogCancel className="border-slate-700">Cancel</AlertDialogCancel>
                  <AlertDialogAction
                    onClick={handleLogoutAllSessions}
                    className="bg-red-600 hover:bg-red-700"
                  >
                    Log Out All
                  </AlertDialogAction>
                </AlertDialogFooter>
              </AlertDialogContent>
            </AlertDialog>
          </div>

          <div className="space-y-3">
            {isLoadingSessions ? (
              <div className="flex items-center justify-center py-8">
                <Loader2 className="w-6 h-6 text-gray-400 animate-spin" />
              </div>
            ) : sessions.length === 0 ? (
              <p className="text-center text-gray-500 py-4">No active sessions found</p>
            ) : (
              sessions.map((session) => (
                <div
                  key={session.session_id}
                  className="flex items-start justify-between p-4 bg-slate-800/50 rounded-lg border border-slate-700/50"
                >
                  <div className="flex-1">
                    <div className="flex items-center gap-2">
                      <p className="font-medium text-gray-100">
                        {session.browser} on {session.os}
                      </p>
                      {session.is_current && (
                        <span className="px-2 py-0.5 bg-green-500/20 text-green-400 text-xs rounded">
                          Current
                        </span>
                      )}
                    </div>
                    <p className="text-sm text-gray-400 mt-1">
                      {session.location} • {session.ip_address}
                    </p>
                    <p className="text-xs text-gray-500 mt-1">
                      {session.device} • Last active: {formatLastActive(session.last_active)}
                    </p>
                  </div>
                  {!session.is_current && (
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => handleLogoutSession(session.session_id)}
                      className="text-red-400 hover:text-red-300 hover:bg-red-500/10"
                    >
                      <LogOut className="w-4 h-4" />
                    </Button>
                  )}
                </div>
              ))
            )}
          </div>
        </div>
      </GlassCard>
    </div>
  );
}
