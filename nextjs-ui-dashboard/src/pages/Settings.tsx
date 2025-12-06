import ErrorBoundary from "@/components/ErrorBoundary";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Slider } from "@/components/ui/slider";
import { useState, useEffect } from "react";
import { usePaperTradingContext } from "@/contexts/PaperTradingContext";
import { useToast } from "@/hooks/use-toast";
import { useSecurity } from "@/hooks/useSecurity";
import { useNotificationPreferences, localToApiFormat, NotificationCredentials } from "@/hooks/useNotificationPreferences";
import { usePushNotifications } from "@/hooks/usePushNotifications";
import { toast } from "sonner";
import ChatBot from "@/components/ChatBot";
import { motion, AnimatePresence } from "framer-motion";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
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
} from "@/components/ui/alert-dialog";
import { Copy } from "lucide-react";
import { TradingSettings } from "@/components/dashboard/TradingSettings";
import {
  Settings2,
  TrendingUp,
  Shield,
  Bell,
  Key,
  Wifi,
  WifiOff,
  Check,
  Loader2,
  ChevronRight,
  Zap,
  AlertTriangle,
  Lock,
  Smartphone,
  Mail,
  MessageCircle,
  Volume2,
  Eye,
  EyeOff,
  User,
  LogOut,
  RefreshCw,
  Save,
  Activity,
  BarChart3,
  Cpu,
  HardDrive,
  Clock,
  Server,
  Database,
  Coins,
} from "lucide-react";
import logger from "@/utils/logger";
import {
  luxuryColors,
  GlassCard,
  GradientText,
  PremiumButton,
  PremiumInput,
  Badge,
  GlowIcon,
  SectionHeader,
  PageWrapper,
} from "@/styles/luxury-design-system";

// API Base URL
const API_BASE = import.meta.env.VITE_RUST_API_URL || "http://localhost:8080";

// Section type definition - matching old UI tabs
type SettingsSection = "bot" | "per-symbol" | "strategy" | "health" | "api" | "notifications" | "security";

// Trading pair configuration
interface TradingPair {
  symbol: string;
  enabled: boolean;
  leverage?: number;
  positionSize?: number;
}

// System health metrics
interface SystemHealth {
  cpu: number;
  memory: number;
  uptime: string;
  apiLatency: number;
  wsConnected: boolean;
  dbConnected: boolean;
  lastUpdate: string;
}

/**
 * Premium Settings Page - Dark OLED Luxury Design
 *
 * Reorganized into 7 tabs (matching old UI):
 * 1. Bot Settings - Basic bot configuration
 * 2. Per-Symbol - Symbol-specific settings
 * 3. Strategy Tuning - Opens TradingSettings modal
 * 4. System Health - System status monitoring
 * 5. API Keys - API configuration
 * 6. Notifications - Alert settings
 * 7. Security - Account security
 *
 * @spec:FR-DASHBOARD-004 - Bot Settings UI
 * @ref:specs/02-design/2.5-components/COMP-FRONTEND-DASHBOARD.md
 */
