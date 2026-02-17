import { useState, useEffect } from "react";
import logger from "@/utils/logger";
import { useNavigate, Link } from "react-router-dom";
import { useAuth } from "@/contexts/AuthContext";
import { toast } from "sonner";
import { useTranslation } from "react-i18next";
import ChatBot from "@/components/ChatBot";
import { Logo } from "@/components/ui/Logo";
import { motion } from "framer-motion";
import { Mail, Lock, Sparkles, Shield, CheckCircle2 } from "lucide-react";
import {
  GlassCard,
  GradientText,
  PremiumButton,
  PremiumInput,
  containerVariants,
  itemVariants,
  GlowIcon,
} from "@/styles/luxury-design-system";
import { useThemeColors } from "@/hooks/useThemeColors";

const Login = () => {
  const { t } = useTranslation('auth');
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const navigate = useNavigate();
  const { login, isAuthenticated, loading } = useAuth();
  const colors = useThemeColors();

  // Redirect if already authenticated
  useEffect(() => {
    if (isAuthenticated) {
      navigate("/dashboard", { replace: true });
    }
  }, [isAuthenticated, navigate]);

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!email || !password) {
      toast.error(t('login.error'), {
        description: t('login.errorEmpty'),
      });
      return;
    }

    try {
      toast.loading(t('login.submitting'), { id: "login-loading" });

      const result = await login(email, password);

      if (result.success) {
        toast.success(t('login.success'), {
          description: t('login.successMessage'),
          id: "login-loading",
        });
        navigate("/dashboard", { replace: true });
      } else {
        toast.error(t('login.error'), {
          description: result.error || t('login.errorInvalid'),
          id: "login-loading",
        });
      }
    } catch (err) {
      logger.error("Login error:", err);
      toast.error(t('login.error'), {
        description: t('login.errorGeneric'),
        id: "login-loading",
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
        {/* Radial gradient glow */}
        <div
          className="absolute inset-0"
          style={{
            background: `radial-gradient(ellipse at center top, ${colors.cyan}15 0%, transparent 50%)`,
          }}
        />
        {/* Grid pattern */}
        <div
          className="absolute inset-0 opacity-[0.02]"
          style={{
            backgroundImage: `linear-gradient(${colors.borderSubtle} 1px, transparent 1px), linear-gradient(90deg, ${colors.borderSubtle} 1px, transparent 1px)`,
            backgroundSize: '50px 50px',
          }}
        />
      </div>

      <motion.div
        className="relative z-10 w-full max-w-md"
        initial="hidden"
        animate="visible"
        variants={containerVariants}
      >
        {/* Logo/Brand */}
        <motion.div variants={itemVariants} className="text-center mb-8">
          <div className="flex justify-center mb-6">
            <motion.div
              whileHover={{ scale: 1.05, rotate: 5 }}
              transition={{ type: "spring", stiffness: 300 }}
            >
              <Logo size="xl" showText={false} />
            </motion.div>
          </div>
          <h1 className="text-3xl md:text-4xl font-black mb-2 tracking-tight">
            <span style={{ color: colors.textPrimary }}>Bot</span>
            <GradientText className="ml-1">Core</GradientText>
          </h1>
          <p
            className="text-sm tracking-wide"
            style={{ color: colors.textMuted }}
          >
            {t('login.tagline')}
          </p>
        </motion.div>

        {/* Login Form Card */}
        <GlassCard className="mb-6">
          <div className="mb-6">
            <div className="flex items-center justify-center gap-2 mb-2">
              <GlowIcon icon={Sparkles} size="sm" />
              <h2 className="text-xl font-black" style={{ color: colors.textPrimary }}>
                {t('login.title')}
              </h2>
            </div>
            <p
              className="text-xs text-center tracking-wide"
              style={{ color: colors.textMuted }}
            >
              {t('login.subtitle')}
            </p>
          </div>

          <form onSubmit={handleLogin} className="space-y-4">
            <PremiumInput
              label={t('login.emailLabel')}
              type="email"
              value={email}
              onChange={setEmail}
              placeholder={t('login.emailPlaceholder')}
              prefix={<Mail className="w-4 h-4" />}
            />

            <PremiumInput
              label={t('login.passwordLabel')}
              type="password"
              value={password}
              onChange={setPassword}
              placeholder={t('login.passwordPlaceholder')}
              prefix={<Lock className="w-4 h-4" />}
            />

            <PremiumButton
              type="submit"
              variant="primary"
              size="lg"
              fullWidth
              disabled={loading}
              loading={loading}
            >
              {loading ? t('login.submitting') : t('login.submitButton')}
            </PremiumButton>
          </form>

          {/* Register Link */}
          <motion.div variants={itemVariants} className="mt-6 text-center">
            <p className="text-xs" style={{ color: colors.textMuted }}>
              {t('login.noAccount')}{" "}
              <Link
                to="/register"
                className="font-bold hover:underline transition-colors"
                style={{ color: colors.cyan }}
              >
                {t('login.signUpLink')}
              </Link>
            </p>
          </motion.div>

          {/* Demo Credentials */}
          <motion.div
            variants={itemVariants}
            className="mt-6 p-4 rounded-xl"
            style={{
              backgroundColor: 'rgba(255, 255, 255, 0.02)',
              border: `1px solid ${colors.borderSubtle}`,
            }}
          >
            <p
              className="text-[10px] uppercase tracking-wider mb-3 font-bold"
              style={{ color: colors.textMuted }}
            >
              {t('login.demoCredentials')}
            </p>
            <div className="space-y-2 text-xs mb-3">
              <div className="flex justify-between items-center">
                <span style={{ color: colors.textMuted }}>Email:</span>
                <span className="font-mono" style={{ color: colors.textSecondary }}>
                  trader@botcore.com
                </span>
              </div>
              <div className="flex justify-between items-center">
                <span style={{ color: colors.textMuted }}>Password:</span>
                <span className="font-mono" style={{ color: colors.textSecondary }}>
                  password123
                </span>
              </div>
            </div>
            <div className="flex gap-2">
              <motion.button
                type="button"
                disabled={loading}
                onClick={async () => {
                  setEmail('trader@botcore.com');
                  setPassword('password123');
                  try {
                    toast.loading(t('login.submitting'), { id: "login-loading" });
                    const result = await login('trader@botcore.com', 'password123');
                    if (result.success) {
                      toast.success(t('login.success'), { description: t('login.successMessage'), id: "login-loading" });
                      navigate("/dashboard", { replace: true });
                    } else {
                      toast.error(t('login.error'), { description: result.error || t('login.errorInvalid'), id: "login-loading" });
                    }
                  } catch (err) {
                    logger.error("Demo login error:", err);
                    toast.error(t('login.error'), { description: t('login.errorGeneric'), id: "login-loading" });
                  }
                }}
                whileHover={loading ? undefined : { scale: 1.02 }}
                whileTap={loading ? undefined : { scale: 0.98 }}
                className="flex-1 px-3 py-2 text-xs font-bold rounded-lg transition-all"
                style={{
                  background: `rgba(${34}, ${197}, ${94}, 0.1)`,
                  border: `1px solid rgba(${34}, ${197}, ${94}, 0.3)`,
                  color: colors.profit,
                  opacity: loading ? 0.5 : 1,
                }}
              >
                {t('login.useTrader')}
              </motion.button>
              <motion.button
                type="button"
                disabled={loading}
                onClick={async () => {
                  setEmail('admin@botcore.com');
                  setPassword('password123');
                  try {
                    toast.loading(t('login.submitting'), { id: "login-loading" });
                    const result = await login('admin@botcore.com', 'password123');
                    if (result.success) {
                      toast.success(t('login.success'), { description: t('login.successMessage'), id: "login-loading" });
                      navigate("/dashboard", { replace: true });
                    } else {
                      toast.error(t('login.error'), { description: result.error || t('login.errorInvalid'), id: "login-loading" });
                    }
                  } catch (err) {
                    logger.error("Demo login error:", err);
                    toast.error(t('login.error'), { description: t('login.errorGeneric'), id: "login-loading" });
                  }
                }}
                whileHover={loading ? undefined : { scale: 1.02 }}
                whileTap={loading ? undefined : { scale: 0.98 }}
                className="flex-1 px-3 py-2 text-xs font-bold rounded-lg transition-all"
                style={{
                  background: `rgba(${139}, ${92}, ${246}, 0.1)`,
                  border: `1px solid rgba(${139}, ${92}, ${246}, 0.3)`,
                  color: colors.purple,
                  opacity: loading ? 0.5 : 1,
                }}
              >
                {t('login.useAdmin')}
              </motion.button>
            </div>
          </motion.div>

          {/* Features Preview */}
          <motion.div variants={itemVariants} className="mt-6 space-y-3">
            <div className="flex items-center gap-3 text-xs">
              <div className="flex-shrink-0">
                <CheckCircle2 className="w-4 h-4" style={{ color: colors.profit }} />
              </div>
              <span style={{ color: colors.textSecondary }}>
                {t('features.aiSignals')}
              </span>
            </div>
            <div className="flex items-center gap-3 text-xs">
              <div className="flex-shrink-0">
                <CheckCircle2 className="w-4 h-4" style={{ color: colors.cyan }} />
              </div>
              <span style={{ color: colors.textSecondary }}>
                {t('features.analytics')}
              </span>
            </div>
            <div className="flex items-center gap-3 text-xs">
              <div className="flex-shrink-0">
                <CheckCircle2 className="w-4 h-4" style={{ color: colors.purple }} />
              </div>
              <span style={{ color: colors.textSecondary }}>
                {t('features.riskManagement')}
              </span>
            </div>
          </motion.div>
        </GlassCard>

        {/* Security Badge */}
        <motion.div
          variants={itemVariants}
          className="text-center"
        >
          <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full"
            style={{
              backgroundColor: 'rgba(255, 255, 255, 0.02)',
              border: `1px solid ${colors.borderSubtle}`,
            }}
          >
            <Shield className="w-3 h-3" style={{ color: colors.cyan }} />
            <span
              className="text-[10px] uppercase tracking-wider font-bold"
              style={{ color: colors.textMuted }}
            >
              {t('login.securityBadge')}
            </span>
          </div>
        </motion.div>
      </motion.div>

      {/* Chatbot Widget */}
      <ChatBot />
    </div>
  );
};

export default Login;
