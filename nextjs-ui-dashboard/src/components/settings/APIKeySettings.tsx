/**
 * APIKeySettings Component
 *
 * Manage Binance API keys with secure backend storage.
 * Keys are encrypted and stored in the database.
 */

import { useState, useEffect, useCallback } from 'react';
import { GlassCard } from '@/components/ui/GlassCard';
import { Label } from '@/components/ui/label';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Switch } from '@/components/ui/switch';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import { Key, Plus, Trash2, CheckCircle2, XCircle, Loader2, RefreshCw, Shield } from 'lucide-react';

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080';

interface ApiKeyStatus {
  configured: boolean;
  api_key_masked: string | null;
  use_testnet: boolean;
  permissions: {
    spot_trading: boolean;
    futures_trading: boolean;
    margin_trading: boolean;
    options_trading: boolean;
  } | null;
  last_updated: string | null;
  connection_status: {
    connected: boolean;
    message: string;
    account_type: string | null;
    can_trade: boolean | null;
    balances_count: number | null;
  } | null;
}

interface SaveApiKeysRequest {
  api_key: string;
  api_secret: string;
  use_testnet: boolean;
  permissions: {
    spot_trading: boolean;
    futures_trading: boolean;
    margin_trading: boolean;
    options_trading: boolean;
  };
}

