import { motion } from "framer-motion";
import { Link } from "react-router-dom";
import {
  Code,
  Terminal,
  Key,
  Webhook,
  Database,
  Zap,
  ArrowLeft,
  Copy,
  ExternalLink,
} from "lucide-react";
import {
  luxuryColors,
  GlassCard,
  GradientText,
  PremiumButton,
  Badge,
  GlowIcon,
  PageWrapper,
} from "@/styles/luxury-design-system";
import { useState } from "react";
import { toast } from "sonner";

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
  {
    method: "POST",
    path: "/api/v1/webhooks/register",
    description: "Register a webhook for events",
    example: '{ "url": "https://your-server.com/webhook", "events": ["trade", "signal"] }',
  },
];

const sdks = [
  { name: "Python", version: "1.2.0", icon: "🐍" },
  { name: "JavaScript", version: "1.1.5", icon: "📦" },
  { name: "Go", version: "1.0.3", icon: "🔵" },
  { name: "Rust", version: "0.9.1", icon: "🦀" },
];

const API = () => {
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
            Back to Home
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
          Developer API
        </Badge>
        <h1 className="text-4xl md:text-5xl font-black mb-4">
          <GradientText>Build With</GradientText>
          <br />
          <span style={{ color: luxuryColors.textPrimary }}>Bot Core API</span>
        </h1>
        <p className="text-lg max-w-2xl mx-auto" style={{ color: luxuryColors.textMuted }}>
          Integrate powerful trading capabilities into your applications with our RESTful API and WebSocket streams.
        </p>
      </motion.div>

      {/* Quick Start */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.1 }}
        className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-12"
      >
        {[
          { icon: Key, title: "Get API Key", desc: "Create your API credentials in Settings" },
          { icon: Terminal, title: "Make Requests", desc: "Use our REST API or WebSocket" },
          { icon: Zap, title: "Go Live", desc: "Deploy your trading automation" },
        ].map((step, index) => (
          <GlassCard key={step.title} noPadding className="p-4 text-center">
            <div className="flex items-center justify-center gap-2 mb-2">
              <Badge variant="default" size="sm">{index + 1}</Badge>
              <GlowIcon icon={step.icon} size="sm" color={luxuryColors.cyan} />
            </div>
            <h3 className="font-semibold mb-1" style={{ color: luxuryColors.textPrimary }}>
              {step.title}
            </h3>
            <p className="text-xs" style={{ color: luxuryColors.textMuted }}>{step.desc}</p>
          </GlassCard>
        ))}
      </motion.div>

      {/* API Explorer */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.2 }}
        className="mb-12"
      >
        <h2 className="text-2xl font-bold mb-6" style={{ color: luxuryColors.textPrimary }}>
          API Endpoints
        </h2>
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Endpoint List */}
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
                style={{ borderColor: activeEndpoint === index ? luxuryColors.cyan : 'transparent' }}
              >
                <div className="flex items-center gap-2 mb-1">
                  <Badge
                    variant={endpoint.method === "GET" ? "success" : "warning"}
                    size="sm"
                  >
                    {endpoint.method}
                  </Badge>
                  <code className="text-xs" style={{ color: luxuryColors.textPrimary }}>
                    {endpoint.path.split('/').slice(-1)[0]}
                  </code>
                </div>
                <p className="text-xs" style={{ color: luxuryColors.textMuted }}>
                  {endpoint.description}
                </p>
              </button>
            ))}
          </div>

          {/* Code Preview */}
          <div className="lg:col-span-2">
            <GlassCard>
              <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-2">
                  <Code className="w-4 h-4" style={{ color: luxuryColors.cyan }} />
                  <span className="text-sm font-mono" style={{ color: luxuryColors.textPrimary }}>
                    {endpoints[activeEndpoint].path}
                  </span>
                </div>
                <button
                  onClick={() => copyToClipboard(endpoints[activeEndpoint].example)}
                  className="p-2 rounded hover:bg-white/10 transition-colors"
                >
                  <Copy className="w-4 h-4" style={{ color: luxuryColors.textMuted }} />
                </button>
              </div>
              <pre
                className="p-4 rounded-lg overflow-x-auto text-sm font-mono"
                style={{ backgroundColor: luxuryColors.bgPrimary, color: luxuryColors.cyan }}
              >
                {endpoints[activeEndpoint].example}
              </pre>
            </GlassCard>
          </div>
        </div>
      </motion.div>

      {/* SDKs */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.3 }}
        className="mb-12"
      >
        <h2 className="text-2xl font-bold mb-6" style={{ color: luxuryColors.textPrimary }}>
          Official SDKs
        </h2>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          {sdks.map((sdk) => (
            <GlassCard key={sdk.name} noPadding className="p-4 text-center hover:border-cyan-500/30 transition-all cursor-pointer">
              <span className="text-3xl mb-2 block">{sdk.icon}</span>
              <h4 className="font-semibold" style={{ color: luxuryColors.textPrimary }}>{sdk.name}</h4>
              <p className="text-xs" style={{ color: luxuryColors.textMuted }}>v{sdk.version}</p>
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
          <GlowIcon icon={Webhook} size="lg" color={luxuryColors.cyan} className="mx-auto mb-4" />
          <h2 className="text-2xl font-bold mb-4" style={{ color: luxuryColors.textPrimary }}>
            Ready to Build?
          </h2>
          <p className="mb-6" style={{ color: luxuryColors.textMuted }}>
            Check out our full documentation for detailed guides and examples.
          </p>
          <div className="flex justify-center gap-4">
            <Link to="/docs">
              <PremiumButton variant="primary">
                View Documentation
                <ExternalLink className="w-4 h-4" />
              </PremiumButton>
            </Link>
            <Link to="/settings">
              <PremiumButton variant="secondary">
                Get API Keys
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
