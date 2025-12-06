// @spec:FR-AUTH-012 through FR-AUTH-016 - Account Security Management Hook
// @ref:specs/02-design/2.5-components/COMP-FRONTEND-DASHBOARD.md
// @test:TC-AUTH-020 through TC-AUTH-035

import { useState, useCallback, useEffect } from 'react';
import { apiClient, SessionInfo, Setup2FAResponse } from '@/services/api';
import { useAuth } from '@/contexts/AuthContext';
import { toast } from 'sonner';
import logger from '@/utils/logger';

interface SecurityState {
  sessions: SessionInfo[];
  twoFactorEnabled: boolean;
  setup2FAData: Setup2FAResponse | null;
  isLoading: boolean;
  isChangingPassword: boolean;
  isSettingUp2FA: boolean;
  isLoadingSessions: boolean;
}

interface UseSecurity {
  // State
  sessions: SessionInfo[];
  twoFactorEnabled: boolean;
  setup2FAData: Setup2FAResponse | null;
  isLoading: boolean;
  isChangingPassword: boolean;
  isSettingUp2FA: boolean;
  isLoadingSessions: boolean;

  // Actions
  changePassword: (currentPassword: string, newPassword: string) => Promise<boolean>;
  updateDisplayName: (displayName: string) => Promise<boolean>;
  setup2FA: () => Promise<boolean>;
  verify2FA: (code: string) => Promise<boolean>;
  disable2FA: (code: string) => Promise<boolean>;
  cancelSetup2FA: () => void;
  loadSessions: () => Promise<void>;
  revokeSession: (sessionId: string) => Promise<boolean>;
  revokeAllSessions: () => Promise<boolean>;
  refreshUserProfile: () => Promise<void>;
}

export function useSecurity(): UseSecurity {
  const { user, refreshUser } = useAuth();

  const [state, setState] = useState<SecurityState>({
    sessions: [],
    twoFactorEnabled: user?.two_factor_enabled ?? false,
    setup2FAData: null,
    isLoading: false,
    isChangingPassword: false,
    isSettingUp2FA: false,
    isLoadingSessions: false,
  });

  // Update 2FA state when user changes
  useEffect(() => {
    if (user) {
      setState(prev => ({
        ...prev,
        twoFactorEnabled: user.two_factor_enabled ?? false,
      }));
    }
  }, [user]);

  // Load sessions on mount
  useEffect(() => {
    loadSessions();
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const loadSessions = useCallback(async () => {
    setState(prev => ({ ...prev, isLoadingSessions: true }));
    try {
      const sessions = await apiClient.auth.getSessions();
      setState(prev => ({ ...prev, sessions, isLoadingSessions: false }));
    } catch (error) {
      logger.error('Failed to load sessions:', error);
      setState(prev => ({ ...prev, sessions: [], isLoadingSessions: false }));
      // Don't show error toast for session loading - it's background data
    }
  }, []);

  const refreshUserProfile = useCallback(async () => {
    try {
      await refreshUser();
    } catch (error) {
      logger.error('Failed to refresh user profile:', error);
    }
  }, [refreshUser]);

  const changePassword = useCallback(async (currentPassword: string, newPassword: string): Promise<boolean> => {
    setState(prev => ({ ...prev, isChangingPassword: true }));
    try {
      await apiClient.auth.changePassword({
        current_password: currentPassword,
        new_password: newPassword,
      });
      toast.success('Password changed successfully');
      setState(prev => ({ ...prev, isChangingPassword: false }));
      return true;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to change password';
      toast.error(message);
      setState(prev => ({ ...prev, isChangingPassword: false }));
      return false;
    }
  }, []);

  const updateDisplayName = useCallback(async (displayName: string): Promise<boolean> => {
    setState(prev => ({ ...prev, isLoading: true }));
    try {
      await apiClient.auth.updateProfile({ display_name: displayName });
      await refreshUserProfile();
      toast.success('Profile updated successfully');
      setState(prev => ({ ...prev, isLoading: false }));
      return true;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to update profile';
      toast.error(message);
      setState(prev => ({ ...prev, isLoading: false }));
      return false;
    }
  }, [refreshUserProfile]);

  const setup2FA = useCallback(async (): Promise<boolean> => {
    setState(prev => ({ ...prev, isSettingUp2FA: true }));
    try {
      const data = await apiClient.auth.setup2FA();
      setState(prev => ({ ...prev, setup2FAData: data, isSettingUp2FA: false }));
      return true;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to setup 2FA';
      toast.error(message);
      setState(prev => ({ ...prev, isSettingUp2FA: false }));
      return false;
    }
  }, []);

  const verify2FA = useCallback(async (code: string): Promise<boolean> => {
    setState(prev => ({ ...prev, isLoading: true }));
    try {
      await apiClient.auth.verify2FA({ code });
      await refreshUserProfile();
      toast.success('Two-factor authentication enabled');
      setState(prev => ({
        ...prev,
        twoFactorEnabled: true,
        setup2FAData: null,
        isLoading: false,
      }));
      return true;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Invalid verification code';
      toast.error(message);
      setState(prev => ({ ...prev, isLoading: false }));
      return false;
    }
  }, [refreshUserProfile]);

  const disable2FA = useCallback(async (code: string): Promise<boolean> => {
    setState(prev => ({ ...prev, isLoading: true }));
    try {
      await apiClient.auth.disable2FA({ code });
      await refreshUserProfile();
      toast.success('Two-factor authentication disabled');
      setState(prev => ({
        ...prev,
        twoFactorEnabled: false,
        isLoading: false,
      }));
      return true;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Invalid verification code';
      toast.error(message);
      setState(prev => ({ ...prev, isLoading: false }));
      return false;
    }
  }, [refreshUserProfile]);

  const cancelSetup2FA = useCallback(() => {
    setState(prev => ({ ...prev, setup2FAData: null }));
  }, []);

  const revokeSession = useCallback(async (sessionId: string): Promise<boolean> => {
    try {
      await apiClient.auth.revokeSession(sessionId);
      setState(prev => ({
        ...prev,
        sessions: prev.sessions.filter(s => s.session_id !== sessionId),
      }));
      toast.success('Session revoked');
      return true;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to revoke session';
      toast.error(message);
      return false;
    }
  }, []);

  const revokeAllSessions = useCallback(async (): Promise<boolean> => {
    setState(prev => ({ ...prev, isLoading: true }));
    try {
      const result = await apiClient.auth.revokeAllSessions();
      toast.success(`${result.revoked_count} sessions revoked`);
      // Reload sessions to show only current session
      await loadSessions();
      setState(prev => ({ ...prev, isLoading: false }));
      return true;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to revoke sessions';
      toast.error(message);
      setState(prev => ({ ...prev, isLoading: false }));
      return false;
    }
  }, [loadSessions]);

  return {
    // State
    sessions: state.sessions,
    twoFactorEnabled: state.twoFactorEnabled,
    setup2FAData: state.setup2FAData,
    isLoading: state.isLoading,
    isChangingPassword: state.isChangingPassword,
    isSettingUp2FA: state.isSettingUp2FA,
    isLoadingSessions: state.isLoadingSessions,

    // Actions
    changePassword,
    updateDisplayName,
    setup2FA,
    verify2FA,
    disable2FA,
    cancelSetup2FA,
    loadSessions,
    revokeSession,
    revokeAllSessions,
    refreshUserProfile,
  };
}
