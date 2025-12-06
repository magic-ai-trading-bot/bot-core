import { motion } from "framer-motion";
import { Link } from "react-router-dom";
import {
  Scale,
  Globe,
  FileCheck,
  Building,
  Shield,
  AlertCircle,
  CheckCircle,
  ArrowLeft,
  Download,
  ExternalLink,
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

const regulations = [
  {
    name: "GDPR",
    fullName: "General Data Protection Regulation",
    region: "European Union",
    status: "Compliant",
    description: "Full compliance with EU data protection requirements including data subject rights, consent management, and data processing agreements.",
  },
  {
    name: "CCPA",
    fullName: "California Consumer Privacy Act",
    region: "California, USA",
    status: "Compliant",
    description: "Compliance with California privacy laws including consumer rights to access, delete, and opt-out of data sales.",
  },
  {
    name: "MiCA",
    fullName: "Markets in Crypto-Assets Regulation",
    region: "European Union",
    status: "Preparing",
    description: "Preparing for full compliance with EU crypto-asset regulations as they come into effect.",
  },
  {
    name: "FinCEN",
    fullName: "Financial Crimes Enforcement Network",
    region: "United States",
    status: "Compliant",
    description: "Registered Money Services Business (MSB) complying with US anti-money laundering requirements.",
  },
];

const certifications = [
  {
    icon: Shield,
    name: "SOC 2 Type II",
    description: "Independent audit of security, availability, and confidentiality controls",
    date: "Certified August 2024",
  },
  {
    icon: FileCheck,
    name: "ISO 27001",
    description: "Information security management system certification",
    date: "In Progress",
  },
  {
    icon: Building,
    name: "MSB Registration",
    description: "Registered Money Services Business with FinCEN",
    date: "Registration #31000231456789",
  },
];

const complianceAreas = [
  {
    title: "Anti-Money Laundering (AML)",
    items: [
      "Customer identification procedures (KYC)",
      "Transaction monitoring systems",
      "Suspicious activity reporting",
      "Sanctions screening (OFAC, UN, EU)",
    ],
  },
  {
    title: "Data Protection",
    items: [
      "Data minimization practices",
      "User consent management",
      "Data retention policies",
      "Cross-border transfer safeguards",
    ],
  },
  {
    title: "Financial Compliance",
    items: [
      "Trading activity monitoring",
      "Market manipulation prevention",
      "Fair trading practices",
      "Customer fund segregation",
    ],
  },
];

const Compliance = () => {
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
          <GradientText>Compliance</GradientText>
        </h1>
        <p className="text-lg max-w-2xl mx-auto" style={{ color: luxuryColors.textMuted }}>
          We're committed to operating within regulatory frameworks and maintaining the highest compliance standards.
        </p>
      </motion.div>

      {/* Compliance Overview */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.1 }}
        className="mb-12"
      >
        <GlassCard>
          <div className="flex items-start gap-4">
            <GlowIcon icon={Scale} size="lg" color={luxuryColors.cyan} />
            <div>
              <h2 className="text-xl font-bold mb-2" style={{ color: luxuryColors.textPrimary }}>
                Our Commitment to Compliance
              </h2>
              <p className="text-sm" style={{ color: luxuryColors.textSecondary }}>
                Bot Core operates as a regulated financial technology company. We maintain robust compliance programs to protect our users and ensure we operate within all applicable laws and regulations. Our compliance team continuously monitors regulatory developments across all jurisdictions where we operate.
              </p>
            </div>
          </div>
        </GlassCard>
      </motion.div>

      {/* Certifications */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.2 }}
        className="mb-12"
      >
        <h2 className="text-2xl font-bold text-center mb-8" style={{ color: luxuryColors.textPrimary }}>
          Certifications & Registrations
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {certifications.map((cert, index) => (
            <motion.div
              key={cert.name}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.3 + index * 0.1 }}
            >
              <GlassCard className="h-full text-center">
                <GlowIcon icon={cert.icon} size="lg" color={luxuryColors.cyan} className="mx-auto mb-4" />
                <h3 className="font-bold mb-2" style={{ color: luxuryColors.textPrimary }}>
                  {cert.name}
                </h3>
                <p className="text-sm mb-3" style={{ color: luxuryColors.textMuted }}>
                  {cert.description}
                </p>
                <Badge variant={cert.date.includes("Progress") ? "warning" : "success"} size="sm">
                  {cert.date}
                </Badge>
              </GlassCard>
            </motion.div>
          ))}
        </div>
      </motion.div>

      {/* Regulatory Status */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.4 }}
        className="mb-12"
      >
        <h2 className="text-2xl font-bold text-center mb-8" style={{ color: luxuryColors.textPrimary }}>
          Regulatory Compliance
        </h2>
        <div className="space-y-4">
          {regulations.map((reg, index) => (
            <motion.div
              key={reg.name}
              initial={{ opacity: 0, x: -20 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ delay: 0.5 + index * 0.1 }}
            >
              <GlassCard>
                <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
                  <div className="flex items-start gap-4">
                    <GlowIcon icon={Globe} size="md" color={luxuryColors.cyan} />
                    <div>
                      <div className="flex items-center gap-2 mb-1">
                        <h3 className="font-bold" style={{ color: luxuryColors.textPrimary }}>
                          {reg.name}
                        </h3>
                        <Badge variant="default" size="sm">{reg.region}</Badge>
                      </div>
                      <p className="text-sm mb-1" style={{ color: luxuryColors.cyan }}>
                        {reg.fullName}
                      </p>
                      <p className="text-sm" style={{ color: luxuryColors.textMuted }}>
                        {reg.description}
                      </p>
                    </div>
                  </div>
                  <Badge
                    variant={reg.status === "Compliant" ? "success" : "warning"}
                    className="whitespace-nowrap"
                  >
                    {reg.status === "Compliant" ? (
                      <CheckCircle className="w-3 h-3 mr-1" />
                    ) : (
                      <AlertCircle className="w-3 h-3 mr-1" />
                    )}
                    {reg.status}
                  </Badge>
                </div>
              </GlassCard>
            </motion.div>
          ))}
        </div>
      </motion.div>

      {/* Compliance Areas */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.6 }}
        className="mb-12"
      >
        <h2 className="text-2xl font-bold text-center mb-8" style={{ color: luxuryColors.textPrimary }}>
          Compliance Programs
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {complianceAreas.map((area) => (
            <GlassCard key={area.title}>
              <h3 className="font-bold mb-4" style={{ color: luxuryColors.cyan }}>
                {area.title}
              </h3>
              <ul className="space-y-2">
                {area.items.map((item, i) => (
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

      {/* Downloads */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.7 }}
        className="mb-12"
      >
        <GlassCard>
          <h3 className="text-lg font-bold mb-4" style={{ color: luxuryColors.textPrimary }}>
            Compliance Documents
          </h3>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            {[
              { name: "Privacy Policy", type: "PDF" },
              { name: "Terms of Service", type: "PDF" },
              { name: "SOC 2 Report Summary", type: "PDF" },
              { name: "Data Processing Agreement", type: "PDF" },
            ].map((doc) => (
              <button
                key={doc.name}
                className="flex items-center justify-between p-3 rounded-lg border transition-all hover:border-cyan-500/30"
                style={{ borderColor: luxuryColors.borderSubtle }}
              >
                <div className="flex items-center gap-2">
                  <Download className="w-4 h-4" style={{ color: luxuryColors.cyan }} />
                  <span className="text-sm" style={{ color: luxuryColors.textSecondary }}>
                    {doc.name}
                  </span>
                </div>
                <Badge variant="default" size="sm">{doc.type}</Badge>
              </button>
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
            Compliance Inquiries
          </h2>
          <p className="mb-6" style={{ color: luxuryColors.textMuted }}>
            For regulatory or compliance-related questions, please contact our compliance team.
          </p>
          <div className="flex justify-center gap-4">
            <a href="mailto:compliance@botcore.io">
              <PremiumButton variant="primary">
                compliance@botcore.io
              </PremiumButton>
            </a>
            <Link to="/contact">
              <PremiumButton variant="secondary">
                Contact Form
                <ExternalLink className="w-4 h-4" />
              </PremiumButton>
            </Link>
          </div>
        </GlassCard>
      </motion.div>
    </PageWrapper>
  );
};

export default Compliance;
