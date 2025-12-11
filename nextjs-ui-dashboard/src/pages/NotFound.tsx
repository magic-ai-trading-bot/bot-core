import { useLocation, useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import logger from "@/utils/logger";
import { useEffect } from "react";
import { motion } from "framer-motion";
import { Search, Home, ArrowLeft, MapPin } from "lucide-react";
import ChatBot from "@/components/ChatBot";
import {
  GlassCard,
  GradientText,
  PremiumButton,
  PageWrapper,
  containerVariants,
  itemVariants,
} from "@/styles/luxury-design-system";
import { useThemeColors } from "@/hooks/useThemeColors";

const NotFound = () => {
  const { t } = useTranslation('errors');
  const colors = useThemeColors();
  const location = useLocation();
  const navigate = useNavigate();

  useEffect(() => {
    logger.error(
      "404 Error: User attempted to access non-existent route:",
      location.pathname
    );
  }, [location.pathname]);

  const handleGoBack = () => {
    navigate(-1);
  };

  const handleGoHome = () => {
    navigate("/");
  };

  return (
    <PageWrapper className="min-h-screen flex items-center justify-center relative overflow-hidden">
      {/* Decorative Background Orbs */}
      <motion.div
        className="absolute top-1/3 left-1/3 w-72 h-72 rounded-full blur-3xl opacity-20"
        style={{ background: colors.gradientCyan }}
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
        className="absolute bottom-1/3 right-1/3 w-72 h-72 rounded-full blur-3xl opacity-20"
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
        className="flex max-w-2xl flex-col items-center gap-8 text-center relative z-10 px-4"
        variants={containerVariants}
      >
        {/* 404 Icon with Glow */}
        <motion.div
          variants={itemVariants}
          className="relative"
        >
          <motion.div
            className="absolute inset-0 rounded-full blur-2xl"
            style={{ background: colors.gradientPremium, opacity: 0.3 }}
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
            glowColor="0 8px 32px rgba(0, 217, 255, 0.3)"
          >
            <Search
              className="h-24 w-24"
              style={{ color: colors.cyan }}
            />
          </GlassCard>
        </motion.div>

        {/* Error Message Card */}
        <GlassCard className="w-full">
          <motion.div variants={itemVariants} className="space-y-4">
            {/* 404 Code */}
            <div>
              <GradientText
                className="text-7xl font-black tracking-tight"
                gradient={colors.gradientPremium}
              >
                404
              </GradientText>
            </div>

            {/* Error Title */}
            <div>
              <h2 className="text-2xl font-bold mb-2 text-black dark:text-white">
                {t('notFoundPage.title')}
              </h2>
              <p className="text-sm leading-relaxed mb-3 text-black/65 dark:text-white/65">
                {t('notFoundPage.description')}
              </p>

              {/* Current Path Display */}
              <div className="flex items-center justify-center gap-2 text-xs">
                <MapPin className="w-3 h-3 text-black/40 dark:text-white/40" />
                <code className="px-2 py-1 rounded font-mono bg-black/5 dark:bg-white/5 text-black/50 dark:text-white/50">
                  {location.pathname}
                </code>
              </div>
            </div>

            {/* Action Buttons */}
            <div className="flex flex-col sm:flex-row gap-3 pt-4">
              <PremiumButton
                onClick={handleGoBack}
                variant="secondary"
                size="lg"
                className="flex-1"
              >
                <ArrowLeft className="w-4 h-4" />
                {t('notFoundPage.goBack')}
              </PremiumButton>
              <PremiumButton
                onClick={handleGoHome}
                variant="primary"
                size="lg"
                className="flex-1"
              >
                <Home className="w-4 h-4" />
                {t('notFoundPage.goHome')}
              </PremiumButton>
            </div>
          </motion.div>
        </GlassCard>

        {/* Popular Links */}
        <motion.div
          variants={itemVariants}
          className="w-full"
        >
          <GlassCard noPadding>
            <div className="p-4">
              <p className="text-xs uppercase tracking-wider mb-3 font-bold text-black/40 dark:text-white/40">
                {t('notFoundPage.popularPages')}
              </p>
              <div className="grid grid-cols-2 sm:grid-cols-4 gap-2">
                {[
                  { label: t('notFoundPage.pages.dashboard'), path: "/" },
                  { label: t('notFoundPage.pages.portfolio'), path: "/portfolio" },
                  { label: t('notFoundPage.pages.strategies'), path: "/strategies" },
                  { label: t('notFoundPage.pages.settings'), path: "/settings" },
                ].map((link, index) => (
                  <motion.button
                    key={link.path}
                    onClick={() => navigate(link.path)}
                    className="px-3 py-2 rounded-lg text-xs font-medium transition-all duration-300 bg-black/[0.03] dark:bg-white/[0.03] border border-black/[0.08] dark:border-white/[0.08] text-black/65 dark:text-white/65 hover:bg-black/[0.08] dark:hover:bg-white/[0.08] hover:border-black/[0.15] dark:hover:border-white/[0.15]"
                    whileHover={{ scale: 1.02 }}
                    whileTap={{ scale: 0.98 }}
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ delay: index * 0.05 }}
                  >
                    {link.label}
                  </motion.button>
                ))}
              </div>
            </div>
          </GlassCard>
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

      {/* Chatbot Widget */}
      <ChatBot />
    </PageWrapper>
  );
};

export default NotFound;
