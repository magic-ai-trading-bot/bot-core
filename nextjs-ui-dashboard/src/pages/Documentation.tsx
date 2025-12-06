import { motion } from "framer-motion";
import { Link } from "react-router-dom";
import {
  Book,
  Code,
  Rocket,
  Settings,
  Shield,
  Zap,
  ArrowLeft,
  ChevronRight,
  FileText,
  Video,
  MessageCircle,
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

const sections = [
  {
    icon: Rocket,
    title: "Getting Started",
    description: "Quick start guide to set up your trading bot",
    articles: [
      "Creating Your Account",
      "Connecting Your Exchange",
      "Your First Paper Trade",
      "Understanding the Dashboard",
    ],
    color: luxuryColors.profit,
  },
  {
    icon: Settings,
    title: "Configuration",
    description: "Learn how to configure strategies and settings",
    articles: [
      "Strategy Configuration",
      "Risk Management Setup",
      "API Key Management",
      "Notification Settings",
    ],
    color: luxuryColors.cyan,
  },
  {
    icon: Code,
    title: "API Reference",
    description: "Complete API documentation for developers",
    articles: [
      "REST API Overview",
      "WebSocket Streams",
      "Authentication",
      "Rate Limits",
    ],
    color: luxuryColors.primary,
  },
  {
    icon: Shield,
    title: "Security",
    description: "Best practices for securing your account",
    articles: [
      "Two-Factor Authentication",
      "API Key Security",
      "Withdrawal Whitelists",
      "Security Checklist",
    ],
    color: luxuryColors.warning,
  },
];

const guides = [
  {
    icon: FileText,
    title: "Complete Trading Guide",
    description: "Master all trading features in Bot Core",
    readTime: "15 min read",
  },
  {
    icon: Video,
    title: "Video Tutorials",
    description: "Step-by-step video walkthroughs",
    readTime: "10 videos",
  },
  {
    icon: Zap,
    title: "Strategy Optimization",
    description: "Learn to optimize your trading strategies",
    readTime: "20 min read",
  },
];

const Documentation = () => {
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
          Documentation
        </Badge>
        <h1 className="text-4xl md:text-5xl font-black mb-4">
          <GradientText>Learn Bot Core</GradientText>
        </h1>
        <p className="text-lg max-w-2xl mx-auto" style={{ color: luxuryColors.textMuted }}>
          Everything you need to master automated cryptocurrency trading.
        </p>
      </motion.div>

      {/* Search Bar */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.1 }}
        className="max-w-2xl mx-auto mb-12"
      >
        <div
          className="flex items-center gap-3 p-4 rounded-xl border"
          style={{
            backgroundColor: luxuryColors.bgSecondary,
            borderColor: luxuryColors.borderSubtle,
          }}
        >
          <Book className="w-5 h-5" style={{ color: luxuryColors.textMuted }} />
          <input
            type="text"
            placeholder="Search documentation..."
            className="flex-1 bg-transparent outline-none text-sm"
            style={{ color: luxuryColors.textPrimary }}
          />
          <Badge variant="default" size="sm">⌘K</Badge>
        </div>
      </motion.div>

      {/* Documentation Sections */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-12">
        {sections.map((section, index) => (
          <motion.div
            key={section.title}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.2 + index * 0.1 }}
          >
            <GlassCard className="h-full hover:border-cyan-500/30 transition-all duration-300">
              <div className="flex items-start gap-4 mb-4">
                <GlowIcon icon={section.icon} size="md" color={section.color} />
                <div>
                  <h3 className="text-lg font-bold" style={{ color: luxuryColors.textPrimary }}>
                    {section.title}
                  </h3>
                  <p className="text-sm" style={{ color: luxuryColors.textMuted }}>
                    {section.description}
                  </p>
                </div>
              </div>
              <ul className="space-y-2">
                {section.articles.map((article) => (
                  <li key={article}>
                    <button
                      className="w-full flex items-center justify-between p-2 rounded-lg hover:bg-white/5 transition-colors group"
                    >
                      <span className="text-sm" style={{ color: luxuryColors.textSecondary }}>
                        {article}
                      </span>
                      <ChevronRight
                        className="w-4 h-4 opacity-0 group-hover:opacity-100 transition-opacity"
                        style={{ color: luxuryColors.cyan }}
                      />
                    </button>
                  </li>
                ))}
              </ul>
            </GlassCard>
          </motion.div>
        ))}
      </div>

      {/* Featured Guides */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.5 }}
        className="mb-12"
      >
        <h2 className="text-2xl font-bold mb-6" style={{ color: luxuryColors.textPrimary }}>
          Featured Guides
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {guides.map((guide) => (
            <GlassCard
              key={guide.title}
              noPadding
              className="p-4 hover:border-cyan-500/30 transition-all cursor-pointer"
            >
              <GlowIcon icon={guide.icon} size="sm" color={luxuryColors.cyan} className="mb-3" />
              <h4 className="font-semibold mb-1" style={{ color: luxuryColors.textPrimary }}>
                {guide.title}
              </h4>
              <p className="text-xs mb-2" style={{ color: luxuryColors.textMuted }}>
                {guide.description}
              </p>
              <Badge variant="default" size="sm">{guide.readTime}</Badge>
            </GlassCard>
          ))}
        </div>
      </motion.div>

      {/* Help Section */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.6 }}
      >
        <GlassCard className="text-center">
          <GlowIcon icon={MessageCircle} size="lg" color={luxuryColors.cyan} className="mx-auto mb-4" />
          <h2 className="text-2xl font-bold mb-4" style={{ color: luxuryColors.textPrimary }}>
            Need More Help?
          </h2>
          <p className="mb-6" style={{ color: luxuryColors.textMuted }}>
            Our support team is here to help you succeed.
          </p>
          <div className="flex justify-center gap-4">
            <Link to="/contact">
              <PremiumButton variant="primary">
                Contact Support
              </PremiumButton>
            </Link>
            <a href="https://discord.gg/botcore" target="_blank" rel="noopener noreferrer">
              <PremiumButton variant="secondary">
                Join Discord
              </PremiumButton>
            </a>
          </div>
        </GlassCard>
      </motion.div>
    </PageWrapper>
  );
};

export default Documentation;
