import { useState, useEffect } from "react";
import logger from "@/utils/logger";
import { useNavigate, Link } from "react-router-dom";
import { useAuth } from "@/contexts/AuthContext";
import { toast } from "sonner";
import ChatBot from "@/components/ChatBot";
import { Logo } from "@/components/ui/Logo";
import { motion } from "framer-motion";
import { Mail, Lock, Sparkles, Shield, TrendingUp, Brain, CheckCircle2 } from "lucide-react";
import {
  luxuryColors,
  GlassCard,
  GradientText,
  PremiumButton,
  PremiumInput,
  containerVariants,
  itemVariants,
  GlowIcon,
} from "@/styles/luxury-design-system";

const Login = () => {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const navigate = useNavigate();
  const { login, isAuthenticated, loading, error } = useAuth();

  // Redirect if already authenticated
  useEffect(() => {
    if (isAuthenticated) {
      navigate("/dashboard", { replace: true });
    }
  }, [isAuthenticated, navigate]);

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!email || !password) {
      toast.error("Login Error", {
        description: "Please enter your email and password",
      });
      return;
    }

    try {
      toast.loading("Signing in...", { id: "login-loading" });

      const success = await login(email, password);

      if (success) {
        toast.success("Login Successful", {
          description: "Welcome back to Trading Bot Dashboard!",
          id: "login-loading",
        });
        navigate("/dashboard", { replace: true });
      } else {
        toast.error("Login Error", {
          description: error || "Invalid credentials",
          id: "login-loading",
        });
      }
    } catch (err) {
      logger.error("Login error:", err);
      toast.error("Login Error", {
        description: "An error occurred while signing in. Please try again.",
        id: "login-loading",
      });
    }
  };

  return (
    <div
      className="min-h-screen flex items-center justify-center p-4 relative overflow-hidden"
      style={{ backgroundColor: luxuryColors.bgPrimary }}
    >
      {/* Premium Background Effects */}
      <div className="absolute inset-0">
        {/* Radial gradient glow */}
        <div
          className="absolute inset-0"
          style={{
            background: `radial-gradient(ellipse at center top, ${luxuryColors.cyan}15 0%, transparent 50%)`,
          }}
        />
        {/* Grid pattern */}
        <div
          className="absolute inset-0 opacity-[0.02]"
          style={{
            backgroundImage: `linear-gradient(${luxuryColors.borderSubtle} 1px, transparent 1px), linear-gradient(90deg, ${luxuryColors.borderSubtle} 1px, transparent 1px)`,
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
            <span style={{ color: luxuryColors.textPrimary }}>Bot</span>
            <GradientText className="ml-1">Core</GradientText>
          </h1>
          <p
            className="text-sm tracking-wide"
            style={{ color: luxuryColors.textMuted }}
          >
            AI-Powered Trading Intelligence Platform
          </p>
        </motion.div>

        {/* Login Form Card */}
        <GlassCard className="mb-6">
          <div className="mb-6">
            <div className="flex items-center justify-center gap-2 mb-2">
              <GlowIcon icon={Sparkles} size="sm" />
              <h2 className="text-xl font-black" style={{ color: luxuryColors.textPrimary }}>
                Sign In
              </h2>
            </div>
            <p
              className="text-xs text-center tracking-wide"
              style={{ color: luxuryColors.textMuted }}
            >
              Access your premium trading dashboard
            </p>
          </div>

          <form onSubmit={handleLogin} className="space-y-4">
            <PremiumInput
              label="Email Address"
              type="email"
              value={email}
              onChange={setEmail}
              placeholder="trader@botcore.com"
              prefix={<Mail className="w-4 h-4" />}
            />

            <PremiumInput
              label="Password"
              type="password"
              value={password}
              onChange={setPassword}
              placeholder="Enter your password"
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
              {loading ? "Signing in..." : "Sign In"}
            </PremiumButton>
          </form>

          {/* Register Link */}
          <motion.div variants={itemVariants} className="mt-6 text-center">
            <p className="text-xs" style={{ color: luxuryColors.textMuted }}>
              Don't have an account?{" "}
              <Link
                to="/register"
                className="font-bold hover:underline transition-colors"
                style={{ color: luxuryColors.cyan }}
              >
                Sign up now
              </Link>
            </p>
          </motion.div>

          {/* Demo Credentials */}
          <motion.div
            variants={itemVariants}
            className="mt-6 p-4 rounded-xl"
            style={{
              backgroundColor: 'rgba(255, 255, 255, 0.02)',
              border: `1px solid ${luxuryColors.borderSubtle}`,
            }}
          >
            <p
              className="text-[10px] uppercase tracking-wider mb-3 font-bold"
              style={{ color: luxuryColors.textMuted }}
            >
              Demo Credentials
            </p>
            <div className="space-y-2 text-xs mb-3">
              <div className="flex justify-between items-center">
                <span style={{ color: luxuryColors.textMuted }}>Email:</span>
                <span className="font-mono" style={{ color: luxuryColors.textSecondary }}>
                  trader@botcore.com
                </span>
              </div>
              <div className="flex justify-between items-center">
                <span style={{ color: luxuryColors.textMuted }}>Password:</span>
                <span className="font-mono" style={{ color: luxuryColors.textSecondary }}>
                  password123
                </span>
              </div>
            </div>
            <div className="flex gap-2">
              <motion.button
                type="button"
                onClick={() => {
                  setEmail('trader@botcore.com');
                  setPassword('password123');
                }}
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.98 }}
                className="flex-1 px-3 py-2 text-xs font-bold rounded-lg transition-all"
                style={{
                  background: `rgba(${34}, ${197}, ${94}, 0.1)`,
                  border: `1px solid rgba(${34}, ${197}, ${94}, 0.3)`,
                  color: luxuryColors.profit,
                }}
              >
                Use Trader
              </motion.button>
              <motion.button
                type="button"
                onClick={() => {
                  setEmail('admin@botcore.com');
                  setPassword('password123');
                }}
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.98 }}
                className="flex-1 px-3 py-2 text-xs font-bold rounded-lg transition-all"
                style={{
                  background: `rgba(${139}, ${92}, ${246}, 0.1)`,
                  border: `1px solid rgba(${139}, ${92}, ${246}, 0.3)`,
                  color: luxuryColors.purple,
                }}
              >
                Use Admin
              </motion.button>
            </div>
          </motion.div>

          {/* Features Preview */}
          <motion.div variants={itemVariants} className="mt-6 space-y-3">
            <div className="flex items-center gap-3 text-xs">
              <div className="flex-shrink-0">
                <CheckCircle2 className="w-4 h-4" style={{ color: luxuryColors.profit }} />
              </div>
              <span style={{ color: luxuryColors.textSecondary }}>
                AI-Powered Trading Signals
              </span>
            </div>
            <div className="flex items-center gap-3 text-xs">
              <div className="flex-shrink-0">
                <CheckCircle2 className="w-4 h-4" style={{ color: luxuryColors.cyan }} />
              </div>
              <span style={{ color: luxuryColors.textSecondary }}>
                Real-time Performance Analytics
              </span>
            </div>
            <div className="flex items-center gap-3 text-xs">
              <div className="flex-shrink-0">
                <CheckCircle2 className="w-4 h-4" style={{ color: luxuryColors.purple }} />
              </div>
              <span style={{ color: luxuryColors.textSecondary }}>
                Advanced Risk Management
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
              border: `1px solid ${luxuryColors.borderSubtle}`,
            }}
          >
            <Shield className="w-3 h-3" style={{ color: luxuryColors.cyan }} />
            <span
              className="text-[10px] uppercase tracking-wider font-bold"
              style={{ color: luxuryColors.textMuted }}
            >
              Secured with E2E Encryption & 2FA
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
