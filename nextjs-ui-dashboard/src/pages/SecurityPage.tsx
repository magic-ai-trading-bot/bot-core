import { motion } from "framer-motion";
import { Link } from "react-router-dom";
import { useTranslation } from "react-i18next";
import {
  Shield,
  Lock,
  Key,
  Server,
  Eye,
  AlertTriangle,
  CheckCircle,
  ArrowLeft,
  Bug,
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

const featureIcons = [Lock, Key, Shield, Server, Eye, AlertTriangle];
const featureKeys = ["encryption", "mfa", "keys", "audit", "soc2", "monitoring"];

const SecurityPage = () => {
  const colors = useThemeColors();
  const { t } = useTranslation('pages');

  const practices = t('security.practices.items', { returnObjects: true }) as string[];

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
        className="text-center mb-12"
      >
        <Badge variant="success" className="mb-4">
          {t('security.badge')}
        </Badge>
        <h1 className="text-4xl md:text-5xl font-black mb-4">
          <GradientText>{t('security.title')}</GradientText>
          <br />
          <span style={{ color: colors.textPrimary }}>{t('security.subtitle')}</span>
        </h1>
        <p className="text-lg max-w-2xl mx-auto" style={{ color: colors.textMuted }}>
          {t('security.description')}
        </p>
      </motion.div>

      {/* Security Features */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.2 }}
        className="mb-12"
      >
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {featureKeys.map((key, index) => {
            const Icon = featureIcons[index];
            return (
              <motion.div
                key={key}
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.3 + index * 0.05 }}
              >
                <GlassCard noPadding className="p-4 h-full">
                  <div className="flex items-start justify-between mb-3">
                    <GlowIcon icon={Icon} size="md" color={colors.cyan} />
                    <Badge variant="success" size="sm">Active</Badge>
                  </div>
                  <h3 className="font-semibold mb-1" style={{ color: colors.textPrimary }}>
                    {t(`security.features.${key}.title`)}
                  </h3>
                  <p className="text-sm" style={{ color: colors.textMuted }}>
                    {t(`security.features.${key}.description`)}
                  </p>
                </GlassCard>
              </motion.div>
            );
          })}
        </div>
      </motion.div>

      {/* Security Practices */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.4 }}
        className="mb-12"
      >
        <GlassCard>
          <h2 className="text-xl font-bold mb-4" style={{ color: colors.textPrimary }}>
            {t('security.practices.title')}
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            {practices.map((practice, index) => (
              <div key={index} className="flex items-start gap-2">
                <CheckCircle className="w-4 h-4 mt-0.5 flex-shrink-0" style={{ color: colors.profit }} />
                <span className="text-sm" style={{ color: colors.textSecondary }}>{practice}</span>
              </div>
            ))}
          </div>
        </GlassCard>
      </motion.div>

      {/* Report Vulnerability */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.5 }}
      >
        <GlassCard className="text-center">
          <GlowIcon icon={Bug} size="lg" color={colors.warning} className="mx-auto mb-4" />
          <h2 className="text-xl font-bold mb-4" style={{ color: colors.textPrimary }}>
            {t('security.report.title')}
          </h2>
          <p className="mb-6" style={{ color: colors.textMuted }}>
            {t('security.report.description')}
          </p>
          <div className="flex justify-center gap-4">
            <a href="mailto:security@botcore.io">
              <PremiumButton variant="primary">
                {t('security.report.button')}
              </PremiumButton>
            </a>
          </div>
        </GlassCard>
      </motion.div>
    </PageWrapper>
  );
};

export default SecurityPage;
