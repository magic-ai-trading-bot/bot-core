import { motion } from "framer-motion";
import { Link } from "react-router-dom";
import { useTranslation } from "react-i18next";
import {
  Code,
  Terminal,
  Key,
  Webhook,
  Zap,
  ArrowLeft,
  Copy,
  ExternalLink,
  FileText,
} from "lucide-react";
import {
  GlassCard,
  GradientText,
  PremiumButton,
  Badge,
  GlowIcon,
  PageWrapper,
} from "@/styles/luxury-design-system";
import { useThemeColors } from "@/hooks/useThemeColors";
import { useState } from "react";
import { toast } from "sonner";

const featureIcons = [Code, Webhook, Terminal, FileText];
const featureKeys = ["rest", "websocket", "sdks", "docs"];

const endpoints = [
  {
    method: "GET",
    path: "/api/v1/market/price/:symbol",
    description: "Get current price for a trading pair",
    example: '{ "symbol": "BTCUSDT", "price": "43250.50", "change24h": "+2.5%" }',
  },
  {
    method: "POST",
    path: "/api/v1/trading/order",
    description: "Place a new trading order",
    example: '{ "symbol": "BTCUSDT", "side": "BUY", "quantity": 0.01, "type": "MARKET" }',
  },
  {
    method: "GET",
    path: "/api/v1/portfolio/positions",
    description: "Get all open positions",
    example: '{ "positions": [...], "total_pnl": "+$1,234.56" }',
  },
  {
    method: "GET",
    path: "/api/v1/signals/latest",
    description: "Get latest AI trading signals",
    example: '{ "signals": [{ "symbol": "BTCUSDT", "action": "BUY", "confidence": 0.85 }] }',
  },
];

const sdks = [
  { name: "Python", version: "1.2.0", icon: "🐍" },
  { name: "JavaScript", version: "1.1.5", icon: "📦" },
  { name: "Go", version: "1.0.3", icon: "🔵" },
  { name: "Rust", version: "0.9.1", icon: "🦀" },
];

