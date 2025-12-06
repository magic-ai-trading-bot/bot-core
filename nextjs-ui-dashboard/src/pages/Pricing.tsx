import { motion } from "framer-motion";
import { Link } from "react-router-dom";
import {
  Check,
  Zap,
  Crown,
  Building2,
  ArrowLeft,
  ChevronRight,
} from "lucide-react";
import {
  luxuryColors,
  GlassCard,
  GradientText,
  PremiumButton,
  Badge,
  GlowIcon,
  PageWrapper,
} from "@/styles/luxury-design-system";

const plans = [
  {
    name: "Starter",
    icon: Zap,
    price: "Free",
    period: "",
    description: "Perfect for learning and paper trading",
    features: [
      "Paper Trading Only",
      "3 Active Strategies",
      "Basic AI Signals",
      "Community Support",
      "1 Exchange Connection",
      "Daily Performance Reports",
    ],
    cta: "Start Free",
    variant: "secondary" as const,
    popular: false,
  },
  {
    name: "Pro",
    icon: Crown,
    price: "$49",
    period: "/month",
    description: "For serious traders ready to go live",
    features: [
      "Live Trading Enabled",
      "Unlimited Strategies",
      "Advanced AI Analysis",
      "Priority Support",
      "3 Exchange Connections",
      "Real-time Alerts",
      "Advanced Risk Management",
      "Custom Strategy Builder",
    ],
    cta: "Get Started",
    variant: "primary" as const,
    popular: true,
  },
  {
    name: "Enterprise",
    icon: Building2,
    price: "Custom",
    period: "",
    description: "For funds and professional traders",
    features: [
      "Everything in Pro",
      "Unlimited Exchanges",
      "Dedicated Account Manager",
      "Custom API Integration",
      "White-label Options",
      "SLA Guarantee",
      "On-premise Deployment",
      "Custom AI Model Training",
    ],
    cta: "Contact Sales",
    variant: "secondary" as const,
    popular: false,
  },
];

const faqs = [
  {
    question: "Can I switch plans anytime?",
    answer: "Yes, you can upgrade or downgrade your plan at any time. Changes take effect immediately, and we'll prorate your billing.",
  },
  {
    question: "Is there a free trial for Pro?",
    answer: "Yes! You get a 14-day free trial of Pro features. No credit card required.",
  },
  {
    question: "What payment methods do you accept?",
    answer: "We accept all major credit cards, PayPal, and cryptocurrency (BTC, ETH, USDT).",
  },
  {
    question: "Can I cancel anytime?",
    answer: "Absolutely. Cancel your subscription anytime with no cancellation fees.",
  },
];

const Pricing = () => {
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
            Back to Home
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
          Simple Pricing
        </Badge>
        <h1 className="text-4xl md:text-5xl font-black mb-4">
          <GradientText>Choose Your</GradientText>
          <br />
          <span style={{ color: luxuryColors.textPrimary }}>Trading Plan</span>
        </h1>
        <p className="text-lg max-w-2xl mx-auto" style={{ color: luxuryColors.textMuted }}>
          Start free and scale as you grow. No hidden fees, cancel anytime.
        </p>
      </motion.div>

      {/* Pricing Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-16 max-w-5xl mx-auto">
        {plans.map((plan, index) => (
          <motion.div
            key={plan.name}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: index * 0.1 }}
            className="relative"
          >
            {plan.popular && (
              <div className="absolute -top-3 left-1/2 -translate-x-1/2 z-10">
                <Badge variant="warning" glow>Most Popular</Badge>
              </div>
            )}
            <GlassCard
              className={`h-full ${plan.popular ? 'border-cyan-500/50 ring-2 ring-cyan-500/20' : ''}`}
            >
              <div className="text-center mb-6">
                <GlowIcon
                  icon={plan.icon}
                  size="lg"
                  color={plan.popular ? luxuryColors.cyan : luxuryColors.textMuted}
                  className="mx-auto mb-4"
                />
                <h3 className="text-xl font-bold mb-2" style={{ color: luxuryColors.textPrimary }}>
                  {plan.name}
                </h3>
                <div className="mb-2">
                  <span className="text-4xl font-black" style={{ color: luxuryColors.cyan }}>
                    {plan.price}
                  </span>
                  <span style={{ color: luxuryColors.textMuted }}>{plan.period}</span>
                </div>
                <p className="text-sm" style={{ color: luxuryColors.textMuted }}>
                  {plan.description}
                </p>
              </div>

              <ul className="space-y-3 mb-6">
                {plan.features.map((feature) => (
                  <li key={feature} className="flex items-center gap-2">
                    <Check className="w-4 h-4" style={{ color: luxuryColors.profit }} />
                    <span className="text-sm" style={{ color: luxuryColors.textSecondary }}>
                      {feature}
                    </span>
                  </li>
                ))}
              </ul>

              <Link to={plan.name === "Enterprise" ? "/contact" : "/register"}>
                <PremiumButton variant={plan.variant} fullWidth>
                  {plan.cta}
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
        <h2 className="text-2xl font-bold text-center mb-8" style={{ color: luxuryColors.textPrimary }}>
          Frequently Asked Questions
        </h2>
        <div className="space-y-4">
          {faqs.map((faq, index) => (
            <GlassCard key={index} noPadding className="p-4">
              <h4 className="font-semibold mb-2" style={{ color: luxuryColors.textPrimary }}>
                {faq.question}
              </h4>
              <p className="text-sm" style={{ color: luxuryColors.textMuted }}>
                {faq.answer}
              </p>
            </GlassCard>
          ))}
        </div>
      </motion.div>
    </PageWrapper>
  );
};

export default Pricing;