const Settings = () => {
  const { portfolio, settings, updateSettings, startBot, stopBot } = usePaperTradingContext();
  const { toast } = useToast();

  // Active section state
  const [activeSection, setActiveSection] = useState<SettingsSection>("bot");

  // API Keys state
  const [apiKey, setApiKey] = useState("************************************1234");
  const [secretKey, setSecretKey] = useState("************************************5678");
  const [showApiKey, setShowApiKey] = useState(false);
  const [showSecretKey, setShowSecretKey] = useState(false);
  const [isTestingConnection, setIsTestingConnection] = useState(false);
  const [connectionStatus, setConnectionStatus] = useState<"connected" | "disconnected" | "testing">("connected");

  // Bot Settings state
  const [botActive, setBotActive] = useState(settings?.basic?.enabled || false);
  const [capitalAllocation, setCapitalAllocation] = useState([settings?.basic?.default_position_size_pct || 2]);
  const [maxLeverage, setMaxLeverage] = useState([settings?.basic?.default_leverage || 3]);
  const [riskThreshold, setRiskThreshold] = useState([settings?.risk?.max_risk_per_trade_pct || 2]);

  // Trading Pairs state
  const [tradingPairs, setTradingPairs] = useState<TradingPair[]>([
    { symbol: "BTC/USDT", enabled: true, leverage: 3, positionSize: 30 },
    { symbol: "ETH/USDT", enabled: true, leverage: 3, positionSize: 25 },
    { symbol: "BNB/USDT", enabled: false, leverage: 2, positionSize: 15 },
    { symbol: "SOL/USDT", enabled: false, leverage: 2, positionSize: 15 },
    { symbol: "XRP/USDT", enabled: false, leverage: 2, positionSize: 10 },
    { symbol: "ADA/USDT", enabled: false, leverage: 2, positionSize: 5 },
  ]);

  // System Health state - values updated from real API health checks
  const [systemHealth, setSystemHealth] = useState<SystemHealth>({
    cpu: 0,  // Not available from API - removed fake data
    memory: 0,  // Not available from API - removed fake data
    uptime: "--",  // Will be updated from API if available
    apiLatency: 0,  // Measured from real API calls
    wsConnected: false,  // Updated from real health check
    dbConnected: false,  // Updated from real health check
    lastUpdate: new Date().toLocaleTimeString(),
  });

  // Notification settings
  const [notifications, setNotifications] = useState({
    email: true,
    push: false,
    telegram: true,
    discord: false,
    sound: true,
    priceAlerts: true,
    tradeAlerts: true,
    systemAlerts: true,
  });
  const [telegramToken, setTelegramToken] = useState("");
  const [telegramChatId, setTelegramChatId] = useState("");
  const [discordWebhookUrl, setDiscordWebhookUrl] = useState("");
  const [vapidPublicKey, setVapidPublicKey] = useState("");
  const [vapidPrivateKey, setVapidPrivateKey] = useState("");
  const [alertThreshold, setAlertThreshold] = useState([5]);

  // Security settings - using real API hook
  const {
    sessions,
    twoFactorEnabled,
    setup2FAData,
    isLoading: isSecurityLoading,
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

  // Notification preferences - using real API hook
  const {
    preferences: apiNotificationPrefs,
    isLoading: isLoadingNotifications,
    isSaving: isSavingNotifications,
    error: notificationError,
    savePreferences: saveNotificationPreferences,
    testNotification,
  } = useNotificationPreferences();

  // Push notifications - Service Worker integration
  const {
    isSupported: isPushSupported,
    isSubscribed: isPushSubscribed,
    isLoading: isPushLoading,
    permission: pushPermission,
    error: pushError,
    subscribe: subscribeToPush,
    unsubscribe: unsubscribeFromPush,
    showLocalNotification,
  } = usePushNotifications();

  // Security dialog states
  const [showDisable2FADialog, setShowDisable2FADialog] = useState(false);
  const [verificationCode, setVerificationCode] = useState('');
  const [disableCode, setDisableCode] = useState('');
  const [passwordData, setPasswordData] = useState({
    current: '',
    new: '',
    confirm: '',
  });

  // Loading states
  const [isSaving, setIsSaving] = useState(false);
  const [savedSection, setSavedSection] = useState<SettingsSection | null>(null);

  // Sync with backend settings
  useEffect(() => {
    if (settings?.basic) {
      setBotActive(settings.basic.enabled);
      setMaxLeverage([settings.basic.default_leverage]);
      setCapitalAllocation([settings.basic.default_position_size_pct]);
    }
    if (settings?.risk) {
      setRiskThreshold([settings.risk.max_risk_per_trade_pct]);
    }
  }, [settings]);

  // Refresh system health periodically with real health check
  useEffect(() => {
    const fetchSystemHealth = async () => {
      try {
        const startTime = Date.now();
        const response = await fetch(`${API_BASE}/api/health`);
        const latency = Date.now() - startTime;
        const isHealthy = response.ok;

        setSystemHealth((prev) => ({
          ...prev,
          apiLatency: latency,
          wsConnected: isHealthy,
          dbConnected: isHealthy,
          lastUpdate: new Date().toLocaleTimeString(),
        }));
      } catch {
        setSystemHealth((prev) => ({
          ...prev,
          wsConnected: false,
          dbConnected: false,
          lastUpdate: new Date().toLocaleTimeString(),
        }));
      }
    };

    fetchSystemHealth();
    const interval = setInterval(fetchSystemHealth, 10000); // Check every 10 seconds
    return () => clearInterval(interval);
  }, []);

  // Sync notification preferences from API to local state
  useEffect(() => {
    if (!isLoadingNotifications && apiNotificationPrefs) {
      setNotifications({
        email: apiNotificationPrefs.channels.email,
        push: apiNotificationPrefs.channels.push.enabled,
        telegram: apiNotificationPrefs.channels.telegram.enabled,
        discord: apiNotificationPrefs.channels.discord.enabled,
        sound: apiNotificationPrefs.channels.sound,
        priceAlerts: apiNotificationPrefs.alerts.price_alerts,
        tradeAlerts: apiNotificationPrefs.alerts.trade_alerts,
        systemAlerts: apiNotificationPrefs.alerts.system_alerts,
      });
      setTelegramToken(apiNotificationPrefs.channels.telegram.bot_token || "");
      setTelegramChatId(apiNotificationPrefs.channels.telegram.chat_id || "");
      setDiscordWebhookUrl(apiNotificationPrefs.channels.discord.webhook_url || "");
      setVapidPublicKey(apiNotificationPrefs.channels.push.vapid_public_key || "");
      setVapidPrivateKey(apiNotificationPrefs.channels.push.vapid_private_key || "");
      setAlertThreshold([apiNotificationPrefs.price_alert_threshold]);
    }
  }, [isLoadingNotifications, apiNotificationPrefs]);

  // Toggle trading pair
  const toggleTradingPair = (symbol: string) => {
    setTradingPairs((prev) =>
      prev.map((pair) =>
        pair.symbol === symbol ? { ...pair, enabled: !pair.enabled } : pair
      )
    );
  };

  // Update trading pair settings
  const updateTradingPairSetting = (
    symbol: string,
    field: "leverage" | "positionSize",
    value: number
  ) => {
    setTradingPairs((prev) =>
      prev.map((pair) =>
        pair.symbol === symbol ? { ...pair, [field]: value } : pair
      )
    );
  };

  // Handle bot toggle
  const handleBotToggle = async (checked: boolean) => {
    setBotActive(checked);
    try {
      if (checked) {
        await startBot();
        toast({ title: "Bot Started", description: "Trading bot is now active" });
      } else {
        await stopBot();
        toast({ title: "Bot Stopped", description: "Trading bot is now inactive" });
      }
    } catch (error) {
      setBotActive(!checked);
      toast({ title: "Error", description: "Failed to change bot status", variant: "destructive" });
    }
  };


  // Test API connection
  const testConnection = async () => {
    setIsTestingConnection(true);
    setConnectionStatus("testing");
    try {
      const response = await fetch(`${API_BASE}/api/health`);
      if (response.ok) {
        setConnectionStatus("connected");
        toast({ title: "Connection Successful", description: "API is reachable" });
      } else {
        setConnectionStatus("disconnected");
        toast({ title: "Connection Failed", description: "API is not responding", variant: "destructive" });
      }
    } catch {
      setConnectionStatus("disconnected");
      toast({ title: "Connection Failed", description: "Could not reach API", variant: "destructive" });
    } finally {
      setIsTestingConnection(false);
    }
  };

  // Save settings for a section
  const saveSettings = async (section: SettingsSection) => {
    setIsSaving(true);
    try {
      if (section === "bot") {
        await updateSettings({
          basic: {
            ...settings.basic,
            enabled: botActive,
            default_leverage: maxLeverage[0],
            default_position_size_pct: capitalAllocation[0],
          },
          risk: {
            ...settings.risk,
            max_risk_per_trade_pct: riskThreshold[0],
          },
          strategy: settings.strategy,
          exit_strategy: settings.exit_strategy,
        });
      } else if (section === "notifications") {
        // Save notification preferences to backend API
        const success = await saveNotificationPreferences(
          localToApiFormat(notifications, {
            telegramToken,
            telegramChatId,
            discordWebhookUrl,
            vapidPublicKey,
            vapidPrivateKey,
            alertThreshold: alertThreshold[0],
          })
        );
        if (!success) {
          throw new Error("Failed to save notification preferences");
        }
      }
      setSavedSection(section);
      toast({ title: "Settings Saved", description: `${section.charAt(0).toUpperCase() + section.slice(1)} settings updated successfully` });
      setTimeout(() => setSavedSection(null), 2000);
    } catch (error) {
      toast({ title: "Save Failed", description: "Could not save settings", variant: "destructive" });
    } finally {
      setIsSaving(false);
    }
  };

  // Format last active time helper
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

  // Handle 2FA verification
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

  // Handle 2FA disable
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

  // Copy secret to clipboard
  const copySecretToClipboard = () => {
    if (setup2FAData?.secret) {
      navigator.clipboard.writeText(setup2FAData.secret);
      toast.success('Secret copied to clipboard');
    }
  };

  // Section navigation items - matching old UI tabs
  const sectionNav = [
    { id: "bot" as const, label: "Bot Settings", icon: Settings2, description: "Basic bot configuration" },
    { id: "per-symbol" as const, label: "Per-Symbol", icon: Coins, description: "Symbol-specific settings" },
    { id: "strategy" as const, label: "Strategy Tuning", icon: TrendingUp, description: "Advanced strategy config" },
    { id: "health" as const, label: "System Health", icon: Activity, description: "System status monitoring" },
    { id: "api" as const, label: "API Keys", icon: Key, description: "API configuration" },
    { id: "notifications" as const, label: "Thông báo", icon: Bell, description: "Alert settings" },
    { id: "security" as const, label: "Bảo mật", icon: Shield, description: "Account security" },
  ];

  // Current balance for calculations
  const currentBalance = portfolio?.current_balance || settings?.basic?.initial_balance || 10000;

  return (
    <ErrorBoundary>
      <PageWrapper>
        {/* Page Header */}
        <motion.div
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          className="mb-8"
        >
          <div className="flex items-center gap-3 mb-2">
            <GlowIcon icon={Settings2} size="lg" color={luxuryColors.cyan} />
            <div>
              <h1 className="text-2xl lg:text-3xl font-black">
                <GradientText>Settings</GradientText>
              </h1>
              <p className="text-sm" style={{ color: luxuryColors.textMuted }}>
                Configure your trading bot and preferences
              </p>
            </div>
          </div>
        </motion.div>

        <div className="flex flex-col lg:flex-row gap-6">
          {/* Section Navigation - Sidebar */}
          <motion.div
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: 0.1 }}
            className="lg:w-72 flex-shrink-0"
          >
            <GlassCard noPadding>
              <div className="p-2 space-y-1">
                {sectionNav.map((section, index) => (
                  <motion.button
                    key={section.id}
                    initial={{ opacity: 0, x: -10 }}
                    animate={{ opacity: 1, x: 0 }}
                    transition={{ delay: 0.1 + index * 0.05 }}
                    onClick={() => setActiveSection(section.id)}
                    className={`w-full flex items-center gap-3 p-4 rounded-xl transition-all duration-300 group ${
                      activeSection === section.id
                        ? "border-l-2"
                        : ""
                    }`}
                    style={{
                      background: activeSection === section.id
                        ? `linear-gradient(to right, ${luxuryColors.cyan}20, transparent)`
                        : 'transparent',
                      borderColor: activeSection === section.id ? luxuryColors.cyan : 'transparent',
                    }}
                  >
                    <GlowIcon
                      icon={section.icon as React.ElementType}
                      size="sm"
                      color={activeSection === section.id ? luxuryColors.cyan : luxuryColors.textMuted}
                    />
                    <div className="flex-1 text-left">
                      <div
                        className="font-medium"
                        style={{
                          color: activeSection === section.id ? luxuryColors.textPrimary : luxuryColors.textMuted
                        }}
                      >
                        {section.label}
                      </div>
                      <div className="text-xs" style={{ color: luxuryColors.textMuted }}>
                        {section.description}
                      </div>
                    </div>
                    <ChevronRight
                      className={`h-4 w-4 transition-transform ${
                        activeSection === section.id ? "rotate-90" : ""
                      }`}
                      style={{
                        color: activeSection === section.id ? luxuryColors.cyan : luxuryColors.textMuted
                      }}
                    />
                  </motion.button>
                ))}
              </div>
            </GlassCard>
          </motion.div>

          {/* Main Content Area */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.2 }}
            className="flex-1 min-w-0"
          >
              {/* Bot Settings Section - matching old UI */}
              {activeSection === "bot" && (
                <div className="space-y-6">
                  <SectionHeader
                    title="Cài đặt Bot"
                    subtitle="Quản lý cấu hình và tùy chọn cho trading bot của bạn"
                    icon={Settings2}
                  />

                  {/* Bot Configuration Card */}
                  <GlassCard>
                    <div className="flex items-center justify-between mb-6">
                      <h3 className="font-semibold text-lg" style={{ color: luxuryColors.textPrimary }}>
                        Bot Configuration
                      </h3>
                      <Badge variant={botActive ? "success" : "default"} glow={botActive}>
                        {botActive ? "ACTIVE" : "INACTIVE"}
                      </Badge>
                    </div>

                    {/* Bot Status */}
                    <div
                      className="flex items-center justify-between p-4 rounded-xl mb-6"
                      style={{ backgroundColor: luxuryColors.bgSecondary }}
                    >
                      <div>
                        <h4 className="font-medium" style={{ color: luxuryColors.textPrimary }}>
                          Bot Status
                        </h4>
                        <p className="text-sm" style={{ color: luxuryColors.textMuted }}>
                          {botActive ? "Bot is actively trading" : "Bot is currently stopped"}
                        </p>
                      </div>
                      <PremiumSwitch
                        checked={botActive}
                        onCheckedChange={handleBotToggle}
                      />
                    </div>

                    {/* Capital Allocation */}
                    <div className="space-y-6">
                      <SliderSetting
                        label="Capital Allocation"
                        value={capitalAllocation[0]}
                        unit="%"
                        min={1}
                        max={100}
                        step={1}
                        onChange={(v) => setCapitalAllocation([v])}
                        color="primary"
                        description={`Amount: $${((currentBalance * capitalAllocation[0]) / 100).toFixed(2)}`}
                      />
                      <div className="flex justify-between text-xs" style={{ color: luxuryColors.textMuted }}>
                        <span>Conservative (10%)</span>
                        <span>Aggressive (100%)</span>
                      </div>

                      {/* Maximum Leverage */}
                      <SliderSetting
                        label="Maximum Leverage"
                        value={maxLeverage[0]}
                        unit="x"
                        min={1}
                        max={20}
                        step={1}
                        onChange={(v) => setMaxLeverage([v])}
                        color="warning"
                      />
                      <div className="flex justify-between text-xs" style={{ color: luxuryColors.textMuted }}>
                        <span>Safe (1x)</span>
                        <span>High Risk (20x)</span>
                      </div>

                      {/* Risk Threshold */}
                      <SliderSetting
                        label="Risk Threshold"
                        value={riskThreshold[0]}
                        unit="%"
                        min={1}
                        max={15}
                        step={0.5}
                        onChange={(v) => setRiskThreshold([v])}
                        color="loss"
                        description={`Max loss per trade: $${((currentBalance * riskThreshold[0]) / 100).toFixed(2)}`}
                      />
                      <div className="flex justify-between text-xs" style={{ color: luxuryColors.textMuted }}>
                        <span>Conservative (1%)</span>
                        <span>Aggressive (15%)</span>
                      </div>
                    </div>
                  </GlassCard>

                  {/* Active Trading Pairs */}
                  <GlassCard>
                    <h3 className="font-semibold mb-4" style={{ color: luxuryColors.textPrimary }}>
                      Active Trading Pairs
                    </h3>
                    <div className="grid grid-cols-2 gap-4">
                      {tradingPairs.map((pair) => (
                        <div
                          key={pair.symbol}
                          className="flex items-center justify-between p-3 rounded-xl"
                          style={{ backgroundColor: luxuryColors.bgSecondary }}
                        >
                          <span style={{ color: luxuryColors.textPrimary }}>{pair.symbol}</span>
                          <PremiumSwitch
                            checked={pair.enabled}
                            onCheckedChange={() => toggleTradingPair(pair.symbol)}
                          />
                        </div>
                      ))}
                    </div>
                  </GlassCard>

                  <SaveButton
                    onClick={() => saveSettings("bot")}
                    isLoading={isSaving}
                    isSaved={savedSection === "bot"}
                  />
                </div>
              )}

              {/* Per-Symbol Settings Section */}
              {activeSection === "per-symbol" && (
                <div className="space-y-6">
                  <SectionHeader
                    title="Per-Symbol Settings"
                    subtitle="Configure individual settings for each trading pair"
                    icon={Coins}
                  />

                  {tradingPairs.filter(p => p.enabled).map((pair) => (
                    <GlassCard key={pair.symbol}>
                      <div className="flex items-center justify-between mb-4">
                        <div className="flex items-center gap-3">
                          <GlowIcon icon={Coins} size="sm" color={luxuryColors.cyan} />
                          <h3 className="font-semibold" style={{ color: luxuryColors.textPrimary }}>
                            {pair.symbol}
                          </h3>
                        </div>
                        <Badge variant="success" size="sm">Active</Badge>
                      </div>
                      <div className="space-y-4">
                        <SliderSetting
                          label="Position Size"
                          value={pair.positionSize || 25}
                          unit="%"
                          min={5}
                          max={50}
                          step={5}
                          onChange={(v) => updateTradingPairSetting(pair.symbol, "positionSize", v)}
                          color="primary"
                        />
                        <SliderSetting
                          label="Max Leverage"
                          value={pair.leverage || 3}
                          unit="x"
                          min={1}
                          max={10}
                          step={1}
                          onChange={(v) => updateTradingPairSetting(pair.symbol, "leverage", v)}
                          color="warning"
                        />
                      </div>
                    </GlassCard>
                  ))}

                  {tradingPairs.filter(p => p.enabled).length === 0 && (
                    <GlassCard>
                      <div className="text-center py-8">
                        <GlowIcon icon={AlertTriangle} size="lg" color={luxuryColors.warning} className="mx-auto mb-4" />
                        <p style={{ color: luxuryColors.textMuted }}>
                          No active trading pairs. Enable pairs in Bot Settings tab.
                        </p>
                      </div>
                    </GlassCard>
                  )}

                  <SaveButton
                    onClick={() => saveSettings("per-symbol")}
                    isLoading={isSaving}
                    isSaved={savedSection === "per-symbol"}
                  />
                </div>
              )}

              {/* Strategy Tuning Section */}
              {activeSection === "strategy" && (
                <div className="space-y-6">
                  <SectionHeader
                    title="Strategy Tuning"
                    subtitle="Fine-tune your trading strategies and parameters"
                    icon={TrendingUp}
                  />

                  <GlassCard>
                    <div className="text-center py-8">
                      <GlowIcon icon={Settings2} size="lg" color={luxuryColors.cyan} className="mx-auto mb-4" />
                      <h3 className="font-semibold text-lg mb-2" style={{ color: luxuryColors.textPrimary }}>
                        Advanced Strategy Configuration
                      </h3>
                      <p className="text-sm mb-6" style={{ color: luxuryColors.textMuted }}>
                        Configure market presets, strategy parameters, risk management & engine settings
                      </p>
                      <TradingSettings />
                    </div>
                  </GlassCard>
                </div>
              )}

              {/* System Health Section */}
              {activeSection === "health" && (
                <div className="space-y-6">
                  <SectionHeader
                    title="System Health"
                    subtitle="Monitor system status and performance metrics"
                    icon={Activity}
                  />

                  {/* System Status Overview */}
                  <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                    <GlassCard noPadding className="p-4">
                      <div className="flex items-center gap-3">
                        <GlowIcon icon={Cpu} size="sm" color={systemHealth.cpu < 70 ? luxuryColors.profit : luxuryColors.loss} />
                        <div>
                          <p className="text-xs" style={{ color: luxuryColors.textMuted }}>CPU</p>
                          <p className="font-bold" style={{ color: luxuryColors.textPrimary }}>{systemHealth.cpu}%</p>
                        </div>
                      </div>
                    </GlassCard>
                    <GlassCard noPadding className="p-4">
                      <div className="flex items-center gap-3">
                        <GlowIcon icon={HardDrive} size="sm" color={systemHealth.memory < 80 ? luxuryColors.profit : luxuryColors.loss} />
                        <div>
                          <p className="text-xs" style={{ color: luxuryColors.textMuted }}>Memory</p>
                          <p className="font-bold" style={{ color: luxuryColors.textPrimary }}>{systemHealth.memory}%</p>
                        </div>
                      </div>
                    </GlassCard>
                    <GlassCard noPadding className="p-4">
                      <div className="flex items-center gap-3">
                        <GlowIcon icon={Clock} size="sm" color={luxuryColors.cyan} />
                        <div>
                          <p className="text-xs" style={{ color: luxuryColors.textMuted }}>Uptime</p>
                          <p className="font-bold" style={{ color: luxuryColors.textPrimary }}>{systemHealth.uptime}</p>
                        </div>
                      </div>
                    </GlassCard>
                    <GlassCard noPadding className="p-4">
                      <div className="flex items-center gap-3">
                        <GlowIcon icon={Zap} size="sm" color={systemHealth.apiLatency < 100 ? luxuryColors.profit : luxuryColors.warning} />
                        <div>
                          <p className="text-xs" style={{ color: luxuryColors.textMuted }}>API Latency</p>
                          <p className="font-bold" style={{ color: luxuryColors.textPrimary }}>{systemHealth.apiLatency}ms</p>
                        </div>
                      </div>
                    </GlassCard>
                  </div>

                  {/* Connection Status */}
                  <GlassCard>
                    <h3 className="font-semibold mb-4" style={{ color: luxuryColors.textPrimary }}>
                      Connection Status
                    </h3>
                    <div className="space-y-3">
                      <div className="flex items-center justify-between p-3 rounded-xl" style={{ backgroundColor: luxuryColors.bgSecondary }}>
                        <div className="flex items-center gap-3">
                          <GlowIcon icon={Server} size="sm" color={connectionStatus === "connected" ? luxuryColors.profit : luxuryColors.loss} />
                          <span style={{ color: luxuryColors.textPrimary }}>API Server</span>
                        </div>
                        <StatusIndicator label="" status={connectionStatus === "connected" ? "online" : "offline"} />
                      </div>
                      <div className="flex items-center justify-between p-3 rounded-xl" style={{ backgroundColor: luxuryColors.bgSecondary }}>
                        <div className="flex items-center gap-3">
                          <GlowIcon icon={Wifi} size="sm" color={systemHealth.wsConnected ? luxuryColors.profit : luxuryColors.loss} />
                          <span style={{ color: luxuryColors.textPrimary }}>WebSocket</span>
                        </div>
                        <StatusIndicator label="" status={systemHealth.wsConnected ? "online" : "offline"} />
                      </div>
                      <div className="flex items-center justify-between p-3 rounded-xl" style={{ backgroundColor: luxuryColors.bgSecondary }}>
                        <div className="flex items-center gap-3">
                          <GlowIcon icon={Database} size="sm" color={systemHealth.dbConnected ? luxuryColors.profit : luxuryColors.loss} />
                          <span style={{ color: luxuryColors.textPrimary }}>Database</span>
                        </div>
                        <StatusIndicator label="" status={systemHealth.dbConnected ? "online" : "offline"} />
                      </div>
                    </div>
                    <p className="text-xs mt-4" style={{ color: luxuryColors.textMuted }}>
                      Last updated: {systemHealth.lastUpdate}
                    </p>
                  </GlassCard>

                  <PremiumButton variant="secondary" onClick={testConnection} disabled={isTestingConnection} loading={isTestingConnection} fullWidth>
                    <RefreshCw className="h-4 w-4" />
                    Refresh Status
                  </PremiumButton>
                </div>
              )}

              {/* API & Connections Section */}
              {activeSection === "api" && (
                <div className="space-y-6">
                  <SectionHeader
                    title="API & Connections"
                    subtitle="Manage your exchange API keys and connection status"
                    icon={Key}
                  />

                  {/* Connection Status */}
                  <GlassCard>
                      <div className="flex items-center justify-between mb-6">
                        <div className="flex items-center gap-3">
                          <GlowIcon
                            icon={connectionStatus === "connected" ? Wifi : connectionStatus === "testing" ? Loader2 : WifiOff}
                            size="lg"
                            color={
                              connectionStatus === "connected" ? luxuryColors.profit :
                              connectionStatus === "testing" ? luxuryColors.warning :
                              luxuryColors.loss
                            }
                            className={connectionStatus === "testing" ? "animate-spin" : ""}
                          />
                          <div>
                            <h3 className="font-semibold" style={{ color: luxuryColors.textPrimary }}>
                              Connection Status
                            </h3>
                            <p className="text-sm" style={{ color: luxuryColors.textMuted }}>
                              {connectionStatus === "connected" ? "All systems operational" :
                               connectionStatus === "testing" ? "Testing connection..." : "Connection issues detected"}
                            </p>
                          </div>
                        </div>
                        <div className="flex items-center gap-3">
                          <StatusIndicator
                            label="API"
                            status={connectionStatus === "connected" ? "online" : connectionStatus === "testing" ? "pending" : "offline"}
                          />
                          <StatusIndicator
                            label="WebSocket"
                            status={connectionStatus === "connected" ? "online" : "offline"}
                          />
                        </div>
                      </div>

                      <PremiumButton
                        variant="secondary"
                        onClick={testConnection}
                        disabled={isTestingConnection}
                        loading={isTestingConnection}
                        fullWidth
                      >
                        {isTestingConnection ? (
                          "Testing Connection..."
                        ) : (
                          <>
                            <RefreshCw className="h-4 w-4" />
                            Test Connection
                          </>
                        )}
                      </PremiumButton>
                    </GlassCard>

                  {/* Binance API Configuration */}
                  <GlassCard>
                    <div className="space-y-6">
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-2">
                          <GlowIcon icon={Key} size="sm" color={luxuryColors.cyan} />
                          <h3 className="font-semibold" style={{ color: luxuryColors.textPrimary }}>
                            Binance API Configuration
                          </h3>
                        </div>
                        <Badge variant="warning" size="sm">
                          Testnet
                        </Badge>
                      </div>

                      {/* API Key Input */}
                      <div className="space-y-2">
                        <div className="flex items-center justify-between">
                          <Label htmlFor="api-key" className="text-xs uppercase tracking-wider" style={{ color: luxuryColors.textMuted }}>
                            API Key
                          </Label>
                          <button
                            type="button"
                            onClick={() => setShowApiKey(!showApiKey)}
                            className="transition-colors"
                            style={{ color: luxuryColors.textMuted }}
                          >
                            {showApiKey ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                          </button>
                        </div>
                        <PremiumInput
                          type={showApiKey ? "text" : "password"}
                          value={apiKey}
                          onChange={setApiKey}
                          placeholder="Enter your Binance API Key"
                        />
                      </div>

                      {/* Secret Key Input */}
                      <div className="space-y-2">
                        <div className="flex items-center justify-between">
                          <Label htmlFor="secret-key" className="text-xs uppercase tracking-wider" style={{ color: luxuryColors.textMuted }}>
                            Secret Key
                          </Label>
                          <button
                            type="button"
                            onClick={() => setShowSecretKey(!showSecretKey)}
                            className="transition-colors"
                            style={{ color: luxuryColors.textMuted }}
                          >
                            {showSecretKey ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                          </button>
                        </div>
                        <PremiumInput
                          type={showSecretKey ? "text" : "password"}
                          value={secretKey}
                          onChange={setSecretKey}
                          placeholder="Enter your Binance Secret Key"
                        />
                      </div>

                      {/* Security Note */}
                      <div
                        className="flex items-start gap-3 p-4 rounded-xl border"
                        style={{
                          backgroundColor: 'rgba(0, 217, 255, 0.1)',
                          borderColor: 'rgba(0, 217, 255, 0.2)',
                        }}
                      >
                        <Lock className="h-5 w-5 mt-0.5" style={{ color: luxuryColors.cyan }} />
                        <div>
                          <p className="text-sm font-medium" style={{ color: luxuryColors.cyan }}>
                            Security Note
                          </p>
                          <p className="text-xs mt-1" style={{ color: luxuryColors.textMuted }}>
                            API keys are encrypted and stored securely. Only grant Futures Trading permission to the bot.
                          </p>
                        </div>
                      </div>
                    </div>
                  </GlassCard>

                  {/* Trading Permissions */}
                  <GlassCard>
                    <h3 className="font-semibold mb-4" style={{ color: luxuryColors.textPrimary }}>
                      Trading Permissions
                    </h3>
                    <div className="space-y-3">
                      {[
                        { name: "Spot Trading", enabled: false, description: "Basic spot trading" },
                        { name: "Futures Trading", enabled: true, description: "Futures trading with leverage", locked: true },
                        { name: "Margin Trading", enabled: false, description: "Margin trading" },
                        { name: "Options Trading", enabled: false, description: "Options trading" },
                      ].map((permission) => (
                        <div
                          key={permission.name}
                          className="flex items-center justify-between p-4 rounded-xl border transition-colors"
                          style={{
                            backgroundColor: permission.enabled ? 'rgba(34, 197, 94, 0.1)' : luxuryColors.bgSecondary,
                            borderColor: permission.enabled ? 'rgba(34, 197, 94, 0.2)' : luxuryColors.borderSubtle,
                          }}
                        >
                          <div>
                            <div className="font-medium" style={{ color: luxuryColors.textPrimary }}>
                              {permission.name}
                            </div>
                            <div className="text-sm" style={{ color: luxuryColors.textMuted }}>
                              {permission.description}
                            </div>
                          </div>
                          <PremiumSwitch
                            checked={permission.enabled}
                            disabled={permission.locked}
                          />
                        </div>
                      ))}
                    </div>
                  </GlassCard>

                  <SaveButton
                    onClick={() => saveSettings("api")}
                    isLoading={isSaving}
                    isSaved={savedSection === "api"}
                  />
                </div>
              )}

              {/* Notifications Section */}
              {activeSection === "notifications" && (
                <div className="space-y-6">
                  <SectionHeader
                    title="Notifications"
                    subtitle="Configure how you receive alerts and updates"
                    icon={Bell}
                  />

                  {/* Notification Channels */}
                  <GlassCard>
                    <h3 className="font-semibold mb-6" style={{ color: luxuryColors.textPrimary }}>
                      Notification Channels
                    </h3>
                      <div className="space-y-4">
                        <NotificationToggle
                          icon={<Mail className="h-5 w-5" />}
                          title="Email Notifications"
                          description="Receive notifications via email"
                          checked={notifications.email}
                          onChange={(checked) => setNotifications(prev => ({ ...prev, email: checked }))}
                        />
                        <NotificationToggle
                          icon={<Bell className="h-5 w-5" />}
                          title="Push Notifications"
                          description={
                            !isPushSupported
                              ? "Not supported in this browser"
                              : pushPermission === 'denied'
                                ? "Permission denied - enable in browser settings"
                                : !vapidPublicKey
                                  ? "Configure VAPID keys below first"
                                  : isPushSubscribed
                                    ? "Browser push notifications enabled"
                                    : "Browser push notifications"
                          }
                          checked={isPushSubscribed}
                          disabled={!isPushSupported || pushPermission === 'denied' || isPushLoading || !vapidPublicKey}
                          onChange={async (checked) => {
                            if (checked) {
                              if (!vapidPublicKey) {
                                toast({ title: "VAPID Key Required", description: "Please enter your VAPID public key below first", variant: "destructive" });
                                return;
                              }
                              const success = await subscribeToPush(vapidPublicKey);
                              if (success) {
                                setNotifications(prev => ({ ...prev, push: true }));
                                toast({ title: "Push Enabled", description: "You will now receive browser push notifications" });
                                // Test with a local notification
                                await showLocalNotification("Push Notifications Enabled", {
                                  body: "You will now receive trading alerts here!",
                                });
                              } else {
                                toast({ title: "Push Failed", description: pushError || "Failed to enable push notifications", variant: "destructive" });
                              }
                            } else {
                              const success = await unsubscribeFromPush();
                              if (success) {
                                setNotifications(prev => ({ ...prev, push: false }));
                                toast({ title: "Push Disabled", description: "Browser push notifications disabled" });
                              }
                            }
                          }}
                        />
                        <NotificationToggle
                          icon={<MessageCircle className="h-5 w-5" />}
                          title="Telegram Bot"
                          description="Receive notifications via Telegram"
                          checked={notifications.telegram}
                          onChange={(checked) => setNotifications(prev => ({ ...prev, telegram: checked }))}
                        />
                        <NotificationToggle
                          icon={<MessageCircle className="h-5 w-5" />}
                          title="Discord Webhook"
                          description="Receive notifications via Discord"
                          checked={notifications.discord}
                          onChange={(checked) => setNotifications(prev => ({ ...prev, discord: checked }))}
                        />
                      </div>

                    {/* Telegram Settings (conditional) */}
                    <AnimatePresence>
                      {notifications.telegram && (
                        <motion.div
                          initial={{ opacity: 0, height: 0 }}
                          animate={{ opacity: 1, height: "auto" }}
                          exit={{ opacity: 0, height: 0 }}
                          className="mt-4 p-4 rounded-xl border space-y-4"
                          style={{
                            backgroundColor: 'rgba(0, 217, 255, 0.1)',
                            borderColor: 'rgba(0, 217, 255, 0.2)',
                          }}
                        >
                          <PremiumInput
                            label="Telegram Bot Token"
                            value={telegramToken}
                            onChange={setTelegramToken}
                            placeholder="Enter bot token from @BotFather"
                          />
                          <PremiumInput
                            label="Telegram Chat ID"
                            value={telegramChatId}
                            onChange={setTelegramChatId}
                            placeholder="Enter your chat ID (use @userinfobot)"
                          />
                          <div className="flex justify-end">
                            <PremiumButton
                              variant="secondary"
                              size="sm"
                              onClick={async () => {
                                await saveSettings("notifications");
                                const success = await testNotification("telegram");
                                if (success) {
                                  toast({ title: "Test Sent", description: "Telegram test notification sent successfully" });
                                } else {
                                  toast({ title: "Test Failed", description: "Failed to send Telegram test notification", variant: "destructive" });
                                }
                              }}
                              disabled={!telegramToken || !telegramChatId}
                            >
                              Test Telegram
                            </PremiumButton>
                          </div>
                        </motion.div>
                      )}
                    </AnimatePresence>

                    {/* Push Notification VAPID Keys (always show for supported browsers) */}
                    {isPushSupported && (
                      <div
                        className="mt-4 p-4 rounded-xl border space-y-4"
                        style={{
                          backgroundColor: 'rgba(59, 130, 246, 0.1)',
                          borderColor: 'rgba(59, 130, 246, 0.2)',
                        }}
                      >
                        <div className="flex items-center gap-2 mb-3">
                          <Bell className="h-5 w-5" style={{ color: luxuryColors.cyan }} />
                          <span className="font-medium" style={{ color: luxuryColors.textPrimary }}>
                            Push Notification Keys (VAPID)
                          </span>
                        </div>
                        <p className="text-xs mb-3" style={{ color: luxuryColors.textMuted }}>
                          Generate VAPID keys using: <code className="px-1 py-0.5 rounded bg-black/30">npx web-push generate-vapid-keys</code>
                        </p>
                        <PremiumInput
                          label="VAPID Public Key"
                          value={vapidPublicKey}
                          onChange={setVapidPublicKey}
                          placeholder="BLxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx..."
                        />
                        <PremiumInput
                          label="VAPID Private Key (stored securely)"
                          type="password"
                          value={vapidPrivateKey}
                          onChange={setVapidPrivateKey}
                          placeholder="Enter private key (keep secure)"
                        />
                        <div className="flex justify-end">
                          <PremiumButton
                            variant="secondary"
                            size="sm"
                            onClick={async () => {
                              await saveSettings("notifications");
                              toast({ title: "Keys Saved", description: "VAPID keys saved. You can now enable Push Notifications." });
                            }}
                            disabled={!vapidPublicKey}
                          >
                            Save VAPID Keys
                          </PremiumButton>
                        </div>
                      </div>
                    )}

                    {/* Discord Webhook Settings (conditional) */}
                    <AnimatePresence>
                      {notifications.discord && (
                        <motion.div
                          initial={{ opacity: 0, height: 0 }}
                          animate={{ opacity: 1, height: "auto" }}
                          exit={{ opacity: 0, height: 0 }}
                          className="mt-4 p-4 rounded-xl border"
                          style={{
                            backgroundColor: 'rgba(138, 43, 226, 0.1)',
                            borderColor: 'rgba(138, 43, 226, 0.2)',
                          }}
                        >
                          <PremiumInput
                            label="Discord Webhook URL"
                            value={discordWebhookUrl}
                            onChange={setDiscordWebhookUrl}
                            placeholder="https://discord.com/api/webhooks/..."
                          />
                          <div className="flex items-center justify-between mt-3">
                            <p className="text-xs" style={{ color: luxuryColors.textMuted }}>
                              Create a webhook in Discord Server Settings → Integrations → Webhooks
                            </p>
                            <PremiumButton
                              variant="secondary"
                              size="sm"
                              onClick={async () => {
                                // Save first, then test
                                await saveSettings("notifications");
                                const success = await testNotification("discord");
                                if (success) {
                                  toast({ title: "Test Sent", description: "Discord test notification sent successfully" });
                                } else {
                                  toast({ title: "Test Failed", description: "Failed to send Discord test notification", variant: "destructive" });
                                }
                              }}
                              disabled={!discordWebhookUrl}
                            >
                              Test Discord
                            </PremiumButton>
                          </div>
                        </motion.div>
                      )}
                    </AnimatePresence>
                  </GlassCard>

                  {/* Alert Types */}
                  <GlassCard>
                    <h3 className="font-semibold mb-6" style={{ color: luxuryColors.textPrimary }}>
                      Alert Types
                    </h3>
                      <div className="space-y-4">
                        <NotificationToggle
                          icon={<TrendingUp className="h-5 w-5" />}
                          title="Price Alerts"
                          description="Get notified on significant price movements"
                          checked={notifications.priceAlerts}
                          onChange={(checked) => setNotifications(prev => ({ ...prev, priceAlerts: checked }))}
                        />
                        <NotificationToggle
                          icon={<Zap className="h-5 w-5" />}
                          title="Trade Alerts"
                          description="Get notified when trades are executed"
                          checked={notifications.tradeAlerts}
                          onChange={(checked) => setNotifications(prev => ({ ...prev, tradeAlerts: checked }))}
                        />
                        <NotificationToggle
                          icon={<AlertTriangle className="h-5 w-5" />}
                          title="System Alerts"
                          description="Important system notifications"
                          checked={notifications.systemAlerts}
                          onChange={(checked) => setNotifications(prev => ({ ...prev, systemAlerts: checked }))}
                        />
                        <NotificationToggle
                          icon={<Volume2 className="h-5 w-5" />}
                          title="Sound Effects"
                          description="Play sounds for important events"
                          checked={notifications.sound}
                          onChange={(checked) => setNotifications(prev => ({ ...prev, sound: checked }))}
                        />
                      </div>
                  </GlassCard>

                  {/* Alert Threshold */}
                  <GlassCard>
                      <SliderSetting
                        label="Price Alert Threshold"
                        value={alertThreshold[0]}
                        unit="%"
                        min={1}
                        max={20}
                        step={1}
                        onChange={(v) => setAlertThreshold([v])}
                        color="warning"
                        description="Minimum price change to trigger an alert"
                      />
                  </GlassCard>

                  <SaveButton
                    onClick={() => saveSettings("notifications")}
                    isLoading={isSaving}
                    isSaved={savedSection === "notifications"}
                  />
                </div>
              )}

              {/* Account & Security Section */}
              {activeSection === "security" && (
                <div className="space-y-6">
                  <SectionHeader
                    title="Account & Security"
                    subtitle="Manage your profile and security settings"
                    icon={Shield}
                  />

                  {/* Profile Info */}
                  <GlassCard>
                    <div className="flex items-center gap-4 mb-6">
                      <GlowIcon icon={User} size="lg" color={luxuryColors.cyan} />
                      <div>
                        <h3 className="font-semibold text-lg" style={{ color: luxuryColors.textPrimary }}>
                          Trader Profile
                        </h3>
                        <p className="text-sm" style={{ color: luxuryColors.textMuted }}>
                          trader@example.com
                        </p>
                      </div>
                    </div>

                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                      <PremiumInput
                        label="Display Name"
                        value="Crypto Trader"
                        onChange={() => {}}
                      />
                      <PremiumInput
                        label="Email"
                        value="trader@example.com"
                        onChange={() => {}}
                        disabled
                      />
                    </div>
                  </GlassCard>

                  {/* Two-Factor Authentication */}
                  <GlassCard>
                    <div className="flex items-center justify-between mb-6">
                      <div className="flex items-center gap-3">
                        <GlowIcon
                          icon={Shield}
                          size="lg"
                          color={twoFactorEnabled ? luxuryColors.profit : luxuryColors.textMuted}
                        />
                        <div>
                          <h3 className="font-semibold" style={{ color: luxuryColors.textPrimary }}>
                            Two-Factor Authentication
                          </h3>
                          <p className="text-sm" style={{ color: luxuryColors.textMuted }}>
                            {twoFactorEnabled ? "Your account is protected with 2FA" : "Add an extra layer of security"}
                          </p>
                        </div>
                      </div>
                      <Badge variant={twoFactorEnabled ? "success" : "warning"}>
                        {twoFactorEnabled ? "Enabled" : "Disabled"}
                      </Badge>
                    </div>

                    <PremiumButton
                      variant={twoFactorEnabled ? "secondary" : "success"}
                      fullWidth
                      onClick={() => {
                        if (twoFactorEnabled) {
                          setShowDisable2FADialog(true);
                        } else {
                          setup2FA();
                        }
                      }}
                      disabled={isSettingUp2FA}
                    >
                      {isSettingUp2FA ? (
                        <>
                          <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                          Setting up...
                        </>
                      ) : twoFactorEnabled ? "Disable 2FA" : "Enable 2FA"}
                    </PremiumButton>
                  </GlassCard>

                  {/* Change Password */}
                  <GlassCard>
                    <h3 className="font-semibold mb-4" style={{ color: luxuryColors.textPrimary }}>
                      Change Password
                    </h3>
                    <form onSubmit={async (e) => {
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
                        setPasswordData({ current: '', new: '', confirm: '' });
                      }
                    }}>
                      <div className="space-y-3">
                        <PremiumInput
                          type="password"
                          value={passwordData.current}
                          onChange={(e) => setPasswordData({ ...passwordData, current: e.target.value })}
                          placeholder="Current password"
                          disabled={isChangingPassword}
                        />
                        <PremiumInput
                          type="password"
                          value={passwordData.new}
                          onChange={(e) => setPasswordData({ ...passwordData, new: e.target.value })}
                          placeholder="New password"
                          disabled={isChangingPassword}
                        />
                        <PremiumInput
                          type="password"
                          value={passwordData.confirm}
                          onChange={(e) => setPasswordData({ ...passwordData, confirm: e.target.value })}
                          placeholder="Confirm new password"
                          disabled={isChangingPassword}
                        />
                      </div>
                      <div className="mt-4">
                        <PremiumButton
                          variant="secondary"
                          fullWidth
                          type="submit"
                          disabled={isChangingPassword || !passwordData.current || !passwordData.new || !passwordData.confirm}
                        >
                          {isChangingPassword ? (
                            <>
                              <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                              Updating...
                            </>
                          ) : 'Update Password'}
                        </PremiumButton>
                      </div>
                    </form>
                  </GlassCard>

                  {/* Active Sessions */}
                  <GlassCard>
                    <h3 className="font-semibold mb-4" style={{ color: luxuryColors.textPrimary }}>
                      Active Sessions
                    </h3>
                    <div className="space-y-3">
                      {isLoadingSessions ? (
                        <div className="flex justify-center py-8">
                          <Loader2 className="w-6 h-6 animate-spin" style={{ color: luxuryColors.textMuted }} />
                        </div>
                      ) : sessions.length === 0 ? (
                        <p className="text-center py-4" style={{ color: luxuryColors.textMuted }}>
                          No active sessions found
                        </p>
                      ) : (
                        sessions.map((session) => (
                          <div
                            key={session.session_id}
                            className="flex items-center justify-between p-4 rounded-xl border"
                            style={{
                              backgroundColor: luxuryColors.bgSecondary,
                              borderColor: luxuryColors.borderSubtle,
                            }}
                          >
                            <div className="flex items-center gap-3">
                              <GlowIcon
                                icon={Smartphone}
                                size="sm"
                                color={session.is_current ? luxuryColors.profit : luxuryColors.textMuted}
                              />
                              <div>
                                <div className="font-medium" style={{ color: luxuryColors.textPrimary }}>
                                  {session.browser} on {session.os}
                                </div>
                                <div className="text-xs" style={{ color: luxuryColors.textMuted }}>
                                  {session.location}
                                </div>
                              </div>
                            </div>
                            <div className="text-right">
                              <div
                                className="text-sm"
                                style={{
                                  color: session.is_current ? luxuryColors.profit : luxuryColors.textMuted
                                }}
                              >
                                {session.is_current ? 'Active now' : formatLastActive(session.last_active)}
                              </div>
                              {!session.is_current && (
                                <button
                                  className="text-xs hover:underline"
                                  style={{ color: luxuryColors.loss }}
                                  onClick={() => revokeSession(session.session_id)}
                                >
                                  Revoke
                                </button>
                              )}
                            </div>
                          </div>
                        ))
                      )}
                    </div>
                    <div className="mt-4">
                      <AlertDialog>
                        <AlertDialogTrigger asChild>
                          <PremiumButton variant="danger" fullWidth disabled={sessions.length <= 1}>
                            <LogOut className="h-4 w-4" />
                            Sign Out All Devices
                          </PremiumButton>
                        </AlertDialogTrigger>
                        <AlertDialogContent className="bg-slate-900 border-slate-700">
                          <AlertDialogHeader>
                            <AlertDialogTitle className="text-gray-100">
                              Sign Out All Sessions?
                            </AlertDialogTitle>
                            <AlertDialogDescription className="text-gray-400">
                              This will sign you out from all devices except this one.
                            </AlertDialogDescription>
                          </AlertDialogHeader>
                          <AlertDialogFooter>
                            <AlertDialogCancel className="border-slate-700">Cancel</AlertDialogCancel>
                            <AlertDialogAction
                              onClick={revokeAllSessions}
                              className="bg-red-600 hover:bg-red-700"
                            >
                              Sign Out All
                            </AlertDialogAction>
                          </AlertDialogFooter>
                        </AlertDialogContent>
                      </AlertDialog>
                    </div>
                  </GlassCard>

                  <SaveButton
                    onClick={() => saveSettings("security")}
                    isLoading={isSaving}
                    isSaved={savedSection === "security"}
                  />
                </div>
              )}
          </motion.div>
        </div>

        {/* Chatbot Widget */}
        <ChatBot />

        {/* 2FA Setup Dialog - Luxury UI */}
        <Dialog open={!!setup2FAData} onOpenChange={(open) => !open && cancelSetup2FA()}>
          <DialogContent
            className="max-w-md border-0 p-0 overflow-hidden"
            style={{
              background: `linear-gradient(135deg, ${luxuryColors.cardBg} 0%, rgba(15, 23, 42, 0.98) 100%)`,
              boxShadow: `0 25px 50px -12px rgba(0, 0, 0, 0.8), 0 0 40px ${luxuryColors.cyan}15`,
              backdropFilter: 'blur(20px)',
            }}
          >
            <div className="p-6">
              <DialogHeader className="mb-6">
                <div className="flex items-center gap-3 mb-2">
                  <div
                    className="p-2 rounded-lg"
                    style={{ background: `${luxuryColors.cyan}20` }}
                  >
                    <Shield className="w-5 h-5" style={{ color: luxuryColors.cyan }} />
                  </div>
                  <DialogTitle style={{ color: luxuryColors.text }}>
                    Set Up Two-Factor Authentication
                  </DialogTitle>
                </div>
                <DialogDescription style={{ color: luxuryColors.textMuted }}>
                  Scan this QR code with your authenticator app, then enter the verification code.
                </DialogDescription>
              </DialogHeader>

              {setup2FAData && (
                <div className="space-y-5">
                  {/* QR Code - Fixed: backend already includes data:image/png;base64, prefix */}
                  <div
                    className="flex justify-center p-4 rounded-xl mx-auto"
                    style={{
                      background: 'white',
                      maxWidth: '220px',
                      boxShadow: `0 4px 20px ${luxuryColors.cyan}30`
                    }}
                  >
                    <img
                      src={setup2FAData.qr_code}
                      alt="2FA QR Code"
                      className="w-48 h-48"
                    />
                  </div>

                  {/* Manual Entry Secret */}
                  <div className="space-y-2">
                    <Label style={{ color: luxuryColors.textSecondary }}>Manual Entry Code</Label>
                    <div className="flex gap-2">
                      <PremiumInput
                        readOnly
                        value={setup2FAData.secret}
                        className="font-mono text-sm flex-1"
                      />
                      <PremiumButton
                        variant="secondary"
                        size="sm"
                        onClick={copySecretToClipboard}
                        className="shrink-0 !px-3"
                      >
                        <Copy className="w-4 h-4" />
                      </PremiumButton>
                    </div>
                    <p className="text-xs" style={{ color: luxuryColors.textMuted }}>
                      If you can't scan the QR code, enter this secret manually in your authenticator app.
                    </p>
                  </div>

                  {/* Verification Code Input */}
                  <div className="space-y-2">
                    <Label htmlFor="verificationCode" style={{ color: luxuryColors.textSecondary }}>
                      Verification Code
                    </Label>
                    <PremiumInput
                      id="verificationCode"
                      type="text"
                      inputMode="numeric"
                      pattern="[0-9]*"
                      maxLength={6}
                      placeholder="000000"
                      value={verificationCode}
                      onChange={(e) => setVerificationCode(e.target.value.replace(/\D/g, ''))}
                      className="text-center text-2xl tracking-[0.5em]"
                    />
                  </div>
                </div>
              )}

              <DialogFooter className="mt-6 gap-3">
                <PremiumButton variant="secondary" onClick={cancelSetup2FA}>
                  Cancel
                </PremiumButton>
                <PremiumButton
                  variant="success"
                  onClick={handleVerify2FA}
                  disabled={isSecurityLoading || verificationCode.length !== 6}
                >
                  {isSecurityLoading ? (
                    <>
                      <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                      Verifying...
                    </>
                  ) : (
                    'Enable 2FA'
                  )}
                </PremiumButton>
              </DialogFooter>
            </div>
          </DialogContent>
        </Dialog>

        {/* Disable 2FA Dialog - Luxury UI */}
        <Dialog open={showDisable2FADialog} onOpenChange={setShowDisable2FADialog}>
          <DialogContent
            className="max-w-md border-0 p-0 overflow-hidden"
            style={{
              background: `linear-gradient(135deg, ${luxuryColors.cardBg} 0%, rgba(15, 23, 42, 0.98) 100%)`,
              boxShadow: `0 25px 50px -12px rgba(0, 0, 0, 0.8), 0 0 40px ${luxuryColors.red}15`,
              backdropFilter: 'blur(20px)',
            }}
          >
            <div className="p-6">
              <DialogHeader className="mb-6">
                <div className="flex items-center gap-3 mb-2">
                  <div
                    className="p-2 rounded-lg"
                    style={{ background: `${luxuryColors.red}20` }}
                  >
                    <AlertTriangle className="w-5 h-5" style={{ color: luxuryColors.red }} />
                  </div>
                  <DialogTitle style={{ color: luxuryColors.text }}>
                    Disable Two-Factor Authentication?
                  </DialogTitle>
                </div>
                <DialogDescription style={{ color: luxuryColors.textMuted }}>
                  This will make your account less secure. Enter your 2FA code to confirm.
                </DialogDescription>
              </DialogHeader>

              <div className="space-y-2">
                <Label htmlFor="disableCode" style={{ color: luxuryColors.textSecondary }}>
                  Current 2FA Code
                </Label>
                <PremiumInput
                  id="disableCode"
                  type="text"
                  inputMode="numeric"
                  pattern="[0-9]*"
                  maxLength={6}
                  placeholder="000000"
                  value={disableCode}
                  onChange={(e) => setDisableCode(e.target.value.replace(/\D/g, ''))}
                  className="text-center text-2xl tracking-[0.5em]"
                />
              </div>

              <DialogFooter className="mt-6 gap-3">
                <PremiumButton
                  variant="secondary"
                  onClick={() => {
                    setShowDisable2FADialog(false);
                    setDisableCode('');
                  }}
                >
                  Cancel
                </PremiumButton>
                <PremiumButton
                  variant="danger"
                  onClick={handleDisable2FA}
                  disabled={isSecurityLoading || disableCode.length !== 6}
                >
                  {isSecurityLoading ? (
                    <>
                      <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                      Disabling...
                    </>
                  ) : (
                    'Disable 2FA'
                  )}
                </PremiumButton>
              </DialogFooter>
            </div>
          </DialogContent>
        </Dialog>
      </PageWrapper>
    </ErrorBoundary>
  );
};