export function APIKeySettings() {
  const [apiKeyStatus, setApiKeyStatus] = useState<ApiKeyStatus | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [isTesting, setIsTesting] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);
  const [isAddDialogOpen, setIsAddDialogOpen] = useState(false);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [connectionStatus, setConnectionStatus] = useState<{
    connected: boolean;
    message: string;
    account_type?: string;
    can_trade?: boolean;
    balances_count?: number;
  } | null>(null);

  // Fetch API key status from backend
  const fetchApiKeyStatus = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const response = await fetch(`${API_BASE_URL}/api/settings/api-keys`);
      const data = await response.json();

      if (data.success) {
        setApiKeyStatus(data.data);
      } else {
        setError(data.error || 'Failed to fetch API key status');
      }
    } catch (err) {
      setError('Failed to connect to server');
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchApiKeyStatus();
  }, [fetchApiKeyStatus]);

  // Save API keys to backend
  const handleSaveApiKeys = async (formData: SaveApiKeysRequest) => {
    setIsSaving(true);
    setError(null);
    setSuccessMessage(null);

    try {
      const response = await fetch(`${API_BASE_URL}/api/settings/api-keys`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(formData),
      });

      const data = await response.json();

      if (data.success) {
        setSuccessMessage('API keys saved successfully! Keys are encrypted and stored securely.');
        setIsAddDialogOpen(false);
        await fetchApiKeyStatus();
        // Auto-test connection after saving
        setTimeout(() => handleTestConnection(), 500);
      } else {
        setError(data.error || 'Failed to save API keys');
      }
    } catch (err) {
      setError('Failed to save API keys. Please try again.');
    } finally {
      setIsSaving(false);
    }
  };

  // Test connection with Binance
  const handleTestConnection = async () => {
    setIsTesting(true);
    setError(null);
    setConnectionStatus(null);

    try {
      const response = await fetch(`${API_BASE_URL}/api/settings/api-keys/test`, {
        method: 'POST',
      });

      const data = await response.json();

      if (data.success) {
        setConnectionStatus(data.data);
        if (data.data.connected) {
          setSuccessMessage(`Connected to Binance ${data.data.account_type}!`);
        }
      } else {
        setError(data.error || 'Connection test failed');
      }
    } catch (err) {
      setError('Failed to test connection. Please try again.');
    } finally {
      setIsTesting(false);
    }
  };

  // Delete API keys
  const handleDeleteApiKeys = async () => {
    setIsDeleting(true);
    setError(null);

    try {
      const response = await fetch(`${API_BASE_URL}/api/settings/api-keys`, {
        method: 'DELETE',
      });

      const data = await response.json();

      if (data.success) {
        setSuccessMessage('API keys deleted successfully');
        setConnectionStatus(null);
        await fetchApiKeyStatus();
      } else {
        setError(data.error || 'Failed to delete API keys');
      }
    } catch (err) {
      setError('Failed to delete API keys. Please try again.');
    } finally {
      setIsDeleting(false);
      setShowDeleteConfirm(false);
    }
  };

  // Clear messages after 5 seconds
  useEffect(() => {
    if (successMessage) {
      const timer = setTimeout(() => setSuccessMessage(null), 5000);
      return () => clearTimeout(timer);
    }
  }, [successMessage]);

  useEffect(() => {
    if (error) {
      const timer = setTimeout(() => setError(null), 8000);
      return () => clearTimeout(timer);
    }
  }, [error]);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="w-8 h-8 animate-spin text-sky-500" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-100">Binance API Keys</h2>
          <p className="text-sm text-gray-400 mt-1">
            Configure your Binance API credentials for real trading
          </p>
        </div>
        {!apiKeyStatus?.configured && (
          <Dialog open={isAddDialogOpen} onOpenChange={setIsAddDialogOpen}>
            <DialogTrigger asChild>
              <Button className="gap-2 bg-sky-600 hover:bg-sky-700">
                <Plus className="w-4 h-4" />
                Add API Key
              </Button>
            </DialogTrigger>
            <DialogContent className="sm:max-w-[500px] bg-slate-900 border-slate-700">
              <AddAPIKeyDialog onAdd={handleSaveApiKeys} isSaving={isSaving} />
            </DialogContent>
          </Dialog>
        )}
      </div>

      {/* Success Message */}
      {successMessage && (
        <GlassCard className="border-green-500/50 bg-green-950/20">
          <div className="flex items-center gap-2 text-green-400">
            <CheckCircle2 className="w-5 h-5" />
            <span>{successMessage}</span>
          </div>
        </GlassCard>
      )}

      {/* Error Message */}
      {error && (
        <GlassCard className="border-red-500/50 bg-red-950/20">
          <div className="flex items-center gap-2 text-red-400">
            <XCircle className="w-5 h-5" />
            <span>{error}</span>
          </div>
        </GlassCard>
      )}

      {/* No API Key Configured */}
      {!apiKeyStatus?.configured ? (
        <GlassCard className="text-center py-12">
          <Key className="w-12 h-12 text-gray-600 mx-auto mb-4" />
          <p className="text-gray-400">No API keys configured</p>
          <p className="text-sm text-gray-500 mt-1">
            Add your Binance API key to enable real trading
          </p>
          <div className="mt-6 p-4 bg-slate-800/50 rounded-lg text-left max-w-md mx-auto">
            <h4 className="text-sm font-semibold text-gray-300 mb-2">How to get API keys:</h4>
            <ol className="text-xs text-gray-400 space-y-1 list-decimal list-inside">
              <li>Log in to your Binance account</li>
              <li>Go to Account → API Management</li>
              <li>Create a new API key</li>
              <li>Enable "Futures Trading" permission</li>
              <li>Copy and paste your keys here</li>
            </ol>
          </div>
        </GlassCard>
      ) : (
        /* API Key Configured */
        <GlassCard className="hover:border-sky-500/30 transition-colors">
          <div className="space-y-4">
            {/* Header */}
            <div className="flex items-start justify-between">
              <div className="flex items-center gap-3">
                <div className="p-2 bg-sky-500/20 rounded-lg">
                  <Shield className="w-6 h-6 text-sky-400" />
                </div>
                <div>
                  <h3 className="font-semibold text-gray-100">Binance API</h3>
                  <p className="text-xs text-gray-400 mt-0.5">
                    {apiKeyStatus.use_testnet ? 'Testnet Mode' : 'Mainnet Mode'} •
                    Last updated: {apiKeyStatus.last_updated ? new Date(apiKeyStatus.last_updated).toLocaleDateString() : 'Unknown'}
                  </p>
                </div>
              </div>
              <Button
                size="sm"
                variant="destructive"
                onClick={() => setShowDeleteConfirm(true)}
                className="gap-2"
                disabled={isDeleting}
              >
                {isDeleting ? <Loader2 className="w-4 h-4 animate-spin" /> : <Trash2 className="w-4 h-4" />}
                Delete
              </Button>
            </div>

            {/* API Key (Masked) */}
            <div>
              <Label className="text-gray-300 text-xs">API Key</Label>
              <div className="mt-1 p-2 bg-slate-800/50 rounded text-sm text-gray-100 font-mono">
                {apiKeyStatus.api_key_masked || '••••••••'}
              </div>
            </div>

            {/* API Secret (Always Masked) */}
            <div>
              <Label className="text-gray-300 text-xs">API Secret</Label>
              <div className="mt-1 p-2 bg-slate-800/50 rounded text-sm text-gray-400 font-mono">
                ••••••••••••••••••••••••••••
              </div>
              <p className="text-xs text-gray-500 mt-1">
                Secret is encrypted and never displayed for security
              </p>
            </div>

            {/* Permissions */}
            {apiKeyStatus.permissions && (
              <div>
                <Label className="text-gray-300 text-xs">Permissions</Label>
                <div className="flex flex-wrap gap-2 mt-2">
                  {apiKeyStatus.permissions.spot_trading && (
                    <span className="px-2 py-1 bg-blue-500/20 text-blue-400 text-xs rounded">
                      Spot Trading
                    </span>
                  )}
                  {apiKeyStatus.permissions.futures_trading && (
                    <span className="px-2 py-1 bg-green-500/20 text-green-400 text-xs rounded">
                      Futures Trading
                    </span>
                  )}
                  {apiKeyStatus.permissions.margin_trading && (
                    <span className="px-2 py-1 bg-yellow-500/20 text-yellow-400 text-xs rounded">
                      Margin Trading
                    </span>
                  )}
                  {apiKeyStatus.permissions.options_trading && (
                    <span className="px-2 py-1 bg-purple-500/20 text-purple-400 text-xs rounded">
                      Options Trading
                    </span>
                  )}
                </div>
              </div>
            )}

            {/* Connection Status */}
            {connectionStatus && (
              <div className={`p-3 rounded-lg ${connectionStatus.connected ? 'bg-green-950/30 border border-green-500/30' : 'bg-red-950/30 border border-red-500/30'}`}>
                <div className="flex items-center gap-2">
                  {connectionStatus.connected ? (
                    <CheckCircle2 className="w-5 h-5 text-green-400" />
                  ) : (
                    <XCircle className="w-5 h-5 text-red-400" />
                  )}
                  <span className={connectionStatus.connected ? 'text-green-400' : 'text-red-400'}>
                    {connectionStatus.message}
                  </span>
                </div>
                {connectionStatus.connected && connectionStatus.balances_count !== undefined && (
                  <p className="text-xs text-gray-400 mt-2 ml-7">
                    Account Type: {connectionStatus.account_type} •
                    Can Trade: {connectionStatus.can_trade ? 'Yes' : 'No'} •
                    Assets: {connectionStatus.balances_count}
                  </p>
                )}
              </div>
            )}

            {/* Test Connection Button */}
            <div className="flex gap-3 pt-2">
              <Button
                onClick={handleTestConnection}
                disabled={isTesting}
                className="gap-2 bg-sky-600 hover:bg-sky-700"
              >
                {isTesting ? (
                  <Loader2 className="w-4 h-4 animate-spin" />
                ) : (
                  <RefreshCw className="w-4 h-4" />
                )}
                Test Connection
              </Button>
              <Dialog open={isAddDialogOpen} onOpenChange={setIsAddDialogOpen}>
                <DialogTrigger asChild>
                  <Button variant="outline" className="gap-2 border-slate-700">
                    <Key className="w-4 h-4" />
                    Update Keys
                  </Button>
                </DialogTrigger>
                <DialogContent className="sm:max-w-[500px] bg-slate-900 border-slate-700">
                  <AddAPIKeyDialog
                    onAdd={handleSaveApiKeys}
                    isSaving={isSaving}
                    defaultTestnet={apiKeyStatus.use_testnet}
                  />
                </DialogContent>
              </Dialog>
            </div>
          </div>
        </GlassCard>
      )}

      {/* Security Notice */}
      <GlassCard className="border-slate-700/50 bg-slate-900/50">
        <div className="flex items-start gap-3">
          <Shield className="w-5 h-5 text-sky-400 mt-0.5" />
          <div>
            <h4 className="text-sm font-semibold text-gray-300">Security Information</h4>
            <ul className="text-xs text-gray-400 mt-2 space-y-1">
              <li>• Your API secret is encrypted with AES-256-GCM before storage</li>
              <li>• Only the masked API key is displayed for verification</li>
              <li>• Use API keys with only necessary permissions enabled</li>
              <li>• Never enable withdrawal permissions for trading bots</li>
              <li>• We recommend using Testnet keys for testing first</li>
            </ul>
          </div>
        </div>
      </GlassCard>

      {/* Delete Confirmation Dialog */}
      <AlertDialog open={showDeleteConfirm} onOpenChange={setShowDeleteConfirm}>
        <AlertDialogContent className="bg-slate-900 border-slate-700">
          <AlertDialogHeader>
            <AlertDialogTitle className="text-gray-100">Delete API Keys?</AlertDialogTitle>
            <AlertDialogDescription className="text-gray-400">
              This will permanently delete your stored API credentials. You'll need to re-enter them to enable real trading again.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel className="border-slate-700">Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleDeleteApiKeys}
              className="bg-red-600 hover:bg-red-700"
              disabled={isDeleting}
            >
              {isDeleting ? <Loader2 className="w-4 h-4 animate-spin mr-2" /> : null}
              Delete Keys
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}

