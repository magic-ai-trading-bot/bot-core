import { motion } from "framer-motion";
import { Link } from "react-router-dom";
import { useTranslation } from "react-i18next";
import {
  Check,
  Zap,
  Crown,
  Building2,
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

const Pricing = () => {
  const colors = useThemeColors();
  const { t } = useTranslation('pages');

  const plans = [
    {
      key: "starter",
      icon: Zap,
      variant: "secondary" as const,
      popular: false,
    },
    {
      key: "pro",
      icon: Crown,
      variant: "primary" as const,
      popular: true,
    },
    {
      key: "enterprise",
      icon: Building2,
      variant: "secondary" as const,
      popular: false,
    },
  ];

  const faqKeys = ["switch", "trial", "payment", "cancel"];

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
          {t('pricing.badge')}
        </Badge>
        <h1 className="text-4xl md:text-5xl font-black mb-4">
          <GradientText>{t('pricing.title')}</GradientText>
          <br />
          <span style={{ color: colors.textPrimary }}>{t('pricing.subtitle')}</span>
        </h1>
        <p className="text-lg max-w-2xl mx-auto" style={{ color: colors.textMuted }}>
          {t('pricing.description')}
        </p>
      </motion.div>

      {/* Pricing Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-16 max-w-5xl mx-auto">
        {plans.map((plan, index) => (
          <motion.div
            key={plan.key}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: index * 0.1 }}
            className="relative"
          >
            {plan.popular && (
              <div className="absolute -top-3 left-1/2 -translate-x-1/2 z-10">
                <Badge variant="warning" glow>{t('pricing.mostPopular')}</Badge>
              </div>
            )}
            <GlassCard
              className={`h-full ${plan.popular ? 'border-cyan-500/50 ring-2 ring-cyan-500/20' : ''}`}
            >
              <div className="text-center mb-6">
                <GlowIcon
                  icon={plan.icon}
                  size="lg"
                  color={plan.popular ? colors.cyan : colors.textMuted}
                  className="mx-auto mb-4"
                />
                <h3 className="text-xl font-bold mb-2" style={{ color: colors.textPrimary }}>
                  {t(`pricing.plans.${plan.key}.name`)}
                </h3>
                <div className="mb-2">
                  <span className="text-4xl font-black" style={{ color: colors.cyan }}>
                    {t(`pricing.plans.${plan.key}.price`)}
                  </span>
                  {t(`pricing.plans.${plan.key}.period`, { defaultValue: '' }) && (
                    <span style={{ color: colors.textMuted }}>
                      {t(`pricing.plans.${plan.key}.period`)}
                    </span>
                  )}
                </div>
                <p className="text-sm" style={{ color: colors.textMuted }}>
                  {t(`pricing.plans.${plan.key}.description`)}
                </p>
              </div>

              <ul className="space-y-3 mb-6">
                {(t(`pricing.plans.${plan.key}.features`, { returnObjects: true }) as string[]).map((feature) => (
                  <li key={feature} className="flex items-center gap-2">
                    <Check className="w-4 h-4" style={{ color: colors.profit }} />
                    <span className="text-sm" style={{ color: colors.textSecondary }}>
                      {feature}
                    </span>
                  </li>
                ))}
              </ul>

              <Link to={plan.key === "enterprise" ? "/contact" : "/register"}>
                <PremiumButton variant={plan.variant} fullWidth>
                  {t(`pricing.plans.${plan.key}.cta`)}
                  <ChevronRight className="w-4 h-4" />
                </PremiumButton>
              </Link>
            </GlassCard>
          </motion.div>
        ))}
      </div>

      {/* FAQ Section */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.4 }}
        className="max-w-3xl mx-auto"
      >
        <h2 className="text-2xl font-bold text-center mb-8" style={{ color: colors.textPrimary }}>
          {t('pricing.faq')}
        </h2>
        <div className="space-y-4">
          {faqKeys.map((faqKey, index) => (
            <GlassCard key={index} noPadding className="p-4">
              <h4 className="font-semibold mb-2" style={{ color: colors.textPrimary }}>
                {t(`pricing.faqs.${faqKey}.question`)}
              </h4>
              <p className="text-sm" style={{ color: colors.textMuted }}>
                {t(`pricing.faqs.${faqKey}.answer`)}
              </p>
            </GlassCard>
          ))}
        </div>
      </motion.div>
    </PageWrapper>
  );
};

export default Pricing;