// Premium Switch Component with smooth animation
const PremiumSwitch = ({
  checked,
  onCheckedChange,
  disabled,
}: {
  checked: boolean;
  onCheckedChange?: (checked: boolean) => void;
  disabled?: boolean;
}) => (
  <Switch
    checked={checked}
    onCheckedChange={onCheckedChange}
    disabled={disabled}
    className={`
      data-[state=checked]:bg-gradient-to-r data-[state=checked]:from-profit data-[state=checked]:to-emerald-400
      data-[state=unchecked]:bg-muted
      transition-all duration-300
      ${disabled ? "opacity-50 cursor-not-allowed" : ""}
    `}
  />
);

// Slider Setting Component
const SliderSetting = ({
  label,
  value,
  unit,
  min,
  max,
  step,
  onChange,
  color,
  description,
}: {
  label: string;
  value: number;
  unit: string;
  min: number;
  max: number;
  step: number;
  onChange: (value: number) => void;
  color: "profit" | "loss" | "primary" | "warning";
  description?: string;
}) => {
  const colorClasses = {
    profit: "text-profit",
    loss: "text-loss",
    primary: "text-primary",
    warning: "text-warning",
  };

  return (
    <div className="space-y-3">
      <div className="flex justify-between items-center">
        <Label className="text-sm text-muted-foreground">{label}</Label>
        <span className={`text-sm font-bold ${colorClasses[color]}`}>
          {value}{unit}
        </span>
      </div>
      <Slider
        value={[value]}
        onValueChange={([v]) => onChange(v)}
        min={min}
        max={max}
        step={step}
        className="w-full"
      />
      {description && (
        <p className="text-xs text-muted-foreground">{description}</p>
      )}
    </div>
  );
};

