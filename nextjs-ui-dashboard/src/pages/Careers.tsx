import { motion } from "framer-motion";
import { Link } from "react-router-dom";
import { useTranslation } from "react-i18next";
import {
  Briefcase,
  MapPin,
  DollarSign,
  Clock,
  Heart,
  Sparkles,
  Coffee,
  GraduationCap,
  Plane,
  ArrowLeft,
  ChevronRight,
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

const benefitIcons = [DollarSign, Heart, Plane, Coffee, GraduationCap, Sparkles];

const Careers = () => {
  const colors = useThemeColors();
  const { t } = useTranslation('pages');

  const benefits = t('careers.whyJoin.items', { returnObjects: true }) as Array<{
    title: string;
    description: string;
  }>;

  const positions = t('careers.openings.positions', { returnObjects: true }) as Array<{
    title: string;
    location: string;
    type: string;
    department: string;
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
        className="text-center mb-16"
      >
        <Badge variant="success" className="mb-4">
          {t('careers.badge')}
        </Badge>
        <h1 className="text-4xl md:text-5xl font-black mb-4">
          <GradientText>{t('careers.title')}</GradientText>
          <br />
          <span style={{ color: colors.textPrimary }}>{t('careers.subtitle')}</span>
        </h1>
        <p className="text-lg max-w-2xl mx-auto" style={{ color: colors.textMuted }}>
          {t('careers.description')}
        </p>
      </motion.div>

      {/* Benefits */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.2 }}
        className="mb-16"
      >
        <h2 className="text-2xl font-bold text-center mb-8" style={{ color: colors.textPrimary }}>
          {t('careers.whyJoin.title')}
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {benefits.map((benefit, index) => {
            const Icon = benefitIcons[index % benefitIcons.length];
            return (
              <motion.div
                key={benefit.title}
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.3 + index * 0.05 }}
              >
                <GlassCard noPadding className="p-4 h-full">
                  <div className="flex items-start gap-3">
                    <GlowIcon icon={Icon} size="sm" color={colors.cyan} />
                    <div>
                      <h3 className="font-semibold mb-1" style={{ color: colors.textPrimary }}>
                        {benefit.title}
                      </h3>
                      <p className="text-sm" style={{ color: colors.textMuted }}>
                        {benefit.description}
                      </p>
                    </div>
                  </div>
                </GlassCard>
              </motion.div>
            );
          })}
        </div>
      </motion.div>

      {/* Open Positions */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.4 }}
        className="mb-16"
      >
        <h2 className="text-2xl font-bold text-center mb-8" style={{ color: colors.textPrimary }}>
          {t('careers.openings.title')}
        </h2>
        <div className="space-y-4">
          {positions.map((position, index) => (
            <motion.div
              key={position.title}
              initial={{ opacity: 0, x: -20 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ delay: 0.5 + index * 0.1 }}
            >
              <GlassCard className="hover:border-cyan-500/30 transition-all cursor-pointer">
                <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-4">
                  <div className="flex-1">
                    <div className="flex items-center gap-3 mb-2">
                      <GlowIcon icon={Briefcase} size="sm" color={colors.cyan} />
                      <h3 className="font-bold text-lg" style={{ color: colors.textPrimary }}>
                        {position.title}
                      </h3>
                    </div>
                    <div className="flex flex-wrap gap-2">
                      <Badge variant="default" size="sm">
                        <MapPin className="w-3 h-3 mr-1" />
                        {position.location}
                      </Badge>
                      <Badge variant="default" size="sm">
                        <Clock className="w-3 h-3 mr-1" />
                        {position.type}
                      </Badge>
                      <Badge variant="info" size="sm">
                        {position.department}
                      </Badge>
                    </div>
                  </div>
                  <PremiumButton variant="primary" size="sm">
                    {t('careers.openings.apply')}
                    <ChevronRight className="w-4 h-4" />
                  </PremiumButton>
                </div>
              </GlassCard>
            </motion.div>
          ))}
        </div>
      </motion.div>

      {/* CTA */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.7 }}
      >
        <GlassCard className="text-center">
          <h2 className="text-2xl font-bold mb-4" style={{ color: colors.textPrimary }}>
            {t('careers.noPosition.title')}
          </h2>
          <p className="mb-6" style={{ color: colors.textMuted }}>
            {t('careers.noPosition.description')}
          </p>
          <Link to="/contact">
            <PremiumButton variant="primary">
              {t('careers.noPosition.button')}
            </PremiumButton>
          </Link>
        </GlassCard>
      </motion.div>
    </PageWrapper>
  );
};

export default Careers;