// Add API Key Dialog Form
function AddAPIKeyDialog({
  onAdd,
  isSaving,
  defaultTestnet = true
}: {
  onAdd: (data: SaveApiKeysRequest) => void;
  isSaving: boolean;
  defaultTestnet?: boolean;
}) {
  const [formData, setFormData] = useState({
    api_key: '',
    api_secret: '',
    use_testnet: defaultTestnet,
    permissions: {
      spot_trading: false,
      futures_trading: true,
      margin_trading: false,
      options_trading: false,
    },
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onAdd(formData);
  };

  return (
    <form onSubmit={handleSubmit}>
      <DialogHeader>
        <DialogTitle className="text-gray-100">Add Binance API Key</DialogTitle>
        <DialogDescription className="text-gray-400">
          Enter your Binance API credentials. They will be encrypted and stored securely.
        </DialogDescription>
      </DialogHeader>

      <div className="space-y-4 py-4">
        <div className="space-y-2">
          <Label htmlFor="apiKey" className="text-gray-300">
            API Key
          </Label>
          <Input
            id="apiKey"
            placeholder="Enter your Binance API key"
            value={formData.api_key}
            onChange={(e) => setFormData({ ...formData, api_key: e.target.value })}
            className="bg-slate-800 border-slate-700 text-gray-100 font-mono text-sm"
            required
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="apiSecret" className="text-gray-300">
            API Secret
          </Label>
          <Input
            id="apiSecret"
            type="password"
            placeholder="Enter your Binance API secret"
            value={formData.api_secret}
            onChange={(e) => setFormData({ ...formData, api_secret: e.target.value })}
            className="bg-slate-800 border-slate-700 text-gray-100 font-mono text-sm"
            required
          />
        </div>

        <div className="flex items-center justify-between p-3 bg-slate-800/50 rounded-lg">
          <div>
            <Label htmlFor="testnet" className="text-gray-300">
              Use Testnet
            </Label>
            <p className="text-xs text-gray-500">
              Recommended for testing. Uses test funds only.
            </p>
          </div>
          <Switch
            id="testnet"
            checked={formData.use_testnet}
            onCheckedChange={(checked) =>
              setFormData({ ...formData, use_testnet: checked })
            }
          />
        </div>

        <div className="space-y-3 pt-2">
          <Label className="text-gray-300">Trading Permissions</Label>
          <p className="text-xs text-gray-500">
            Select the permissions enabled on your Binance API key
          </p>

          <div className="flex items-center justify-between">
            <div>
              <Label htmlFor="perm-spot" className="text-gray-300">
                Spot Trading
              </Label>
            </div>
            <Switch
              id="perm-spot"
              checked={formData.permissions.spot_trading}
              onCheckedChange={(checked) =>
                setFormData({
                  ...formData,
                  permissions: { ...formData.permissions, spot_trading: checked },
                })
              }
            />
          </div>

          <div className="flex items-center justify-between">
            <div>
              <Label htmlFor="perm-futures" className="text-gray-300">
                Futures Trading
              </Label>
              <p className="text-xs text-green-500">Required for this bot</p>
            </div>
            <Switch
              id="perm-futures"
              checked={formData.permissions.futures_trading}
              onCheckedChange={(checked) =>
                setFormData({
                  ...formData,
                  permissions: { ...formData.permissions, futures_trading: checked },
                })
              }
            />
          </div>

          <div className="flex items-center justify-between">
            <div>
              <Label htmlFor="perm-margin" className="text-gray-300">
                Margin Trading
              </Label>
            </div>
            <Switch
              id="perm-margin"
              checked={formData.permissions.margin_trading}
              onCheckedChange={(checked) =>
                setFormData({
                  ...formData,
                  permissions: { ...formData.permissions, margin_trading: checked },
                })
              }
            />
          </div>
        </div>
      </div>

      <DialogFooter>
        <Button
          type="submit"
          className="bg-sky-600 hover:bg-sky-700 gap-2"
          disabled={isSaving || !formData.api_key || !formData.api_secret}
        >
          {isSaving ? (
            <>
              <Loader2 className="w-4 h-4 animate-spin" />
              Saving...
            </>
          ) : (
            <>
              <Key className="w-4 h-4" />
              Save API Key
            </>
          )}
        </Button>
      </DialogFooter>
    </form>
  );
}