// Status Indicator Component
const StatusIndicator = ({
  label,
  status,
}: {
  label: string;
  status: "online" | "offline" | "pending";
}) => {
  const statusConfig = {
    online: { color: "bg-profit", ring: "ring-profit/30" },
    offline: { color: "bg-loss", ring: "ring-loss/30" },
    pending: { color: "bg-warning animate-pulse", ring: "ring-warning/30" },
  };

  return (
    <div className="flex items-center gap-2 px-3 py-1.5 rounded-full bg-white/5 border border-white/10">
      <div className={`w-2 h-2 rounded-full ${statusConfig[status].color} ring-2 ${statusConfig[status].ring}`} />
      <span className="text-xs font-medium text-muted-foreground">{label}</span>
    </div>
  );
};

// Notification Toggle Component
const NotificationToggle = ({
  icon,
  title,
  description,
  checked,
  onChange,
  disabled = false,
}: {
  icon: React.ReactNode;
  title: string;
  description: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
}) => (
  <div
    className="flex items-center justify-between p-4 rounded-xl border transition-colors"
    style={{
      backgroundColor: checked ? 'rgba(0, 217, 255, 0.1)' : luxuryColors.bgSecondary,
      borderColor: checked ? 'rgba(0, 217, 255, 0.2)' : luxuryColors.borderSubtle,
    }}
  >
    <div className="flex items-center gap-3">
      <div
        className="p-2 rounded-lg"
        style={{
          backgroundColor: checked ? 'rgba(0, 217, 255, 0.2)' : luxuryColors.bgTertiary,
          color: checked ? luxuryColors.cyan : luxuryColors.textMuted,
        }}
      >
        {icon}
      </div>
      <div>
        <div
          className="font-medium"
          style={{ color: checked ? luxuryColors.textPrimary : luxuryColors.textMuted }}
        >
          {title}
        </div>
        <div className="text-xs" style={{ color: luxuryColors.textMuted }}>
          {description}
        </div>
      </div>
    </div>
    <PremiumSwitch checked={checked} onCheckedChange={onChange} disabled={disabled} />
  </div>
);

// Save Button Component with states
const SaveButton = ({
  onClick,
  isLoading,
  isSaved,
}: {
  onClick: () => void;
  isLoading: boolean;
  isSaved: boolean;
}) => (
  <motion.div
    initial={{ opacity: 0, y: 10 }}
    animate={{ opacity: 1, y: 0 }}
    className="flex justify-end"
  >
    <PremiumButton
      onClick={onClick}
      disabled={isLoading}
      loading={isLoading}
      variant={isSaved ? "success" : "primary"}
      size="md"
    >
      {isSaved ? (
        <>
          <Check className="h-4 w-4" />
          Saved!
        </>
      ) : (
        <>
          <Save className="h-4 w-4" />
          Save Changes
        </>
      )}
    </PremiumButton>
  </motion.div>
);

export default Settings;
