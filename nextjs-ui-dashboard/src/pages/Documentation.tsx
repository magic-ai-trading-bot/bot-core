import { motion } from "framer-motion";
import { Link } from "react-router-dom";
import { useTranslation } from "react-i18next";
import {
  Book,
  Code,
  Rocket,
  Settings,
  Shield,
  ArrowLeft,
  ChevronRight,
  MessageCircle,
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

const sectionIcons = [Rocket, Settings, Code, Shield];
const sectionKeys = ["gettingStarted", "trading", "api", "strategies"];

const Documentation = () => {
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
        className="text-center mb-16"
      >
        <Badge variant="info" className="mb-4">
          {t('docs.badge')}
        </Badge>
        <h1 className="text-4xl md:text-5xl font-black mb-4">
          <GradientText>{t('docs.title')}</GradientText>
          <br />
          <span style={{ color: colors.textPrimary }}>{t('docs.subtitle')}</span>
        </h1>
        <p className="text-lg max-w-2xl mx-auto" style={{ color: colors.textMuted }}>
          {t('docs.description')}
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
            backgroundColor: colors.bgSecondary,
            borderColor: colors.borderSubtle,
          }}
        >
          <Book className="w-5 h-5" style={{ color: colors.textMuted }} />
          <input
            type="text"
            placeholder="Search documentation..."
            className="flex-1 bg-transparent outline-none text-sm"
            style={{ color: colors.textPrimary }}
          />
          <Badge variant="default" size="sm">⌘K</Badge>
        </div>
      </motion.div>

      {/* Documentation Sections */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-12">
        {sectionKeys.map((key, index) => {
          const Icon = sectionIcons[index];
          const sectionColors = [colors.profit, colors.cyan, colors.primary, colors.warning];
          return (
            <motion.div
              key={key}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.2 + index * 0.1 }}
            >
              <GlassCard className="h-full hover:border-cyan-500/30 transition-all duration-300">
                <div className="flex items-start gap-4 mb-4">
                  <GlowIcon icon={Icon} size="md" color={sectionColors[index]} />
                  <div>
                    <h3 className="text-lg font-bold" style={{ color: colors.textPrimary }}>
                      {t(`docs.sections.${key}.title`)}
                    </h3>
                    <p className="text-sm" style={{ color: colors.textMuted }}>
                      {t(`docs.sections.${key}.description`)}
                    </p>
                  </div>
                </div>
                <div className="flex justify-end">
                  <button
                    className="flex items-center gap-1 text-sm hover:opacity-80 transition-opacity"
                    style={{ color: colors.cyan }}
                  >
                    {t('common.learnMore')}
                    <ChevronRight className="w-4 h-4" />
                  </button>
                </div>
              </GlassCard>
            </motion.div>
          );
        })}
      </div>

      {/* Help Section */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.6 }}
      >
        <GlassCard className="text-center">
          <GlowIcon icon={MessageCircle} size="lg" color={colors.cyan} className="mx-auto mb-4" />
          <h2 className="text-2xl font-bold mb-4" style={{ color: colors.textPrimary }}>
            Need More Help?
          </h2>
          <p className="mb-6" style={{ color: colors.textMuted }}>
            Our support team is here to help you succeed.
          </p>
          <div className="flex justify-center gap-4">
            <Link to="/contact">
              <PremiumButton variant="primary">
                {t('common.contactUs')}
              </PremiumButton>
            </Link>
            <PremiumButton variant="secondary">
              {t('docs.viewAll')}
            </PremiumButton>
          </div>
        </GlassCard>
      </motion.div>
    </PageWrapper>
  );
};

export default Documentation;
