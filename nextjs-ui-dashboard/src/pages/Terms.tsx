import { motion } from "framer-motion";
import { Link } from "react-router-dom";
import {
  FileText,
  Scale,
  AlertTriangle,
  Ban,
  CreditCard,
  RefreshCw,
  ArrowLeft,
  CheckCircle,
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

const sections = [
  {
    icon: Scale,
    title: "1. Acceptance of Terms",
    content: `By accessing or using Bot Core's services, you agree to be bound by these Terms of Service. If you do not agree to all of these terms, you may not use our services.

These terms apply to all users, including visitors, registered users, and paying subscribers. We reserve the right to modify these terms at any time, and will notify users of significant changes via email or platform notification.`,
  },
  {
    icon: CheckCircle,
    title: "2. Service Description",
    content: `Bot Core provides automated cryptocurrency trading tools, including:
• AI-powered trading signals and analysis
• Automated trading bot execution
• Portfolio management and tracking
• Paper trading simulation
• Real-time market data and alerts

Our services are provided "as is" and we do not guarantee any specific trading outcomes or profits. Past performance does not indicate future results.`,
  },
  {
    icon: AlertTriangle,
    title: "3. Risk Disclosure",
    content: `IMPORTANT: Cryptocurrency trading involves substantial risk of loss and is not suitable for every investor.

• You may lose some or all of your invested capital
• Automated trading systems can experience technical failures
• Market conditions can change rapidly and unexpectedly
• Past performance is not indicative of future results
• You should only trade with funds you can afford to lose

By using our services, you acknowledge that you understand these risks and accept full responsibility for your trading decisions.`,
  },
  {
    icon: Ban,
    title: "4. Prohibited Activities",
    content: `Users are prohibited from:
• Using the service for any illegal purpose
• Attempting to manipulate markets or prices
• Sharing account credentials with others
• Reverse engineering or copying our software
• Using automated tools to scrape our data
• Circumventing any security measures
• Harassing other users or our staff
• Providing false or misleading information

Violation of these terms may result in immediate account termination.`,
  },
  {
    icon: CreditCard,
    title: "5. Payment Terms",
    content: `For paid subscriptions:
• Payments are processed securely via Stripe
• Subscriptions auto-renew unless cancelled
• Refunds are available within 14 days of purchase
• Prices may change with 30 days notice
• Failed payments may result in service suspension

Free tier users may upgrade at any time. Downgrading takes effect at the next billing cycle.`,
  },
  {
    icon: RefreshCw,
    title: "6. Service Modifications",
    content: `We reserve the right to:
• Modify or discontinue features at any time
• Update pricing with appropriate notice
• Suspend service for maintenance
• Terminate accounts that violate terms
• Change API rate limits and quotas

We will provide reasonable notice of significant changes whenever possible.`,
  },
];

const Terms = () => {
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
        className="text-center mb-12"
      >
        <Badge variant="info" className="mb-4">
          Legal
        </Badge>
        <h1 className="text-4xl md:text-5xl font-black mb-4">
          <GradientText>Terms of Service</GradientText>
        </h1>
        <p className="text-lg max-w-2xl mx-auto mb-4" style={{ color: luxuryColors.textMuted }}>
          Please read these terms carefully before using our services.
        </p>
        <div className="flex items-center justify-center gap-2 text-sm" style={{ color: luxuryColors.textMuted }}>
          <FileText className="w-4 h-4" />
          <span>Effective Date: December 1, 2024</span>
        </div>
      </motion.div>

      {/* Important Notice */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.1 }}
        className="mb-12"
      >
        <GlassCard className="border-yellow-500/30">
          <div className="flex items-start gap-4">
            <GlowIcon icon={AlertTriangle} size="lg" color={luxuryColors.warning} />
            <div>
              <h2 className="text-xl font-bold mb-2" style={{ color: luxuryColors.warning }}>
                Important Notice
              </h2>
              <p className="text-sm" style={{ color: luxuryColors.textSecondary }}>
                Bot Core is not a financial advisor. Our tools are for informational purposes only and should not be considered investment advice. Cryptocurrency trading carries significant risk, including the potential loss of your entire investment. Please consult with a qualified financial advisor before making any investment decisions.
              </p>
            </div>
          </div>
        </GlassCard>
      </motion.div>

      {/* Terms Sections */}
      <div className="space-y-6 mb-12">
        {sections.map((section, index) => (
          <motion.div
            key={section.title}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.2 + index * 0.1 }}
          >
            <GlassCard>
              <div className="flex items-start gap-4">
                <GlowIcon icon={section.icon} size="md" color={luxuryColors.cyan} />
                <div className="flex-1">
                  <h3 className="text-lg font-bold mb-3" style={{ color: luxuryColors.textPrimary }}>
                    {section.title}
                  </h3>
                  <div
                    className="text-sm whitespace-pre-line"
                    style={{ color: luxuryColors.textSecondary }}
                  >
                    {section.content}
                  </div>
                </div>
              </div>
            </GlassCard>
          </motion.div>
        ))}
      </div>

      {/* Additional Terms */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.8 }}
        className="mb-12"
      >
        <GlassCard>
          <h3 className="text-lg font-bold mb-4" style={{ color: luxuryColors.textPrimary }}>
            Additional Legal Information
          </h3>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {[
              { title: "Governing Law", desc: "These terms are governed by California law." },
              { title: "Dispute Resolution", desc: "Disputes resolved through binding arbitration." },
              { title: "Limitation of Liability", desc: "Our liability is limited to fees paid." },
              { title: "Indemnification", desc: "Users indemnify us against third-party claims." },
            ].map((item) => (
              <div
                key={item.title}
                className="p-4 rounded-lg"
                style={{ backgroundColor: luxuryColors.bgSecondary }}
              >
                <h4 className="font-semibold mb-1" style={{ color: luxuryColors.textPrimary }}>
                  {item.title}
                </h4>
                <p className="text-xs" style={{ color: luxuryColors.textMuted }}>
                  {item.desc}
                </p>
              </div>
            ))}
          </div>
        </GlassCard>
      </motion.div>

      {/* Contact Section */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.9 }}
      >
        <GlassCard className="text-center">
          <h2 className="text-xl font-bold mb-4" style={{ color: luxuryColors.textPrimary }}>
            Questions About Terms?
          </h2>
          <p className="mb-6" style={{ color: luxuryColors.textMuted }}>
            If you have any questions about these Terms of Service, please contact our legal team.
          </p>
          <div className="flex justify-center gap-4">
            <Link to="/contact">
              <PremiumButton variant="primary">
                Contact Us
              </PremiumButton>
            </Link>
            <a href="mailto:legal@botcore.io">
              <PremiumButton variant="secondary">
                legal@botcore.io
              </PremiumButton>
            </a>
          </div>
        </GlassCard>
      </motion.div>
    </PageWrapper>
  );
};

export default Terms;
