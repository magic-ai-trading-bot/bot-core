import { motion } from "framer-motion";
import { Link } from "react-router-dom";
import { useTranslation } from "react-i18next";
import {
  Scale,
  Shield,
  FileCheck,
  Building,
  CheckCircle,
  ArrowLeft,
  ExternalLink,
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

const certIcons = [Shield, FileCheck, Building, Scale];

const Compliance = () => {
  const colors = useThemeColors();
  const { t } = useTranslation('pages');

  const certifications = t('compliance.certifications.items', { returnObjects: true }) as Array<{
    name: string;
    description: string;
  }>;

  const policies = t('compliance.policies.items', { returnObjects: true }) as Array<{
    title: string;
    description: string;
  }>;

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
        <Badge variant="info" className="mb-4">
          {t('compliance.badge')}
        </Badge>
        <h1 className="text-4xl md:text-5xl font-black mb-4">
          <GradientText>{t('compliance.title')}</GradientText>
          <br />
          <span style={{ color: colors.textPrimary }}>{t('compliance.subtitle')}</span>
        </h1>
        <p className="text-lg max-w-2xl mx-auto" style={{ color: colors.textMuted }}>
          {t('compliance.description')}
        </p>
      </motion.div>

      {/* Certifications */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.2 }}
        className="mb-12"
      >
        <h2 className="text-2xl font-bold text-center mb-8" style={{ color: colors.textPrimary }}>
          {t('compliance.certifications.title')}
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          {certifications.map((cert, index) => {
            const Icon = certIcons[index % certIcons.length];
            return (
              <motion.div
                key={cert.name}
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.3 + index * 0.1 }}
              >
                <GlassCard className="h-full text-center">
                  <GlowIcon icon={Icon} size="lg" color={colors.cyan} className="mx-auto mb-4" />
                  <h3 className="font-bold mb-2" style={{ color: colors.textPrimary }}>
                    {cert.name}
                  </h3>
                  <p className="text-sm" style={{ color: colors.textMuted }}>
                    {cert.description}
                  </p>
                </GlassCard>
              </motion.div>
            );
          })}
        </div>
      </motion.div>

      {/* Policies */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.4 }}
        className="mb-12"
      >
        <h2 className="text-2xl font-bold text-center mb-8" style={{ color: colors.textPrimary }}>
          {t('compliance.policies.title')}
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {policies.map((policy, index) => (
            <motion.div
              key={policy.title}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.5 + index * 0.1 }}
            >
              <GlassCard className="h-full">
                <div className="flex items-start gap-3">
                  <CheckCircle className="w-5 h-5 flex-shrink-0" style={{ color: colors.profit }} />
                  <div>
                    <h3 className="font-bold mb-2" style={{ color: colors.textPrimary }}>
                      {policy.title}
                    </h3>
                    <p className="text-sm" style={{ color: colors.textMuted }}>
                      {policy.description}
                    </p>
                  </div>
                </div>
              </GlassCard>
            </motion.div>
          ))}
        </div>
      </motion.div>

      {/* Updates & Contact */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.6 }}
      >
        <GlassCard className="text-center">
          <h2 className="text-xl font-bold mb-4" style={{ color: colors.textPrimary }}>
            {t('compliance.updates.title')}
          </h2>
          <p className="mb-6" style={{ color: colors.textMuted }}>
            {t('compliance.updates.description')}
          </p>
          <p className="mb-6 text-sm" style={{ color: colors.textMuted }}>
            {t('compliance.contact')}
          </p>
          <div className="flex justify-center gap-4">
            <Link to="/contact">
              <PremiumButton variant="primary">
                {t('common.contactUs')}
                <ExternalLink className="w-4 h-4" />
              </PremiumButton>
            </Link>
          </div>
        </GlassCard>
      </motion.div>
    </PageWrapper>
  );
};

export default Compliance;
