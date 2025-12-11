import { motion } from "framer-motion";
import { Link } from "react-router-dom";
import { useTranslation } from "react-i18next";
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
  GlassCard,
  GradientText,
  PremiumButton,
  Badge,
  GlowIcon,
  PageWrapper,
} from "@/styles/luxury-design-system";
import { useThemeColors } from "@/hooks/useThemeColors";

const Features = () => {
  const colors = useThemeColors();
  const { t } = useTranslation('pages');

  const additionalFeatures = [
    { icon: TrendingUp, key: "backtesting" },
    { icon: Lock, key: "security" },
    { icon: Globe, key: "multiExchange" },
    { icon: Clock, key: "monitoring" },
    { icon: Target, key: "alerts" },
    { icon: Sparkles, key: "aiSignals" },
  ];

  const features = [
    {
      icon: Brain,
      key: "aiAnalysis",
      color: colors.cyan,
    },
    {
      icon: Bot,
      key: "tradingBot",
      color: colors.profit,
    },
    {
      icon: Shield,
      key: "riskManagement",
      color: colors.warning,
    },
    {
      icon: LineChart,
      key: "analytics",
      color: colors.primary,
    },
    {
      icon: Zap,
      key: "execution",
      color: colors.accent,
    },
    {
      icon: BarChart3,
      key: "strategies",
      color: colors.purple,
    },
  ];
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
          {t('features.badge')}
        </Badge>
        <h1 className="text-4xl md:text-5xl font-black mb-4">
          <GradientText>{t('features.title')}</GradientText>
          <br />
          <span style={{ color: colors.textPrimary }}>{t('features.subtitle')}</span>
        </h1>
        <p className="text-lg max-w-2xl mx-auto" style={{ color: colors.textMuted }}>
          {t('features.description')}
        </p>
      </motion.div>

      {/* Main Features Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mb-16">
        {features.map((feature, index) => (
          <motion.div
            key={feature.key}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: index * 0.1 }}
          >
            <GlassCard className="h-full hover:border-cyan-500/30 transition-all duration-300">
              <GlowIcon icon={feature.icon} size="lg" color={feature.color} className="mb-4" />
              <h3 className="text-xl font-bold mb-2" style={{ color: colors.textPrimary }}>
                {t(`features.items.${feature.key}.title`)}
              </h3>
              <p className="text-sm mb-4" style={{ color: colors.textMuted }}>
                {t(`features.items.${feature.key}.description`)}
              </p>
              <div className="flex flex-wrap gap-2">
                {(t(`features.items.${feature.key}.highlights`, { returnObjects: true }) as string[]).map((highlight) => (
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
        <h2 className="text-2xl font-bold text-center mb-8" style={{ color: colors.textPrimary }}>
          {t('features.andMore')}
        </h2>
        <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4">
          {additionalFeatures.map((feature, index) => (
            <motion.div
              key={feature.key}
              initial={{ opacity: 0, scale: 0.9 }}
              animate={{ opacity: 1, scale: 1 }}
              transition={{ delay: 0.7 + index * 0.05 }}
              className="text-center p-4 rounded-xl"
              style={{ backgroundColor: colors.bgSecondary }}
            >
              <GlowIcon icon={feature.icon} size="md" color={colors.cyan} className="mx-auto mb-2" />
              <h4 className="text-sm font-semibold mb-1" style={{ color: colors.textPrimary }}>
                {t(`features.additional.${feature.key}.title`)}
              </h4>
              <p className="text-xs" style={{ color: colors.textMuted }}>
                {t(`features.additional.${feature.key}.description`)}
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
          <h2 className="text-2xl font-bold mb-4" style={{ color: colors.textPrimary }}>
            {t('features.readyTitle')}
          </h2>
          <p className="mb-6" style={{ color: colors.textMuted }}>
            {t('features.readyDescription')}
          </p>
          <div className="flex justify-center gap-4">
            <Link to="/register">
              <PremiumButton variant="primary" size="lg">
                {t('features.getStartedFree')}
                <ChevronRight className="w-5 h-5" />
              </PremiumButton>
            </Link>
            <Link to="/pricing">
              <PremiumButton variant="secondary" size="lg">
                {t('common.viewPricing')}
              </PremiumButton>
            </Link>
          </div>
        </GlassCard>
      </motion.div>
    </PageWrapper>
  );
};

export default Features;
