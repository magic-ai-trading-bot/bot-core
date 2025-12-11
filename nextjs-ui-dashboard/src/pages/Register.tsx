import { useState, useEffect } from "react";
import { motion } from "framer-motion";
import logger from "@/utils/logger";
import { useNavigate, Link } from "react-router-dom";
import { useAuth } from "@/contexts/AuthContext";
import { toast } from "sonner";
import { useTranslation } from "react-i18next";
import ChatBot from "@/components/ChatBot";
import { Logo } from "@/components/ui/Logo";
import {
  GlassCard,
  GradientText,
  PremiumButton,
  PremiumInput,
  containerVariants,
  itemVariants,
  GlowIcon,
} from '@/styles/luxury-design-system';
import { useThemeColors } from "@/hooks/useThemeColors";
import { Sparkles, TrendingUp, BarChart3, Shield } from "lucide-react";

const Register = () => {
  const { t } = useTranslation('auth');
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [fullName, setFullName] = useState("");
  const navigate = useNavigate();
  const { register, isAuthenticated, loading, error } = useAuth();
  const colors = useThemeColors();

  // Redirect if already authenticated
  useEffect(() => {
    if (isAuthenticated) {
      navigate("/dashboard", { replace: true });
    }
  }, [isAuthenticated, navigate]);

  const handleRegister = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!email || !password || !confirmPassword) {
      toast.error(t('register.error'), {
        description: t('register.errorEmpty'),
      });
      return;
    }

    if (password !== confirmPassword) {
      toast.error(t('register.error'), {
        description: t('register.errorPasswordMismatch'),
      });
      return;
    }

    if (password.length < 6) {
      toast.error(t('register.error'), {
        description: t('register.errorPasswordLength'),
      });
      return;
    }

    try {
      toast.loading(t('register.submitting'), { id: "register-loading" });

      const success = await register(email, password, fullName || undefined);

      if (success) {
        toast.success(t('register.success'), {
          description: t('register.successMessage'),
          id: "register-loading",
        });
        navigate("/dashboard", { replace: true });
      } else {
        toast.error(t('register.error'), {
          description: error || t('register.errorGeneric'),
          id: "register-loading",
        });
      }
    } catch (err) {
      logger.error("Registration error:", err);
      toast.error(t('register.error'), {
        description: t('register.errorGeneric'),
        id: "register-loading",
      });
    }
  };

  return (
    <div
      className="min-h-screen flex items-center justify-center p-4 relative overflow-hidden"
      style={{ backgroundColor: colors.bgPrimary }}
    >
      {/* Premium Background Effects */}
      <div className="absolute inset-0">
        {/* Gradient Orbs */}
        <div
          className="absolute top-1/4 -left-1/4 w-96 h-96 rounded-full blur-3xl opacity-20"
          style={{
            background: colors.gradientPremium,
          }}
        />
        <div
          className="absolute bottom-1/4 -right-1/4 w-96 h-96 rounded-full blur-3xl opacity-20"
          style={{
            background: colors.gradientProfit,
          }}
        />
      </div>

      <motion.div
        className="relative z-10 w-full max-w-sm md:max-w-md"
        variants={containerVariants}
        initial="hidden"
        animate="visible"
      >
        {/* Logo/Brand */}
        <motion.div className="text-center mb-6 md:mb-8" variants={itemVariants}>
          <div className="flex justify-center mb-4">
            <motion.div
              whileHover={{ scale: 1.05, rotate: 5 }}
              transition={{ type: "spring", stiffness: 300 }}
            >
              <Logo size="xl" showText={false} />
            </motion.div>
          </div>
          <h1 className="text-2xl md:text-3xl font-black tracking-tight">
            <span style={{ color: colors.textPrimary }}>Bot</span>
            <GradientText className="ml-1">Core</GradientText>
          </h1>
          <p
            className="mt-2 text-sm md:text-base tracking-wide"
            style={{ color: colors.textMuted }}
          >
            {t('register.subtitle')}
          </p>
        </motion.div>

        {/* Register Card */}
        <GlassCard>
          <motion.div variants={itemVariants} className="space-y-6">
            {/* Header */}
            <div className="text-center">
              <h2 className="text-2xl font-black tracking-tight">
                <GradientText>{t('register.title')}</GradientText>
              </h2>
              <p
                className="text-xs mt-1.5 tracking-wide"
                style={{ color: colors.textMuted }}
              >
                {t('register.tagline')}
              </p>
            </div>

            {/* Form */}
            <form onSubmit={handleRegister} className="space-y-4">
              <PremiumInput
                label={t('register.nameLabel')}
                value={fullName}
                onChange={setFullName}
                placeholder={t('register.namePlaceholder')}
                type="text"
              />

              <PremiumInput
                label={t('register.emailLabel')}
                value={email}
                onChange={setEmail}
                placeholder={t('register.emailPlaceholder')}
                type="email"
              />

              <PremiumInput
                label={t('register.passwordLabel')}
                value={password}
                onChange={setPassword}
                placeholder={t('register.passwordPlaceholder')}
                type="password"
              />

              <PremiumInput
                label={t('register.confirmPasswordLabel')}
                value={confirmPassword}
                onChange={setConfirmPassword}
                placeholder={t('register.confirmPasswordPlaceholder')}
                type="password"
              />

              <PremiumButton
                type="submit"
                variant="success"
                size="lg"
                fullWidth
                disabled={loading}
                loading={loading}
              >
                {loading ? t('register.submitting') : t('register.submitButton')}
              </PremiumButton>
            </form>

            {/* Login Link */}
            <motion.div
              className="text-center pt-4 border-t"
              style={{ borderColor: colors.borderSubtle }}
              variants={itemVariants}
            >
              <p className="text-sm" style={{ color: colors.textSecondary }}>
                {t('register.hasAccount')}{" "}
                <Link
                  to="/login"
                  className="font-bold hover:underline transition-all"
                  style={{ color: colors.cyan }}
                >
                  {t('register.signInLink')}
                </Link>
              </p>
            </motion.div>

            {/* Features Preview */}
            <motion.div
              className="pt-4 border-t space-y-3"
              style={{ borderColor: colors.borderSubtle }}
              variants={itemVariants}
            >
              <div className="flex items-center gap-3">
                <GlowIcon icon={Sparkles} color={colors.emerald} size="sm" />
                <span className="text-sm" style={{ color: colors.textSecondary }}>
                  {t('features.aiSignals')}
                </span>
              </div>
              <div className="flex items-center gap-3">
                <GlowIcon icon={TrendingUp} color={colors.cyan} size="sm" />
                <span className="text-sm" style={{ color: colors.textSecondary }}>
                  {t('features.analytics')}
                </span>
              </div>
              <div className="flex items-center gap-3">
                <GlowIcon icon={BarChart3} color={colors.amber} size="sm" />
                <span className="text-sm" style={{ color: colors.textSecondary }}>
                  {t('features.riskManagement')}
                </span>
              </div>
            </motion.div>
          </motion.div>
        </GlassCard>

        {/* Security Badge */}
        <motion.div
          className="text-center mt-6 flex items-center justify-center gap-2"
          variants={itemVariants}
        >
          <GlowIcon icon={Shield} color={colors.emerald} size="sm" />
          <p className="text-xs tracking-wide" style={{ color: colors.textMuted }}>
            {t('login.securityBadge')}
          </p>
        </motion.div>
      </motion.div>

      {/* Chatbot Widget */}
      <ChatBot />
    </div>
  );
};

export default Register;
