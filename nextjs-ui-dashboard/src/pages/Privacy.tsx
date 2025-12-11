import { motion } from "framer-motion";
import { Link } from "react-router-dom";
import { useTranslation } from "react-i18next";
import {
  Eye,
  Lock,
  Database,
  Users,
  ArrowLeft,
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

const sectionIcons = [Database, Eye, Lock, Users];
const sectionKeys = ["collection", "usage", "protection", "rights"];

const Privacy = () => {
  const colors = useThemeColors();
  const { t } = useTranslation('pages');

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
          {t('privacy.badge')}
        </Badge>
        <h1 className="text-4xl md:text-5xl font-black mb-4">
          <GradientText>{t('privacy.title')}</GradientText>
          <br />
          <span style={{ color: colors.textPrimary }}>{t('privacy.subtitle')}</span>
        </h1>
        <p className="text-lg max-w-2xl mx-auto mb-4" style={{ color: colors.textMuted }}>
          {t('privacy.description')}
        </p>
        <div className="flex items-center justify-center gap-2 text-sm" style={{ color: colors.textMuted }}>
          <FileText className="w-4 h-4" />
          <span>{t('privacy.lastUpdated')}</span>
        </div>
      </motion.div>

      {/* Detailed Sections */}
      <div className="space-y-6 mb-12">
        {sectionKeys.map((key, index) => {
          const Icon = sectionIcons[index];
          const items = t(`privacy.sections.${key}.items`, { returnObjects: true }) as string[];
          return (
            <motion.div
              key={key}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.2 + index * 0.1 }}
            >
              <GlassCard>
                <div className="flex items-start gap-4">
                  <GlowIcon icon={Icon} size="md" color={colors.cyan} />
                  <div className="flex-1">
                    <h3 className="text-lg font-bold mb-3" style={{ color: colors.textPrimary }}>
                      {t(`privacy.sections.${key}.title`)}
                    </h3>
                    <ul className="space-y-2">
                      {items.map((item, i) => (
                        <li key={i} className="flex items-start gap-2 text-sm" style={{ color: colors.textSecondary }}>
                          <span style={{ color: colors.cyan }}>•</span>
                          {item}
                        </li>
                      ))}
                    </ul>
                  </div>
                </div>
              </GlassCard>
            </motion.div>
          );
        })}
      </div>

      {/* Contact Section */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.8 }}
      >
        <GlassCard className="text-center">
          <p className="mb-6" style={{ color: colors.textMuted }}>
            {t('privacy.contact')}
          </p>
          <div className="flex justify-center gap-4">
            <Link to="/contact">
              <PremiumButton variant="primary">
                {t('common.contactUs')}
              </PremiumButton>
            </Link>
          </div>
        </GlassCard>
      </motion.div>
    </PageWrapper>
  );
};

export default Privacy;
