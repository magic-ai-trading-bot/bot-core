import { motion } from "framer-motion";
import { Link } from "react-router-dom";
import {
  Shield,
  Eye,
  Lock,
  Database,
  Users,
  Globe,
  ArrowLeft,
  FileText,
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
    icon: Database,
    title: "Information We Collect",
    content: [
      "Account information (email, name, password hash)",
      "Trading preferences and strategy configurations",
      "Usage data and analytics (anonymized)",
      "Device information for security purposes",
      "API keys (encrypted, never stored in plain text)",
    ],
  },
  {
    icon: Eye,
    title: "How We Use Your Information",
    content: [
      "Provide and improve our trading services",
      "Process transactions and execute trades",
      "Send important notifications and alerts",
      "Analyze usage patterns to enhance features",
      "Comply with legal and regulatory requirements",
    ],
  },
  {
    icon: Lock,
    title: "Data Security",
    content: [
      "End-to-end encryption for all sensitive data",
      "AES-256 encryption for data at rest",
      "TLS 1.3 for all data in transit",
      "Regular security audits and penetration testing",
      "SOC 2 Type II certified infrastructure",
    ],
  },
  {
    icon: Users,
    title: "Data Sharing",
    content: [
      "We never sell your personal data to third parties",
      "Data shared only with exchanges you connect",
      "Service providers under strict confidentiality",
      "Legal authorities when required by law",
      "Aggregated, anonymized data for research",
    ],
  },
  {
    icon: Globe,
    title: "Your Rights",
    content: [
      "Access and download your personal data",
      "Request correction of inaccurate data",
      "Delete your account and associated data",
      "Opt-out of marketing communications",
      "Data portability to other services",
    ],
  },
];

const Privacy = () => {
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
          <GradientText>Privacy Policy</GradientText>
        </h1>
        <p className="text-lg max-w-2xl mx-auto mb-4" style={{ color: luxuryColors.textMuted }}>
          Your privacy is important to us. This policy explains how we collect, use, and protect your information.
        </p>
        <div className="flex items-center justify-center gap-2 text-sm" style={{ color: luxuryColors.textMuted }}>
          <FileText className="w-4 h-4" />
          <span>Last updated: December 1, 2024</span>
        </div>
      </motion.div>

      {/* Quick Summary */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.1 }}
        className="mb-12"
      >
        <GlassCard>
          <div className="flex items-start gap-4">
            <GlowIcon icon={Shield} size="lg" color={luxuryColors.profit} />
            <div>
              <h2 className="text-xl font-bold mb-2" style={{ color: luxuryColors.textPrimary }}>
                Privacy at a Glance
              </h2>
              <ul className="space-y-2 text-sm" style={{ color: luxuryColors.textSecondary }}>
                <li>• We collect only what's necessary to provide our services</li>
                <li>• Your data is encrypted with industry-standard protocols</li>
                <li>• We never sell your personal information to third parties</li>
                <li>• You have full control over your data and can delete it anytime</li>
                <li>• We comply with GDPR, CCPA, and other privacy regulations</li>
              </ul>
            </div>
          </div>
        </GlassCard>
      </motion.div>

      {/* Detailed Sections */}
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
                  <ul className="space-y-2">
                    {section.content.map((item, i) => (
                      <li key={i} className="flex items-start gap-2 text-sm" style={{ color: luxuryColors.textSecondary }}>
                        <span style={{ color: luxuryColors.cyan }}>•</span>
                        {item}
                      </li>
                    ))}
                  </ul>
                </div>
              </div>
            </GlassCard>
          </motion.div>
        ))}
      </div>

      {/* Cookies Section */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.7 }}
        className="mb-12"
      >
        <GlassCard>
          <h3 className="text-lg font-bold mb-4" style={{ color: luxuryColors.textPrimary }}>
            Cookies & Tracking
          </h3>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            {[
              { type: "Essential", description: "Required for basic functionality", required: true },
              { type: "Analytics", description: "Help us improve our service", required: false },
              { type: "Marketing", description: "Personalized content and ads", required: false },
            ].map((cookie) => (
              <div
                key={cookie.type}
                className="p-4 rounded-lg"
                style={{ backgroundColor: luxuryColors.bgSecondary }}
              >
                <div className="flex items-center justify-between mb-2">
                  <span className="font-semibold" style={{ color: luxuryColors.textPrimary }}>
                    {cookie.type}
                  </span>
                  <Badge variant={cookie.required ? "success" : "default"} size="sm">
                    {cookie.required ? "Required" : "Optional"}
                  </Badge>
                </div>
                <p className="text-xs" style={{ color: luxuryColors.textMuted }}>
                  {cookie.description}
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
        transition={{ delay: 0.8 }}
      >
        <GlassCard className="text-center">
          <h2 className="text-xl font-bold mb-4" style={{ color: luxuryColors.textPrimary }}>
            Questions About Privacy?
          </h2>
          <p className="mb-6" style={{ color: luxuryColors.textMuted }}>
            If you have any questions about this Privacy Policy, please contact our Data Protection Officer.
          </p>
          <div className="flex justify-center gap-4">
            <Link to="/contact">
              <PremiumButton variant="primary">
                Contact Us
              </PremiumButton>
            </Link>
            <a href="mailto:privacy@botcore.io">
              <PremiumButton variant="secondary">
                privacy@botcore.io
              </PremiumButton>
            </a>
          </div>
        </GlassCard>
      </motion.div>
    </PageWrapper>
  );
};

export default Privacy;
