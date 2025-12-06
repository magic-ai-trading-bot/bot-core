import { useState, useEffect } from "react";
import { motion } from "framer-motion";
import logger from "@/utils/logger";
import { useNavigate, Link } from "react-router-dom";
import { useAuth } from "@/contexts/AuthContext";
import { toast } from "sonner";
import ChatBot from "@/components/ChatBot";
import { Logo } from "@/components/ui/Logo";
import {
  luxuryColors,
  GlassCard,
  GradientText,
  PremiumButton,
  PremiumInput,
  containerVariants,
  itemVariants,
  GlowIcon,
  Badge,
} from '@/styles/luxury-design-system';
import { Sparkles, TrendingUp, BarChart3, Shield } from "lucide-react";

const Register = () => {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [fullName, setFullName] = useState("");
  const navigate = useNavigate();
  const { register, isAuthenticated, loading, error } = useAuth();

  // Redirect if already authenticated
  useEffect(() => {
    if (isAuthenticated) {
      navigate("/dashboard", { replace: true });
    }
  }, [isAuthenticated, navigate]);

  const handleRegister = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!email || !password || !confirmPassword) {
      toast.error("Registration Error", {
        description: "Please fill in all required fields",
      });
      return;
    }

    if (password !== confirmPassword) {
      toast.error("Registration Error", {
        description: "Passwords do not match",
      });
      return;
    }

    if (password.length < 6) {
      toast.error("Registration Error", {
        description: "Password must be at least 6 characters",
      });
      return;
    }

    try {
      toast.loading("Creating account...", { id: "register-loading" });

      const success = await register(email, password, fullName || undefined);

      if (success) {
        toast.success("Registration Successful", {
          description: "Welcome to Trading Bot Dashboard!",
          id: "register-loading",
        });
        navigate("/dashboard", { replace: true });
      } else {
        toast.error("Registration Error", {
          description: error || "Could not create account. Please try again.",
          id: "register-loading",
        });
      }
    } catch (err) {
      logger.error("Registration error:", err);
      toast.error("Registration Error", {
        description: "An error occurred during registration. Please try again.",
        id: "register-loading",
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
        {/* Gradient Orbs */}
        <div
          className="absolute top-1/4 -left-1/4 w-96 h-96 rounded-full blur-3xl opacity-20"
          style={{
            background: luxuryColors.gradientPremium,
          }}
        />
        <div
          className="absolute bottom-1/4 -right-1/4 w-96 h-96 rounded-full blur-3xl opacity-20"
          style={{
            background: luxuryColors.gradientProfit,
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
            <span style={{ color: luxuryColors.textPrimary }}>Bot</span>
            <GradientText className="ml-1">Core</GradientText>
          </h1>
          <p
            className="mt-2 text-sm md:text-base tracking-wide"
            style={{ color: luxuryColors.textMuted }}
          >
            Create an account to start trading
          </p>
        </motion.div>

        {/* Register Card */}
        <GlassCard>
          <motion.div variants={itemVariants} className="space-y-6">
            {/* Header */}
            <div className="text-center">
              <h2 className="text-2xl font-black tracking-tight">
                <GradientText>Sign Up</GradientText>
              </h2>
              <p
                className="text-xs mt-1.5 tracking-wide"
                style={{ color: luxuryColors.textMuted }}
              >
                Join the next generation of traders
              </p>
            </div>

            {/* Form */}
            <form onSubmit={handleRegister} className="space-y-4">
              <PremiumInput
                label="Full Name (optional)"
                value={fullName}
                onChange={setFullName}
                placeholder="John Doe"
                type="text"
              />

              <PremiumInput
                label="Email"
                value={email}
                onChange={setEmail}
                placeholder="your@email.com"
                type="email"
              />

              <PremiumInput
                label="Password"
                value={password}
                onChange={setPassword}
                placeholder="Enter your password"
                type="password"
              />

              <PremiumInput
                label="Confirm Password"
                value={confirmPassword}
                onChange={setConfirmPassword}
                placeholder="Re-enter your password"
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
                {loading ? "Creating account..." : "Create Account"}
              </PremiumButton>
            </form>

            {/* Login Link */}
            <motion.div
              className="text-center pt-4 border-t"
              style={{ borderColor: luxuryColors.borderSubtle }}
              variants={itemVariants}
            >
              <p className="text-sm" style={{ color: luxuryColors.textSecondary }}>
                Already have an account?{" "}
                <Link
                  to="/login"
                  className="font-bold hover:underline transition-all"
                  style={{ color: luxuryColors.cyan }}
                >
                  Sign in now
                </Link>
              </p>
            </motion.div>

            {/* Features Preview */}
            <motion.div
              className="pt-4 border-t space-y-3"
              style={{ borderColor: luxuryColors.borderSubtle }}
              variants={itemVariants}
            >
              <div className="flex items-center gap-3">
                <GlowIcon icon={Sparkles} color={luxuryColors.emerald} size="sm" />
                <span className="text-sm" style={{ color: luxuryColors.textSecondary }}>
                  AI-Powered Trading Signals
                </span>
              </div>
              <div className="flex items-center gap-3">
                <GlowIcon icon={TrendingUp} color={luxuryColors.cyan} size="sm" />
                <span className="text-sm" style={{ color: luxuryColors.textSecondary }}>
                  Real-time Performance Analytics
                </span>
              </div>
              <div className="flex items-center gap-3">
                <GlowIcon icon={BarChart3} color={luxuryColors.amber} size="sm" />
                <span className="text-sm" style={{ color: luxuryColors.textSecondary }}>
                  Advanced Risk Management
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
          <GlowIcon icon={Shield} color={luxuryColors.emerald} size="sm" />
          <p className="text-xs tracking-wide" style={{ color: luxuryColors.textMuted }}>
            Secured with end-to-end encryption and 2FA
          </p>
        </motion.div>
      </motion.div>

      {/* Chatbot Widget */}
      <ChatBot />
    </div>
  );
};

export default Register;
