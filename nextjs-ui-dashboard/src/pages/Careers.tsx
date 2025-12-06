import { motion } from "framer-motion";
import { Link } from "react-router-dom";
import {
  Briefcase,
  MapPin,
  DollarSign,
  Clock,
  Heart,
  Sparkles,
  Coffee,
  GraduationCap,
  Plane,
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

const openPositions = [
  {
    title: "Senior Backend Engineer",
    department: "Engineering",
    location: "Remote",
    type: "Full-time",
    salary: "$150k - $200k",
    description: "Build high-performance trading systems using Rust and distributed architectures.",
  },
  {
    title: "Machine Learning Engineer",
    department: "AI Team",
    location: "San Francisco",
    type: "Full-time",
    salary: "$180k - $250k",
    description: "Develop and improve our AI models for market prediction and signal generation.",
  },
  {
    title: "Frontend Developer",
    department: "Engineering",
    location: "Remote",
    type: "Full-time",
    salary: "$130k - $170k",
    description: "Create beautiful, responsive trading dashboards using React and TypeScript.",
  },
  {
    title: "Product Designer",
    department: "Design",
    location: "New York",
    type: "Full-time",
    salary: "$120k - $160k",
    description: "Design intuitive user experiences for complex trading workflows.",
  },
  {
    title: "DevOps Engineer",
    department: "Infrastructure",
    location: "Remote",
    type: "Full-time",
    salary: "$140k - $180k",
    description: "Maintain and scale our cloud infrastructure for 24/7 trading operations.",
  },
];

const benefits = [
  {
    icon: DollarSign,
    title: "Competitive Salary",
    description: "Top-of-market compensation with equity options",
  },
  {
    icon: Heart,
    title: "Health & Wellness",
    description: "100% covered medical, dental, and vision insurance",
  },
  {
    icon: Plane,
    title: "Unlimited PTO",
    description: "Take the time you need to recharge and refresh",
  },
  {
    icon: Coffee,
    title: "Remote-First",
    description: "Work from anywhere with flexible hours",
  },
  {
    icon: GraduationCap,
    title: "Learning Budget",
    description: "$5,000 annual budget for courses and conferences",
  },
  {
    icon: Sparkles,
    title: "Latest Tech",
    description: "MacBook Pro, 4K monitor, and any tools you need",
  },
];

const Careers = () => {
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
          We're Hiring
        </Badge>
        <h1 className="text-4xl md:text-5xl font-black mb-4">
          <GradientText>Join Our Team</GradientText>
        </h1>
        <p className="text-lg max-w-2xl mx-auto" style={{ color: luxuryColors.textMuted }}>
          Help us build the future of automated trading. We're looking for passionate individuals who want to make a difference.
        </p>
      </motion.div>

      {/* Stats */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.1 }}
        className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-16"
      >
        {[
          { label: "Team Members", value: "45+" },
          { label: "Countries", value: "12" },
          { label: "Open Roles", value: "5" },
          { label: "Avg Tenure", value: "2.5 yrs" },
        ].map((stat) => (
          <GlassCard key={stat.label} noPadding className="p-4 text-center">
            <div className="text-2xl font-black mb-1" style={{ color: luxuryColors.cyan }}>
              {stat.value}
            </div>
            <div className="text-sm" style={{ color: luxuryColors.textMuted }}>
              {stat.label}
            </div>
          </GlassCard>
        ))}
      </motion.div>

      {/* Benefits */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.2 }}
        className="mb-16"
      >
        <h2 className="text-2xl font-bold text-center mb-8" style={{ color: luxuryColors.textPrimary }}>
          Why Work With Us
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {benefits.map((benefit, index) => (
            <motion.div
              key={benefit.title}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.3 + index * 0.05 }}
            >
              <GlassCard noPadding className="p-4 h-full">
                <div className="flex items-start gap-3">
                  <GlowIcon icon={benefit.icon} size="sm" color={luxuryColors.cyan} />
                  <div>
                    <h3 className="font-semibold mb-1" style={{ color: luxuryColors.textPrimary }}>
                      {benefit.title}
                    </h3>
                    <p className="text-sm" style={{ color: luxuryColors.textMuted }}>
                      {benefit.description}
                    </p>
                  </div>
                </div>
              </GlassCard>
            </motion.div>
          ))}
        </div>
      </motion.div>

      {/* Open Positions */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.4 }}
        className="mb-16"
      >
        <h2 className="text-2xl font-bold text-center mb-8" style={{ color: luxuryColors.textPrimary }}>
          Open Positions
        </h2>
        <div className="space-y-4">
          {openPositions.map((position, index) => (
            <motion.div
              key={position.title}
              initial={{ opacity: 0, x: -20 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ delay: 0.5 + index * 0.1 }}
            >
              <GlassCard className="hover:border-cyan-500/30 transition-all cursor-pointer">
                <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-4">
                  <div className="flex-1">
                    <div className="flex items-center gap-3 mb-2">
                      <GlowIcon icon={Briefcase} size="sm" color={luxuryColors.cyan} />
                      <h3 className="font-bold text-lg" style={{ color: luxuryColors.textPrimary }}>
                        {position.title}
                      </h3>
                    </div>
                    <p className="text-sm mb-3" style={{ color: luxuryColors.textMuted }}>
                      {position.description}
                    </p>
                    <div className="flex flex-wrap gap-2">
                      <Badge variant="default" size="sm">
                        <MapPin className="w-3 h-3 mr-1" />
                        {position.location}
                      </Badge>
                      <Badge variant="default" size="sm">
                        <Clock className="w-3 h-3 mr-1" />
                        {position.type}
                      </Badge>
                      <Badge variant="success" size="sm">
                        <DollarSign className="w-3 h-3 mr-1" />
                        {position.salary}
                      </Badge>
                      <Badge variant="info" size="sm">
                        {position.department}
                      </Badge>
                    </div>
                  </div>
                  <PremiumButton variant="primary" size="sm">
                    Apply Now
                    <ChevronRight className="w-4 h-4" />
                  </PremiumButton>
                </div>
              </GlassCard>
            </motion.div>
          ))}
        </div>
      </motion.div>

      {/* CTA */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.7 }}
      >
        <GlassCard className="text-center">
          <h2 className="text-2xl font-bold mb-4" style={{ color: luxuryColors.textPrimary }}>
            Don't See a Perfect Fit?
          </h2>
          <p className="mb-6" style={{ color: luxuryColors.textMuted }}>
            We're always looking for talented individuals. Send us your resume and we'll reach out when a suitable position opens up.
          </p>
          <Link to="/contact">
            <PremiumButton variant="primary">
              Send Your Resume
            </PremiumButton>
          </Link>
        </GlassCard>
      </motion.div>
    </PageWrapper>
  );
};

export default Careers;
