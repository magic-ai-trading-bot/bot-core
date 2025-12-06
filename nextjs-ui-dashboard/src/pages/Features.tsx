import { motion } from "framer-motion";
import { Link } from "react-router-dom";
import {
  Bot,
  Brain,
  LineChart,
  Shield,
  Zap,
  BarChart3,
  TrendingUp,
  Lock,
  Globe,
  Clock,
  Target,
  Sparkles,
  ChevronRight,
  ArrowLeft,
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

const features = [
  {
    icon: Brain,
    title: "AI-Powered Analysis",
    description: "Advanced machine learning models analyze market patterns, sentiment, and technical indicators in real-time to identify profitable trading opportunities.",
    highlights: ["LSTM & Transformer Models", "GPT-4 Market Analysis", "Sentiment Detection"],
    color: luxuryColors.cyan,
  },
  {
    icon: Bot,
    title: "Automated Trading Bot",
    description: "Set your strategy once and let our bot execute trades 24/7 with precision timing and emotion-free decision making.",
    highlights: ["24/7 Trading", "Multiple Strategies", "Paper & Live Trading"],
    color: luxuryColors.profit,
  },
  {
    icon: Shield,
    title: "Advanced Risk Management",
    description: "Protect your capital with intelligent stop-loss, take-profit, and position sizing algorithms that adapt to market conditions.",
    highlights: ["Dynamic Stop Loss", "Position Sizing", "Drawdown Protection"],
    color: luxuryColors.warning,
  },
  {
    icon: LineChart,
    title: "Real-Time Analytics",
    description: "Monitor your portfolio performance with comprehensive dashboards, real-time charts, and detailed trade analytics.",
    highlights: ["Live P&L Tracking", "Performance Metrics", "Trade History"],
    color: luxuryColors.primary,
  },
  {
    icon: Zap,
    title: "Lightning Fast Execution",
    description: "Execute trades in milliseconds with our optimized infrastructure directly connected to major exchanges.",
    highlights: ["<50ms Latency", "Smart Order Routing", "Slippage Protection"],
    color: luxuryColors.accent,
  },
  {
    icon: BarChart3,
    title: "Multiple Strategies",
    description: "Choose from proven trading strategies including RSI, MACD, Bollinger Bands, and Volume-based approaches.",
    highlights: ["RSI Strategy", "MACD Crossover", "Bollinger Bands"],
    color: luxuryColors.purple,
  },
];

const additionalFeatures = [
  { icon: TrendingUp, title: "Backtesting Engine", description: "Test strategies on historical data" },
  { icon: Lock, title: "Bank-Grade Security", description: "End-to-end encryption & 2FA" },
  { icon: Globe, title: "Multi-Exchange Support", description: "Trade on Binance, Bybit & more" },
  { icon: Clock, title: "24/7 Monitoring", description: "Always-on system monitoring" },
  { icon: Target, title: "Custom Alerts", description: "Telegram, Discord & Email alerts" },
  { icon: Sparkles, title: "AI Signals", description: "Machine learning predictions" },
];

const Features = () => {
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
          Platform Features
        </Badge>
        <h1 className="text-4xl md:text-5xl font-black mb-4">
          <GradientText>Powerful Features for</GradientText>
          <br />
          <span style={{ color: luxuryColors.textPrimary }}>Modern Traders</span>
        </h1>
        <p className="text-lg max-w-2xl mx-auto" style={{ color: luxuryColors.textMuted }}>
          Everything you need to trade smarter, faster, and more profitably with cutting-edge AI technology.
        </p>
      </motion.div>

      {/* Main Features Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mb-16">
        {features.map((feature, index) => (
          <motion.div
            key={feature.title}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: index * 0.1 }}
          >
            <GlassCard className="h-full hover:border-cyan-500/30 transition-all duration-300">
              <GlowIcon icon={feature.icon} size="lg" color={feature.color} className="mb-4" />
              <h3 className="text-xl font-bold mb-2" style={{ color: luxuryColors.textPrimary }}>
                {feature.title}
              </h3>
              <p className="text-sm mb-4" style={{ color: luxuryColors.textMuted }}>
                {feature.description}
              </p>
              <div className="flex flex-wrap gap-2">
                {feature.highlights.map((highlight) => (
                  <Badge key={highlight} variant="default" size="sm">
                    {highlight}
                  </Badge>
                ))}
              </div>
            </GlassCard>
          </motion.div>
        ))}
      </div>

      {/* Additional Features */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.6 }}
        className="mb-16"
      >
        <h2 className="text-2xl font-bold text-center mb-8" style={{ color: luxuryColors.textPrimary }}>
          And Much More...
        </h2>
        <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4">
          {additionalFeatures.map((feature, index) => (
            <motion.div
              key={feature.title}
              initial={{ opacity: 0, scale: 0.9 }}
              animate={{ opacity: 1, scale: 1 }}
              transition={{ delay: 0.7 + index * 0.05 }}
              className="text-center p-4 rounded-xl"
              style={{ backgroundColor: luxuryColors.bgSecondary }}
            >
              <GlowIcon icon={feature.icon} size="md" color={luxuryColors.cyan} className="mx-auto mb-2" />
              <h4 className="text-sm font-semibold mb-1" style={{ color: luxuryColors.textPrimary }}>
                {feature.title}
              </h4>
              <p className="text-xs" style={{ color: luxuryColors.textMuted }}>
                {feature.description}
              </p>
            </motion.div>
          ))}
        </div>
      </motion.div>

      {/* CTA Section */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.8 }}
      >
        <GlassCard className="text-center">
          <h2 className="text-2xl font-bold mb-4" style={{ color: luxuryColors.textPrimary }}>
            Ready to Start Trading?
          </h2>
          <p className="mb-6" style={{ color: luxuryColors.textMuted }}>
            Join thousands of traders using Bot Core to automate their trading strategies.
          </p>
          <div className="flex justify-center gap-4">
            <Link to="/register">
              <PremiumButton variant="primary" size="lg">
                Get Started Free
                <ChevronRight className="w-5 h-5" />
              </PremiumButton>
            </Link>
            <Link to="/pricing">
              <PremiumButton variant="secondary" size="lg">
                View Pricing
              </PremiumButton>
            </Link>
          </div>
        </GlassCard>
      </motion.div>
    </PageWrapper>
  );
};

export default Features;
