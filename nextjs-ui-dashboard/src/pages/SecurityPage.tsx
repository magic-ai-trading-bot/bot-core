import { motion } from "framer-motion";
import { Link } from "react-router-dom";
import {
  Shield,
  Lock,
  Key,
  Server,
  Eye,
  AlertTriangle,
  CheckCircle,
  ArrowLeft,
  FileText,
  Bug,
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

const securityFeatures = [
  {
    icon: Lock,
    title: "Encryption",
    description: "All data encrypted with AES-256 at rest and TLS 1.3 in transit",
    status: "Active",
  },
  {
    icon: Key,
    title: "Authentication",
    description: "RS256 JWT tokens with secure refresh mechanism",
    status: "Active",
  },
  {
    icon: Shield,
    title: "2FA Support",
    description: "Two-factor authentication via authenticator apps",
    status: "Active",
  },
  {
    icon: Server,
    title: "Infrastructure",
    description: "SOC 2 Type II certified cloud infrastructure",
    status: "Certified",
  },
  {
    icon: Eye,
    title: "Monitoring",
    description: "24/7 security monitoring and anomaly detection",
    status: "Active",
  },
  {
    icon: AlertTriangle,
    title: "Incident Response",
    description: "Dedicated security team with rapid response protocols",
    status: "Ready",
  },
];

const securityPractices = [
  {
    category: "API Key Security",
    items: [
      "API keys encrypted with RSA-2048 before storage",
      "Keys can be restricted by IP address",
      "Read-only keys available for monitoring",
      "Instant key revocation capability",
    ],
  },
  {
    category: "Account Protection",
    items: [
      "Secure password hashing with bcrypt",
      "Account lockout after failed attempts",
      "Session management with secure tokens",
      "Email verification required",
    ],
  },
  {
    category: "Infrastructure Security",
    items: [
      "DDoS protection and rate limiting",
      "Regular penetration testing",
      "Automated vulnerability scanning",
      "Isolated production environments",
    ],
  },
];

const certifications = [
  { name: "SOC 2 Type II", status: "Certified" },
  { name: "GDPR Compliant", status: "Compliant" },
  { name: "CCPA Compliant", status: "Compliant" },
  { name: "PCI DSS", status: "In Progress" },
];

const SecurityPage = () => {
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
        <Badge variant="success" className="mb-4">
          Security
        </Badge>
        <h1 className="text-4xl md:text-5xl font-black mb-4">
          <GradientText>Security First</GradientText>
        </h1>
        <p className="text-lg max-w-2xl mx-auto" style={{ color: luxuryColors.textMuted }}>
          Your security is our top priority. Learn how we protect your data and assets.
        </p>
      </motion.div>

      {/* Security Score */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.1 }}
        className="mb-12"
      >
        <GlassCard className="text-center">
          <div className="flex items-center justify-center gap-4 mb-4">
            <GlowIcon icon={Shield} size="lg" color={luxuryColors.profit} />
            <div>
              <div className="text-5xl font-black" style={{ color: luxuryColors.profit }}>
                98/100
              </div>
              <div className="text-sm" style={{ color: luxuryColors.textMuted }}>
                Security Score
              </div>
            </div>
          </div>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mt-6">
            {certifications.map((cert) => (
              <div key={cert.name} className="text-center">
                <Badge variant={cert.status === "Certified" || cert.status === "Compliant" ? "success" : "warning"} size="sm">
                  {cert.status}
                </Badge>
                <p className="text-sm mt-1" style={{ color: luxuryColors.textSecondary }}>
                  {cert.name}
                </p>
              </div>
            ))}
          </div>
        </GlassCard>
      </motion.div>

      {/* Security Features */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.2 }}
        className="mb-12"
      >
        <h2 className="text-2xl font-bold text-center mb-8" style={{ color: luxuryColors.textPrimary }}>
          Security Features
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {securityFeatures.map((feature, index) => (
            <motion.div
              key={feature.title}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.3 + index * 0.05 }}
            >
              <GlassCard noPadding className="p-4 h-full">
                <div className="flex items-start justify-between mb-3">
                  <GlowIcon icon={feature.icon} size="md" color={luxuryColors.cyan} />
                  <Badge variant="success" size="sm">{feature.status}</Badge>
                </div>
                <h3 className="font-semibold mb-1" style={{ color: luxuryColors.textPrimary }}>
                  {feature.title}
                </h3>
                <p className="text-sm" style={{ color: luxuryColors.textMuted }}>
                  {feature.description}
                </p>
              </GlassCard>
            </motion.div>
          ))}
        </div>
      </motion.div>

      {/* Security Practices */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.4 }}
        className="mb-12"
      >
        <h2 className="text-2xl font-bold text-center mb-8" style={{ color: luxuryColors.textPrimary }}>
          Our Security Practices
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {securityPractices.map((practice, index) => (
            <GlassCard key={practice.category}>
              <h3 className="font-bold mb-4" style={{ color: luxuryColors.cyan }}>
                {practice.category}
              </h3>
              <ul className="space-y-2">
                {practice.items.map((item, i) => (
                  <li key={i} className="flex items-start gap-2 text-sm" style={{ color: luxuryColors.textSecondary }}>
                    <CheckCircle className="w-4 h-4 mt-0.5 flex-shrink-0" style={{ color: luxuryColors.profit }} />
                    {item}
                  </li>
                ))}
              </ul>
            </GlassCard>
          ))}
        </div>
      </motion.div>

      {/* Bug Bounty */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.5 }}
        className="mb-12"
      >
        <GlassCard>
          <div className="flex items-start gap-4">
            <GlowIcon icon={Bug} size="lg" color={luxuryColors.warning} />
            <div className="flex-1">
              <h3 className="text-lg font-bold mb-2" style={{ color: luxuryColors.textPrimary }}>
                Bug Bounty Program
              </h3>
              <p className="text-sm mb-4" style={{ color: luxuryColors.textMuted }}>
                We reward security researchers who help us keep our platform secure. Report vulnerabilities responsibly and earn rewards up to $10,000.
              </p>
              <div className="flex flex-wrap gap-4">
                <div>
                  <div className="text-xl font-bold" style={{ color: luxuryColors.profit }}>$500 - $10,000</div>
                  <div className="text-xs" style={{ color: luxuryColors.textMuted }}>Bounty Range</div>
                </div>
                <div>
                  <div className="text-xl font-bold" style={{ color: luxuryColors.cyan }}>24 hrs</div>
                  <div className="text-xs" style={{ color: luxuryColors.textMuted }}>Response Time</div>
                </div>
                <div>
                  <div className="text-xl font-bold" style={{ color: luxuryColors.textPrimary }}>45+</div>
                  <div className="text-xs" style={{ color: luxuryColors.textMuted }}>Reports Resolved</div>
                </div>
              </div>
            </div>
          </div>
        </GlassCard>
      </motion.div>

      {/* Contact Section */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.6 }}
      >
        <GlassCard className="text-center">
          <GlowIcon icon={FileText} size="lg" color={luxuryColors.cyan} className="mx-auto mb-4" />
          <h2 className="text-xl font-bold mb-4" style={{ color: luxuryColors.textPrimary }}>
            Security Questions?
          </h2>
          <p className="mb-6" style={{ color: luxuryColors.textMuted }}>
            For security inquiries or to report a vulnerability, contact our security team.
          </p>
          <div className="flex justify-center gap-4">
            <a href="mailto:security@botcore.io">
              <PremiumButton variant="primary">
                security@botcore.io
              </PremiumButton>
            </a>
            <Link to="/docs">
              <PremiumButton variant="secondary">
                Security Docs
              </PremiumButton>
            </Link>
          </div>
        </GlassCard>
      </motion.div>
    </PageWrapper>
  );
};

export default SecurityPage;
