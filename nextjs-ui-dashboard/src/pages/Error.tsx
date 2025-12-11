import { AlertTriangle, Home, RefreshCw, Zap } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { motion } from "framer-motion";
import {
  GlassCard,
  GradientText,
  PremiumButton,
  PageWrapper,
  containerVariants,
  itemVariants,
} from "@/styles/luxury-design-system";
import { useThemeColors } from "@/hooks/useThemeColors";

const Error = () => {
  const { t } = useTranslation('errors');
  const colors = useThemeColors();
  const navigate = useNavigate();

  const handleRefresh = () => {
    window.location.reload();
  };

  const handleGoHome = () => {
    navigate("/");
  };

  return (
    <PageWrapper className="min-h-screen flex items-center justify-center relative overflow-hidden">
      {/* Decorative Background Orbs */}
      <motion.div
        className="absolute top-1/4 left-1/4 w-64 h-64 rounded-full blur-3xl opacity-20"
        style={{ background: colors.gradientLoss }}
        animate={{
          scale: [1, 1.2, 1],
          opacity: [0.2, 0.3, 0.2],
        }}
        transition={{
          duration: 4,
          repeat: Infinity,
          ease: "easeInOut",
        }}
      />
      <motion.div
        className="absolute bottom-1/4 right-1/4 w-64 h-64 rounded-full blur-3xl opacity-20"
        style={{ background: colors.gradientPurple }}
        animate={{
          scale: [1, 1.2, 1],
          opacity: [0.2, 0.3, 0.2],
        }}
        transition={{
          duration: 4,
          repeat: Infinity,
          ease: "easeInOut",
          delay: 2,
        }}
      />

      {/* Main Content */}
      <motion.div
        className="flex max-w-lg flex-col items-center gap-8 text-center relative z-10 px-4"
        variants={containerVariants}
      >
        {/* Error Icon with Glow */}
        <motion.div
          variants={itemVariants}
          className="relative"
        >
          <motion.div
            className="absolute inset-0 rounded-full blur-2xl"
            style={{ background: colors.gradientLoss, opacity: 0.3 }}
            animate={{
              scale: [1, 1.1, 1],
            }}
            transition={{
              duration: 2,
              repeat: Infinity,
              ease: "easeInOut",
            }}
          />
          <GlassCard
            noPadding
            className="p-8 relative"
            glowColor="0 8px 32px rgba(239, 68, 68, 0.3)"
          >
            <AlertTriangle
              className="h-24 w-24"
              style={{ color: colors.loss }}
            />
          </GlassCard>
        </motion.div>

        {/* Error Message Card */}
        <GlassCard className="w-full">
          <motion.div variants={itemVariants} className="space-y-4">
            {/* Error Code */}
            <div>
              <GradientText
                className="text-6xl font-black tracking-tight"
                gradient={colors.gradientLoss}
              >
                500
              </GradientText>
            </div>

            {/* Error Title */}
            <div>
              <h2
                className="text-2xl font-bold mb-2"
                style={{ color: colors.textPrimary }}
              >
                {t('errorPage.title')}
              </h2>
              <p
                className="text-sm leading-relaxed"
                style={{ color: colors.textSecondary }}
              >
                {t('errorPage.description')}
              </p>
            </div>

            {/* Action Buttons */}
            <div className="flex flex-col sm:flex-row gap-3 pt-4">
              <PremiumButton
                onClick={handleRefresh}
                variant="primary"
                size="lg"
                className="flex-1"
              >
                <RefreshCw className="w-4 h-4" />
                {t('errorPage.refresh')}
              </PremiumButton>
              <PremiumButton
                onClick={handleGoHome}
                variant="secondary"
                size="lg"
                className="flex-1"
              >
                <Home className="w-4 h-4" />
                {t('errorPage.goHome')}
              </PremiumButton>
            </div>
          </motion.div>
        </GlassCard>

        {/* Support Message */}
        <motion.div
          variants={itemVariants}
          className="flex items-center gap-2 text-xs"
          style={{ color: colors.textMuted }}
        >
          <Zap className="w-3 h-3" style={{ color: colors.cyan }} />
          <p>{t('errorPage.contactSupport')}</p>
        </motion.div>

        {/* Decorative Pattern */}
        <motion.div
          variants={itemVariants}
          className="absolute -bottom-4 left-1/2 transform -translate-x-1/2 w-48 h-1 rounded-full"
          style={{
            background: colors.gradientPremium,
            opacity: 0.3,
          }}
        />
      </motion.div>
    </PageWrapper>
  );
};

export default Error;