const API = () => {
  const colors = useThemeColors();
  const { t } = useTranslation('pages');
  const [activeEndpoint, setActiveEndpoint] = useState(0);

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    toast.success("Copied to clipboard!");
  };

  return (
    <PageWrapper>
      {/* Back Button */}
      <motion.div
        initial={{ opacity: 0, x: -20 }}
        animate={{ opacity: 1, x: 0 }}
        className="mb-8"
      >
        <Link to="/">
          <PremiumButton variant="secondary" size="sm">
            <ArrowLeft className="w-4 h-4" />
            {t('common.backToHome')}
          </PremiumButton>
        </Link>
      </motion.div>

      {/* Hero Section */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        className="text-center mb-16"
      >
        <Badge variant="info" className="mb-4">
          {t('api.badge')}
        </Badge>
        <h1 className="text-4xl md:text-5xl font-black mb-4">
          <GradientText>{t('api.title')}</GradientText>
          <br />
          <span style={{ color: colors.textPrimary }}>{t('api.subtitle')}</span>
        </h1>
        <p className="text-lg max-w-2xl mx-auto" style={{ color: colors.textMuted }}>
          {t('api.description')}
        </p>
      </motion.div>

      {/* Features */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.1 }}
        className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-12"
      >
        {featureKeys.map((key, index) => {
          const Icon = featureIcons[index];
          return (
            <GlassCard key={key} noPadding className="p-4 text-center">
              <GlowIcon icon={Icon} size="md" color={colors.cyan} className="mx-auto mb-3" />
              <h3 className="font-semibold mb-1" style={{ color: colors.textPrimary }}>
                {t(`api.features.${key}.title`)}
              </h3>
              <p className="text-xs" style={{ color: colors.textMuted }}>
                {t(`api.features.${key}.description`)}
              </p>
            </GlassCard>
          );
        })}
      </motion.div>

      {/* Quick Start */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.2 }}
        className="mb-12"
      >
        <GlassCard>
          <div className="flex items-center gap-3 mb-4">
            <GlowIcon icon={Zap} size="md" color={colors.cyan} />
            <div>
              <h2 className="text-xl font-bold" style={{ color: colors.textPrimary }}>
                {t('api.quickStart.title')}
              </h2>
              <p className="text-sm" style={{ color: colors.textMuted }}>
                {t('api.quickStart.description')}
              </p>
            </div>
          </div>

          {/* Endpoint List */}
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            <div className="space-y-2">
              {endpoints.map((endpoint, index) => (
                <button
                  key={endpoint.path}
                  onClick={() => setActiveEndpoint(index)}
                  className={`w-full text-left p-3 rounded-lg border transition-all ${
                    activeEndpoint === index
                      ? 'border-cyan-500/50 bg-cyan-500/10'
                      : 'border-transparent hover:bg-white/5'
                  }`}
                  style={{ borderColor: activeEndpoint === index ? colors.cyan : 'transparent' }}
                >
                  <div className="flex items-center gap-2 mb-1">
                    <Badge
                      variant={endpoint.method === "GET" ? "success" : "warning"}
                      size="sm"
                    >
                      {endpoint.method}
                    </Badge>
                    <code className="text-xs" style={{ color: colors.textPrimary }}>
                      {endpoint.path.split('/').slice(-1)[0]}
                    </code>
                  </div>
                  <p className="text-xs" style={{ color: colors.textMuted }}>
                    {endpoint.description}
                  </p>
                </button>
              ))}
            </div>

            {/* Code Preview */}
            <div className="lg:col-span-2">
              <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-2">
                  <Code className="w-4 h-4" style={{ color: colors.cyan }} />
                  <span className="text-sm font-mono" style={{ color: colors.textPrimary }}>
                    {endpoints[activeEndpoint].path}
                  </span>
                </div>
                <button
                  onClick={() => copyToClipboard(endpoints[activeEndpoint].example)}
                  className="p-2 rounded hover:bg-white/10 transition-colors"
                >
                  <Copy className="w-4 h-4" style={{ color: colors.textMuted }} />
                </button>
              </div>
              <pre
                className="p-4 rounded-lg overflow-x-auto text-sm font-mono"
                style={{ backgroundColor: colors.bgPrimary, color: colors.cyan }}
              >
                {endpoints[activeEndpoint].example}
              </pre>
            </div>
          </div>
        </GlassCard>
      </motion.div>

      {/* SDKs */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.3 }}
        className="mb-12"
      >
        <h2 className="text-2xl font-bold mb-6" style={{ color: colors.textPrimary }}>
          Official SDKs
        </h2>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          {sdks.map((sdk) => (
            <GlassCard key={sdk.name} noPadding className="p-4 text-center hover:border-cyan-500/30 transition-all cursor-pointer">
              <span className="text-3xl mb-2 block">{sdk.icon}</span>
              <h4 className="font-semibold" style={{ color: colors.textPrimary }}>{sdk.name}</h4>
              <p className="text-xs" style={{ color: colors.textMuted }}>v{sdk.version}</p>
            </GlassCard>
          ))}
        </div>
      </motion.div>

      {/* CTA */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.4 }}
      >
        <GlassCard className="text-center">
          <GlowIcon icon={Webhook} size="lg" color={colors.cyan} className="mx-auto mb-4" />
          <h2 className="text-2xl font-bold mb-4" style={{ color: colors.textPrimary }}>
            Ready to Build?
          </h2>
          <p className="mb-6" style={{ color: colors.textMuted }}>
            Check out our full documentation for detailed guides and examples.
          </p>
          <div className="flex justify-center gap-4">
            <Link to="/docs">
              <PremiumButton variant="primary">
                {t('api.viewDocs')}
                <ExternalLink className="w-4 h-4" />
              </PremiumButton>
            </Link>
            <Link to="/settings">
              <PremiumButton variant="secondary">
                {t('api.getApiKey')}
                <Key className="w-4 h-4" />
              </PremiumButton>
            </Link>
          </div>
        </GlassCard>
      </motion.div>
    </PageWrapper>
  );
};

export default API;
