import ErrorBoundary from "@/components/ErrorBoundary";
import { useTranslation } from "react-i18next";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Slider } from "@/components/ui/slider";
import { useState, useEffect } from "react";
import { usePaperTradingContext } from "@/contexts/PaperTradingContext";
import { useToast } from "@/hooks/use-toast";
import { useSecurity } from "@/hooks/useSecurity";
import { useNotificationPreferences, localToApiFormat } from "@/hooks/useNotificationPreferences";
import { usePushNotifications } from "@/hooks/usePushNotifications";
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
import { InlineTradingSettings } from "@/components/dashboard/TradingSettings";
import { PerSymbolSettings } from "@/components/dashboard/PerSymbolSettings";
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
  Cpu,
  HardDrive,
  Clock,
  Server,
  Database,
  Coins,
} from "lucide-react";
import { useThemeColors } from "@/hooks/useThemeColors";
import {
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
  const { t } = useTranslation('settings');
  const colors = useThemeColors();
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

  // Trading Pairs state - fetched from API
  const [tradingPairs, setTradingPairs] = useState<TradingPair[]>([]);
  const [isLoadingPairs, setIsLoadingPairs] = useState(true);

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

  // Fetch trading pairs from API (real data, not mock)
  useEffect(() => {
    const fetchTradingPairs = async () => {
      setIsLoadingPairs(true);
      try {
        // First fetch available symbols
        const symbolsResponse = await fetch(`${API_BASE}/api/market/symbols`);
        const symbolsData = await symbolsResponse.json();

        // Handle multiple API response formats:
        // 1. { success: true, data: { symbols: [...] } }
        // 2. { data: { symbols: [...] } }
        // 3. { symbols: [...] }
        const symbols: string[] = symbolsData.data?.symbols
          || symbolsData.symbols
          || (Array.isArray(symbolsData.data) ? symbolsData.data : []);

        if (symbols.length > 0) {

          // Then fetch symbol settings to get enabled status
          const settingsResponse = await fetch(`${API_BASE}/api/paper-trading/symbol-settings`);
          const settingsData = await settingsResponse.json();

          // Map symbols to trading pairs with settings
          // Handle multiple settingsData formats
          const settingsList = settingsData.data || settingsData.settings || (Array.isArray(settingsData) ? settingsData : []);

          const pairs: TradingPair[] = symbols.map(symbol => {
            // Format symbol for display (BTCUSDT -> BTC/USDT)
            const displaySymbol = symbol.replace(/USDT$/, '/USDT');

            // Find settings for this symbol
            const symbolSettings = Array.isArray(settingsList)
              ? settingsList.find((s: { symbol: string }) => s.symbol === symbol)
              : null;

            return {
              symbol: displaySymbol,
              enabled: symbolSettings?.enabled ?? true, // Default enabled
              leverage: symbolSettings?.leverage ?? 3,
              positionSize: symbolSettings?.position_size_pct ?? 25,
            };
          });

          setTradingPairs(pairs);
        }
      } catch (error) {
        console.error('Failed to fetch trading pairs:', error);
        // Fallback to empty array - no mock data
        setTradingPairs([]);
      } finally {
        setIsLoadingPairs(false);
      }
    };

    fetchTradingPairs();
  }, []);

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
        toast({ title: t('toast.botStarted'), description: t('toast.botStartedDesc') });
      } else {
        await stopBot();
        toast({ title: t('toast.botStopped'), description: t('toast.botStoppedDesc') });
      }
    } catch (error) {
      setBotActive(!checked);
      toast({ title: t('toast.botError'), description: t('toast.botErrorDesc'), variant: "destructive" });
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
        toast({ title: t('toast.connectionSuccess'), description: t('toast.connectionSuccessDesc') });
      } else {
        setConnectionStatus("disconnected");
        toast({ title: t('toast.connectionError'), description: t('toast.connectionFailedDesc'), variant: "destructive" });
      }
    } catch {
      setConnectionStatus("disconnected");
      toast({ title: t('toast.connectionError'), description: t('toast.connectionErrorDesc'), variant: "destructive" });
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
      toast({ title: t('toast.settingsSaved'), description: t('toast.sectionSaved', { section: section.charAt(0).toUpperCase() + section.slice(1) }) });
      setTimeout(() => setSavedSection(null), 2000);
    } catch (error) {
      toast({ title: t('toast.settingsError'), description: t('toast.saveError'), variant: "destructive" });
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

    if (diffMins < 1) return t('time.justNow');
    if (diffMins < 60) return diffMins === 1 ? t('time.minuteAgo', { count: diffMins }) : t('time.minutesAgo', { count: diffMins });
    if (diffHours < 24) return diffHours === 1 ? t('time.hourAgo', { count: diffHours }) : t('time.hoursAgo', { count: diffHours });
    return diffDays === 1 ? t('time.dayAgo', { count: diffDays }) : t('time.daysAgo', { count: diffDays });
  };

  // Handle 2FA verification
  const handleVerify2FA = async () => {
    if (verificationCode.length !== 6) {
      toast.error(t('toast.invalidCode'));
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
      toast.error(t('toast.invalidCode'));
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
      toast.success(t('toast.secretCopied'));
    }
  };

  // Section navigation items - matching old UI tabs
  const sectionNav = [
    { id: "bot" as const, label: t('tabs.bot'), icon: Settings2, description: t('bot.subtitle') },
    { id: "per-symbol" as const, label: t('tabs.perSymbol'), icon: Coins, description: t('perSymbol.subtitle') },
    { id: "strategy" as const, label: t('tabs.strategy'), icon: TrendingUp, description: t('strategy.subtitle') },
    { id: "health" as const, label: t('tabs.health'), icon: Activity, description: t('health.subtitle') },
    { id: "api" as const, label: t('tabs.api'), icon: Key, description: t('api.subtitle') },
    { id: "notifications" as const, label: t('tabs.notifications'), icon: Bell, description: t('notifications.subtitle') },
    { id: "security" as const, label: t('tabs.security'), icon: Shield, description: t('security.subtitle') },
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
            <GlowIcon icon={Settings2} size="lg" color={colors.cyan} />
            <div>
              <h1 className="text-2xl lg:text-3xl font-black">
                <GradientText>{t('page.title')}</GradientText>
              </h1>
              <p className="text-sm" style={{ color: colors.textMuted }}>
                {t('page.subtitle')}
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
                        ? `linear-gradient(to right, ${colors.cyan}20, transparent)`
                        : 'transparent',
                      borderColor: activeSection === section.id ? colors.cyan : 'transparent',
                    }}
                  >
                    <GlowIcon
                      icon={section.icon as React.ElementType}
                      size="sm"
                      color={activeSection === section.id ? colors.cyan : colors.textMuted}
                    />
                    <div className="flex-1 text-left">
                      <div
                        className="font-medium"
                        style={{
                          color: activeSection === section.id ? colors.textPrimary : colors.textMuted
                        }}
                      >
                        {section.label}
                      </div>
                      <div className="text-xs" style={{ color: colors.textMuted }}>
                        {section.description}
                      </div>
                    </div>
                    <ChevronRight
                      className={`h-4 w-4 transition-transform ${
                        activeSection === section.id ? "rotate-90" : ""
                      }`}
                      style={{
                        color: activeSection === section.id ? colors.cyan : colors.textMuted
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
                    title={t('bot.title')}
                    subtitle={t('bot.subtitle')}
                    icon={Settings2}
                  />

                  {/* Bot Configuration Card */}
                  <GlassCard>
                    <div className="flex items-center justify-between mb-6">
                      <h3 className="font-semibold text-lg" style={{ color: colors.textPrimary }}>
                        {t('bot.configuration')}
                      </h3>
                      <Badge variant={botActive ? "success" : "default"} glow={botActive}>
                        {botActive ? t('bot.active') : t('bot.inactive')}
                      </Badge>
                    </div>

                    {/* Bot Status */}
                    <div
                      className="flex items-center justify-between p-4 rounded-xl mb-6"
                      style={{ backgroundColor: colors.bgSecondary }}
                    >
                      <div>
                        <h4 className="font-medium" style={{ color: colors.textPrimary }}>
                          {t('bot.status')}
                        </h4>
                        <p className="text-sm" style={{ color: colors.textMuted }}>
                          {botActive ? t('bot.statusActive') : t('bot.statusInactive')}
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
                        label={t('bot.capitalAllocation')}
                        value={capitalAllocation[0]}
                        unit="%"
                        min={1}
                        max={100}
                        step={1}
                        onChange={(v) => setCapitalAllocation([v])}
                        color="primary"
                        description={t('bot.amount', { value: ((currentBalance * capitalAllocation[0]) / 100).toFixed(2) })}
                      />
                      <div className="flex justify-between text-xs" style={{ color: colors.textMuted }}>
                        <span>{t('bot.conservativePercent', { value: 10 })}</span>
                        <span>{t('bot.aggressivePercent', { value: 100 })}</span>
                      </div>

                      {/* Maximum Leverage */}
                      <SliderSetting
                        label={t('bot.maxLeverage')}
                        value={maxLeverage[0]}
                        unit="x"
                        min={1}
                        max={20}
                        step={1}
                        onChange={(v) => setMaxLeverage([v])}
                        color="warning"
                      />
                      <div className="flex justify-between text-xs" style={{ color: colors.textMuted }}>
                        <span>{t('bot.safeLeverage', { value: 1 })}</span>
                        <span>{t('bot.highRiskLeverage', { value: 20 })}</span>
                      </div>

                      {/* Risk Threshold */}
                      <SliderSetting
                        label={t('bot.riskThreshold')}
                        value={riskThreshold[0]}
                        unit="%"
                        min={1}
                        max={15}
                        step={0.5}
                        onChange={(v) => setRiskThreshold([v])}
                        color="loss"
                        description={t('bot.maxLossPerTrade', { value: ((currentBalance * riskThreshold[0]) / 100).toFixed(2) })}
                      />
                      <div className="flex justify-between text-xs" style={{ color: colors.textMuted }}>
                        <span>{t('bot.conservativePercent', { value: 1 })}</span>
                        <span>{t('bot.aggressivePercent', { value: 15 })}</span>
                      </div>
                    </div>
                  </GlassCard>

                  {/* Active Trading Pairs - fetched from API */}
                  <GlassCard>
                    <h3 className="font-semibold mb-4" style={{ color: colors.textPrimary }}>
                      {t('bot.activeTradingPairs')}
                    </h3>
                    {isLoadingPairs ? (
                      <div className="flex items-center justify-center py-8">
                        <Loader2 className="h-6 w-6 animate-spin" style={{ color: colors.cyan }} />
                        <span className="ml-2 text-sm" style={{ color: colors.textMuted }}>
                          {t('common.loading') || 'Loading...'}
                        </span>
                      </div>
                    ) : tradingPairs.length === 0 ? (
                      <div className="text-center py-8">
                        <GlowIcon icon={AlertTriangle} size="lg" color={colors.warning} className="mx-auto mb-4" />
                        <p style={{ color: colors.textMuted }}>
                          {t('bot.noPairsAvailable') || 'No trading pairs available from API'}
                        </p>
                      </div>
                    ) : (
                      <div className="grid grid-cols-2 gap-4">
                        {tradingPairs.map((pair) => (
                          <div
                            key={pair.symbol}
                            className="flex items-center justify-between p-3 rounded-xl"
                            style={{ backgroundColor: colors.bgSecondary }}
                          >
                            <span style={{ color: colors.textPrimary }}>{pair.symbol}</span>
                            <PremiumSwitch
                              checked={pair.enabled}
                              onCheckedChange={() => toggleTradingPair(pair.symbol)}
                            />
                          </div>
                        ))}
                      </div>
                    )}
                  </GlassCard>

                  <SaveButton
                    onClick={() => saveSettings("bot")}
                    isLoading={isSaving}
                    isSaved={savedSection === "bot"}
                  />
                </div>
              )}

              {/* Per-Symbol Settings Section - Using PerSymbolSettings component with real API */}
              {activeSection === "per-symbol" && (
                <div className="space-y-6">
                  <SectionHeader
                    title={t('perSymbol.title')}
                    subtitle={t('perSymbol.subtitle')}
                    icon={Coins}
                  />

                  {/* PerSymbolSettings component fetches symbols from API and saves to backend */}
                  <PerSymbolSettings
                    currentBalance={currentBalance}
                    onSettingsUpdate={(configs) => {
                      // Sync with local tradingPairs state for Bot Settings section
                      const updatedPairs = configs.map(config => ({
                        symbol: config.symbol.replace(/USDT$/, '/USDT'),
                        enabled: config.enabled,
                        leverage: config.leverage,
                        positionSize: config.position_size_pct,
                      }));
                      setTradingPairs(updatedPairs);
                      toast({ title: t('toast.settingsSaved'), description: t('toast.symbolSettingsSynced') });
                    }}
                  />
                </div>
              )}

              {/* Strategy Tuning Section */}
              {activeSection === "strategy" && (
                <div className="space-y-6">
                  <SectionHeader
                    title={t('strategy.title')}
                    subtitle={t('strategy.subtitle')}
                    icon={TrendingUp}
                  />

                  {/* Inline Trading Settings - all settings displayed directly on page */}
                  <InlineTradingSettings />
                </div>
              )}

              {/* System Health Section */}
              {activeSection === "health" && (
                <div className="space-y-6">
                  <SectionHeader
                    title={t('health.title')}
                    subtitle={t('health.subtitle')}
                    icon={Activity}
                  />

                  {/* System Status Overview */}
                  <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                    <GlassCard noPadding className="p-4">
                      <div className="flex items-center gap-3">
                        <GlowIcon icon={Cpu} size="sm" color={systemHealth.cpu < 70 ? colors.profit : colors.loss} />
                        <div>
                          <p className="text-xs" style={{ color: colors.textMuted }}>{t('health.cpu')}</p>
                          <p className="font-bold" style={{ color: colors.textPrimary }}>{systemHealth.cpu}%</p>
                        </div>
                      </div>
                    </GlassCard>
                    <GlassCard noPadding className="p-4">
                      <div className="flex items-center gap-3">
                        <GlowIcon icon={HardDrive} size="sm" color={systemHealth.memory < 80 ? colors.profit : colors.loss} />
                        <div>
                          <p className="text-xs" style={{ color: colors.textMuted }}>{t('health.memory')}</p>
                          <p className="font-bold" style={{ color: colors.textPrimary }}>{systemHealth.memory}%</p>
                        </div>
                      </div>
                    </GlassCard>
                    <GlassCard noPadding className="p-4">
                      <div className="flex items-center gap-3">
                        <GlowIcon icon={Clock} size="sm" color={colors.cyan} />
                        <div>
                          <p className="text-xs" style={{ color: colors.textMuted }}>{t('health.uptime')}</p>
                          <p className="font-bold" style={{ color: colors.textPrimary }}>{systemHealth.uptime}</p>
                        </div>
                      </div>
                    </GlassCard>
                    <GlassCard noPadding className="p-4">
                      <div className="flex items-center gap-3">
                        <GlowIcon icon={Zap} size="sm" color={systemHealth.apiLatency < 100 ? colors.profit : colors.warning} />
                        <div>
                          <p className="text-xs" style={{ color: colors.textMuted }}>{t('health.apiLatency')}</p>
                          <p className="font-bold" style={{ color: colors.textPrimary }}>{systemHealth.apiLatency}ms</p>
                        </div>
                      </div>
                    </GlassCard>
                  </div>

                  {/* Connection Status */}
                  <GlassCard>
                    <h3 className="font-semibold mb-4" style={{ color: colors.textPrimary }}>
                      {t('health.connectionStatus')}
                    </h3>
                    <div className="space-y-3">
                      <div className="flex items-center justify-between p-3 rounded-xl" style={{ backgroundColor: colors.bgSecondary }}>
                        <div className="flex items-center gap-3">
                          <GlowIcon icon={Server} size="sm" color={connectionStatus === "connected" ? colors.profit : colors.loss} />
                          <span style={{ color: colors.textPrimary }}>{t('health.apiServer')}</span>
                        </div>
                        <StatusIndicator label="" status={connectionStatus === "connected" ? "online" : "offline"} />
                      </div>
                      <div className="flex items-center justify-between p-3 rounded-xl" style={{ backgroundColor: colors.bgSecondary }}>
                        <div className="flex items-center gap-3">
                          <GlowIcon icon={Wifi} size="sm" color={systemHealth.wsConnected ? colors.profit : colors.loss} />
                          <span style={{ color: colors.textPrimary }}>{t('health.wsStatus')}</span>
                        </div>
                        <StatusIndicator label="" status={systemHealth.wsConnected ? "online" : "offline"} />
                      </div>
                      <div className="flex items-center justify-between p-3 rounded-xl" style={{ backgroundColor: colors.bgSecondary }}>
                        <div className="flex items-center gap-3">
                          <GlowIcon icon={Database} size="sm" color={systemHealth.dbConnected ? colors.profit : colors.loss} />
                          <span style={{ color: colors.textPrimary }}>{t('health.dbStatus')}</span>
                        </div>
                        <StatusIndicator label="" status={systemHealth.dbConnected ? "online" : "offline"} />
                      </div>
                    </div>
                    <p className="text-xs mt-4" style={{ color: colors.textMuted }}>
                      {t('health.lastUpdate')}: {systemHealth.lastUpdate}
                    </p>
                  </GlassCard>

                  <PremiumButton variant="secondary" onClick={testConnection} disabled={isTestingConnection} loading={isTestingConnection} fullWidth>
                    <RefreshCw className="h-4 w-4" />
                    {t('health.refresh')}
                  </PremiumButton>
                </div>
              )}

              {/* API & Connections Section */}
              {activeSection === "api" && (
                <div className="space-y-6">
                  <SectionHeader
                    title={t('api.title')}
                    subtitle={t('api.subtitle')}
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
                              connectionStatus === "connected" ? colors.profit :
                              connectionStatus === "testing" ? colors.warning :
                              colors.loss
                            }
                            className={connectionStatus === "testing" ? "animate-spin" : ""}
                          />
                          <div>
                            <h3 className="font-semibold" style={{ color: colors.textPrimary }}>
                              Connection Status
                            </h3>
                            <p className="text-sm" style={{ color: colors.textMuted }}>
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
                          <GlowIcon icon={Key} size="sm" color={colors.cyan} />
                          <h3 className="font-semibold" style={{ color: colors.textPrimary }}>
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
                          <Label htmlFor="api-key" className="text-xs uppercase tracking-wider" style={{ color: colors.textMuted }}>
                            API Key
                          </Label>
                          <button
                            type="button"
                            onClick={() => setShowApiKey(!showApiKey)}
                            className="transition-colors"
                            style={{ color: colors.textMuted }}
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
                          <Label htmlFor="secret-key" className="text-xs uppercase tracking-wider" style={{ color: colors.textMuted }}>
                            Secret Key
                          </Label>
                          <button
                            type="button"
                            onClick={() => setShowSecretKey(!showSecretKey)}
                            className="transition-colors"
                            style={{ color: colors.textMuted }}
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
                        <Lock className="h-5 w-5 mt-0.5" style={{ color: colors.cyan }} />
                        <div>
                          <p className="text-sm font-medium" style={{ color: colors.cyan }}>
                            Security Note
                          </p>
                          <p className="text-xs mt-1" style={{ color: colors.textMuted }}>
                            API keys are encrypted and stored securely. Only grant Futures Trading permission to the bot.
                          </p>
                        </div>
                      </div>
                    </div>
                  </GlassCard>

                  {/* Trading Permissions */}
                  <GlassCard>
                    <h3 className="font-semibold mb-4" style={{ color: colors.textPrimary }}>
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
                            backgroundColor: permission.enabled ? 'rgba(34, 197, 94, 0.1)' : colors.bgSecondary,
                            borderColor: permission.enabled ? 'rgba(34, 197, 94, 0.2)' : colors.borderSubtle,
                          }}
                        >
                          <div>
                            <div className="font-medium" style={{ color: colors.textPrimary }}>
                              {permission.name}
                            </div>
                            <div className="text-sm" style={{ color: colors.textMuted }}>
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
                    title={t('notifications.title')}
                    subtitle={t('notifications.subtitle')}
                    icon={Bell}
                  />

                  {/* Notification Channels */}
                  <GlassCard>
                    <h3 className="font-semibold mb-6" style={{ color: colors.textPrimary }}>
                      {t('notifications.channels')}
                    </h3>
                      <div className="space-y-4">
                        <NotificationToggle
                          icon={<Mail className="h-5 w-5" />}
                          title={t('notifications.email.title')}
                          description={t('notifications.email.description')}
                          checked={notifications.email}
                          onChange={(checked) => setNotifications(prev => ({ ...prev, email: checked }))}
                        />
                        <NotificationToggle
                          icon={<Bell className="h-5 w-5" />}
                          title={t('notifications.push.title')}
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
                          <Bell className="h-5 w-5" style={{ color: colors.cyan }} />
                          <span className="font-medium" style={{ color: colors.textPrimary }}>
                            Push Notification Keys (VAPID)
                          </span>
                        </div>
                        <p className="text-xs mb-3" style={{ color: colors.textMuted }}>
                          Generate VAPID keys using: <code className="px-1 py-0.5 rounded bg-black/30">npx web-push generate-vapid-keys</code>
                        </p>
                        <PremiumInput
                          label={t('notifications.vapid.publicKeyLabel')}
                          value={vapidPublicKey}
                          onChange={setVapidPublicKey}
                          placeholder={t('notifications.vapid.publicKeyPlaceholder')}
                        />
                        <PremiumInput
                          label={t('notifications.vapid.privateKeyLabel')}
                          type="password"
                          value={vapidPrivateKey}
                          onChange={setVapidPrivateKey}
                          placeholder={t('notifications.vapid.privateKeyPlaceholder')}
                        />
                        <div className="flex justify-end">
                          <PremiumButton
                            variant="secondary"
                            size="sm"
                            onClick={async () => {
                              await saveSettings("notifications");
                              toast({ title: t('notifications.vapid.keysSaved'), description: t('notifications.vapid.keysSavedDesc') });
                            }}
                            disabled={!vapidPublicKey}
                          >
                            {t('notifications.vapid.saveKeys')}
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
                            label={t('notifications.discord.webhookLabel')}
                            value={discordWebhookUrl}
                            onChange={setDiscordWebhookUrl}
                            placeholder={t('notifications.discord.webhookPlaceholder')}
                          />
                          <div className="flex items-center justify-between mt-3">
                            <p className="text-xs" style={{ color: colors.textMuted }}>
                              {t('notifications.discord.webhookHint')}
                            </p>
                            <PremiumButton
                              variant="secondary"
                              size="sm"
                              onClick={async () => {
                                // Save first, then test
                                await saveSettings("notifications");
                                const success = await testNotification("discord");
                                if (success) {
                                  toast({ title: t('notifications.discord.testSent'), description: t('notifications.discord.testSentDesc') });
                                } else {
                                  toast({ title: t('notifications.discord.testFailed'), description: t('notifications.discord.testFailedDesc'), variant: "destructive" });
                                }
                              }}
                              disabled={!discordWebhookUrl}
                            >
                              {t('notifications.discord.testButton')}
                            </PremiumButton>
                          </div>
                        </motion.div>
                      )}
                    </AnimatePresence>
                  </GlassCard>

                  {/* Alert Types */}
                  <GlassCard>
                    <h3 className="font-semibold mb-6" style={{ color: colors.textPrimary }}>
                      {t('notifications.alerts.title')}
                    </h3>
                      <div className="space-y-4">
                        <NotificationToggle
                          icon={<TrendingUp className="h-5 w-5" />}
                          title={t('notifications.alerts.price.title')}
                          description={t('notifications.alerts.price.description')}
                          checked={notifications.priceAlerts}
                          onChange={(checked) => setNotifications(prev => ({ ...prev, priceAlerts: checked }))}
                        />
                        <NotificationToggle
                          icon={<Zap className="h-5 w-5" />}
                          title={t('notifications.alerts.trade.title')}
                          description={t('notifications.alerts.trade.description')}
                          checked={notifications.tradeAlerts}
                          onChange={(checked) => setNotifications(prev => ({ ...prev, tradeAlerts: checked }))}
                        />
                        <NotificationToggle
                          icon={<AlertTriangle className="h-5 w-5" />}
                          title={t('notifications.alerts.system.title')}
                          description={t('notifications.alerts.system.description')}
                          checked={notifications.systemAlerts}
                          onChange={(checked) => setNotifications(prev => ({ ...prev, systemAlerts: checked }))}
                        />
                        <NotificationToggle
                          icon={<Volume2 className="h-5 w-5" />}
                          title={t('notifications.sound.title')}
                          description={t('notifications.sound.description')}
                          checked={notifications.sound}
                          onChange={(checked) => setNotifications(prev => ({ ...prev, sound: checked }))}
                        />
                      </div>
                  </GlassCard>

                  {/* Alert Threshold */}
                  <GlassCard>
                      <SliderSetting
                        label={t('notifications.alerts.thresholdLabel')}
                        value={alertThreshold[0]}
                        unit="%"
                        min={1}
                        max={20}
                        step={1}
                        onChange={(v) => setAlertThreshold([v])}
                        color="warning"
                        description={t('notifications.alerts.thresholdDesc')}
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
                    title={t('security.title')}
                    subtitle={t('security.subtitle')}
                    icon={Shield}
                  />

                  {/* Profile Info */}
                  <GlassCard>
                    <div className="flex items-center gap-4 mb-6">
                      <GlowIcon icon={User} size="lg" color={colors.cyan} />
                      <div>
                        <h3 className="font-semibold text-lg" style={{ color: colors.textPrimary }}>
                          {t('security.profile.title')}
                        </h3>
                        <p className="text-sm" style={{ color: colors.textMuted }}>
                          trader@example.com
                        </p>
                      </div>
                    </div>

                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                      <PremiumInput
                        label={t('security.profile.displayName')}
                        value="Crypto Trader"
                        onChange={() => {}}
                      />
                      <PremiumInput
                        label={t('security.profile.email')}
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
                          color={twoFactorEnabled ? colors.profit : colors.textMuted}
                        />
                        <div>
                          <h3 className="font-semibold" style={{ color: colors.textPrimary }}>
                            {t('security.twoFactor.title')}
                          </h3>
                          <p className="text-sm" style={{ color: colors.textMuted }}>
                            {twoFactorEnabled ? t('security.twoFactor.protected') : t('security.twoFactor.description')}
                          </p>
                        </div>
                      </div>
                      <Badge variant={twoFactorEnabled ? "success" : "warning"}>
                        {twoFactorEnabled ? t('security.twoFactor.enabled') : t('security.twoFactor.disabled')}
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
                          {t('security.twoFactor.settingUp')}
                        </>
                      ) : twoFactorEnabled ? t('security.twoFactor.disable') : t('security.twoFactor.enable')}
                    </PremiumButton>
                  </GlassCard>

                  {/* Change Password */}
                  <GlassCard>
                    <h3 className="font-semibold mb-4" style={{ color: colors.textPrimary }}>
                      {t('security.password.title')}
                    </h3>
                    <form onSubmit={async (e) => {
                      e.preventDefault();
                      if (passwordData.new !== passwordData.confirm) {
                        toast.error(t('security.password.mismatch'));
                        return;
                      }
                      if (passwordData.new.length < 8) {
                        toast.error(t('security.password.tooShort'));
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
                          placeholder={t('security.password.currentPlaceholder')}
                          disabled={isChangingPassword}
                        />
                        <PremiumInput
                          type="password"
                          value={passwordData.new}
                          onChange={(e) => setPasswordData({ ...passwordData, new: e.target.value })}
                          placeholder={t('security.password.newPlaceholder')}
                          disabled={isChangingPassword}
                        />
                        <PremiumInput
                          type="password"
                          value={passwordData.confirm}
                          onChange={(e) => setPasswordData({ ...passwordData, confirm: e.target.value })}
                          placeholder={t('security.password.confirmPlaceholder')}
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
                              {t('security.password.updating')}
                            </>
                          ) : t('security.password.update')}
                        </PremiumButton>
                      </div>
                    </form>
                  </GlassCard>

                  {/* Active Sessions */}
                  <GlassCard>
                    <h3 className="font-semibold mb-4" style={{ color: colors.textPrimary }}>
                      {t('security.sessions.title')}
                    </h3>
                    <div className="space-y-3">
                      {isLoadingSessions ? (
                        <div className="flex justify-center py-8">
                          <Loader2 className="w-6 h-6 animate-spin" style={{ color: colors.textMuted }} />
                        </div>
                      ) : sessions.length === 0 ? (
                        <p className="text-center py-4" style={{ color: colors.textMuted }}>
                          {t('security.sessions.noSessions')}
                        </p>
                      ) : (
                        sessions.map((session) => (
                          <div
                            key={session.session_id}
                            className="flex items-center justify-between p-4 rounded-xl border"
                            style={{
                              backgroundColor: colors.bgSecondary,
                              borderColor: colors.borderSubtle,
                            }}
                          >
                            <div className="flex items-center gap-3">
                              <GlowIcon
                                icon={Smartphone}
                                size="sm"
                                color={session.is_current ? colors.profit : colors.textMuted}
                              />
                              <div>
                                <div className="font-medium" style={{ color: colors.textPrimary }}>
                                  {session.browser} on {session.os}
                                </div>
                                <div className="text-xs" style={{ color: colors.textMuted }}>
                                  {session.location}
                                </div>
                              </div>
                            </div>
                            <div className="text-right">
                              <div
                                className="text-sm"
                                style={{
                                  color: session.is_current ? colors.profit : colors.textMuted
                                }}
                              >
                                {session.is_current ? t('security.sessions.activeNow') : formatLastActive(session.last_active)}
                              </div>
                              {!session.is_current && (
                                <button
                                  className="text-xs hover:underline"
                                  style={{ color: colors.loss }}
                                  onClick={() => revokeSession(session.session_id)}
                                >
                                  {t('security.sessions.revoke')}
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
                            {t('security.sessions.signOutAll')}
                          </PremiumButton>
                        </AlertDialogTrigger>
                        <AlertDialogContent className="bg-slate-900 border-slate-700">
                          <AlertDialogHeader>
                            <AlertDialogTitle className="text-gray-100">
                              {t('security.sessions.signOutAllTitle')}
                            </AlertDialogTitle>
                            <AlertDialogDescription className="text-gray-400">
                              {t('security.sessions.signOutAllDesc')}
                            </AlertDialogDescription>
                          </AlertDialogHeader>
                          <AlertDialogFooter>
                            <AlertDialogCancel className="border-slate-700">{t('common.cancel')}</AlertDialogCancel>
                            <AlertDialogAction
                              onClick={revokeAllSessions}
                              className="bg-red-600 hover:bg-red-700"
                            >
                              {t('security.sessions.signOutAllButton')}
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
              background: `linear-gradient(135deg, ${colors.cardBg} 0%, rgba(15, 23, 42, 0.98) 100%)`,
              boxShadow: `0 25px 50px -12px rgba(0, 0, 0, 0.8), 0 0 40px ${colors.cyan}15`,
              backdropFilter: 'blur(20px)',
            }}
          >
            <div className="p-6">
              <DialogHeader className="mb-6">
                <div className="flex items-center gap-3 mb-2">
                  <div
                    className="p-2 rounded-lg"
                    style={{ background: `${colors.cyan}20` }}
                  >
                    <Shield className="w-5 h-5" style={{ color: colors.cyan }} />
                  </div>
                  <DialogTitle style={{ color: colors.text }}>
                    {t('security.twoFactor.setupTitle')}
                  </DialogTitle>
                </div>
                <DialogDescription style={{ color: colors.textMuted }}>
                  {t('security.twoFactor.setupDescription')}
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
                      boxShadow: `0 4px 20px ${colors.cyan}30`
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
                    <Label style={{ color: colors.textSecondary }}>{t('security.twoFactor.manualEntryLabel')}</Label>
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
                    <p className="text-xs" style={{ color: colors.textMuted }}>
                      {t('security.twoFactor.manualEntryHint')}
                    </p>
                  </div>

                  {/* Verification Code Input */}
                  <div className="space-y-2">
                    <Label htmlFor="verificationCode" style={{ color: colors.textSecondary }}>
                      {t('security.twoFactor.verificationCodeLabel')}
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
                  {t('common.cancel')}
                </PremiumButton>
                <PremiumButton
                  variant="success"
                  onClick={handleVerify2FA}
                  disabled={isSecurityLoading || verificationCode.length !== 6}
                >
                  {isSecurityLoading ? (
                    <>
                      <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                      {t('security.twoFactor.verifying')}
                    </>
                  ) : (
                    t('security.twoFactor.enable')
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
              background: `linear-gradient(135deg, ${colors.cardBg} 0%, rgba(15, 23, 42, 0.98) 100%)`,
              boxShadow: `0 25px 50px -12px rgba(0, 0, 0, 0.8), 0 0 40px ${colors.red}15`,
              backdropFilter: 'blur(20px)',
            }}
          >
            <div className="p-6">
              <DialogHeader className="mb-6">
                <div className="flex items-center gap-3 mb-2">
                  <div
                    className="p-2 rounded-lg"
                    style={{ background: `${colors.red}20` }}
                  >
                    <AlertTriangle className="w-5 h-5" style={{ color: colors.red }} />
                  </div>
                  <DialogTitle style={{ color: colors.text }}>
                    {t('security.twoFactor.disableTitle')}
                  </DialogTitle>
                </div>
                <DialogDescription style={{ color: colors.textMuted }}>
                  {t('security.twoFactor.disableDescription')}
                </DialogDescription>
              </DialogHeader>

              <div className="space-y-2">
                <Label htmlFor="disableCode" style={{ color: colors.textSecondary }}>
                  {t('security.twoFactor.currentCodeLabel')}
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
                  {t('common.cancel')}
                </PremiumButton>
                <PremiumButton
                  variant="danger"
                  onClick={handleDisable2FA}
                  disabled={isSecurityLoading || disableCode.length !== 6}
                >
                  {isSecurityLoading ? (
                    <>
                      <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                      {t('security.twoFactor.disabling')}
                    </>
                  ) : (
                    t('security.twoFactor.disable')
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
// Uses base Switch component which already has the correct cyan-emerald gradient
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
}) => {
  const colors = useThemeColors();
  return (
    <div
      className="flex items-center justify-between p-4 rounded-xl border transition-colors"
      style={{
        backgroundColor: checked ? 'rgba(0, 217, 255, 0.1)' : colors.bgSecondary,
        borderColor: checked ? 'rgba(0, 217, 255, 0.2)' : colors.borderSubtle,
      }}
    >
      <div className="flex items-center gap-3">
        <div
          className="p-2 rounded-lg"
          style={{
            backgroundColor: checked ? 'rgba(0, 217, 255, 0.2)' : colors.bgTertiary,
            color: checked ? colors.cyan : colors.textMuted,
          }}
        >
          {icon}
        </div>
        <div>
          <div
            className="font-medium"
            style={{ color: checked ? colors.textPrimary : colors.textMuted }}
          >
            {title}
          </div>
          <div className="text-xs" style={{ color: colors.textMuted }}>
            {description}
          </div>
        </div>
      </div>
      <PremiumSwitch checked={checked} onCheckedChange={onChange} disabled={disabled} />
    </div>
  );
};

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
