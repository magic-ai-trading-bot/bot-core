import { motion } from "framer-motion";
import { Link } from "react-router-dom";
import {
  Users,
  Target,
  Heart,
  Globe,
  Award,
  Lightbulb,
  ArrowLeft,
  Linkedin,
  Twitter,
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

const team = [
  {
    name: "Alex Chen",
    role: "CEO & Co-Founder",
    bio: "Former quantitative trader at Goldman Sachs with 10+ years in algorithmic trading.",
    image: "https://images.unsplash.com/photo-1507003211169-0a1dd7228f2d?w=200&h=200&fit=crop",
  },
  {
    name: "Sarah Kim",
    role: "CTO & Co-Founder",
    bio: "Ex-Google engineer specializing in machine learning and distributed systems.",
    image: "https://images.unsplash.com/photo-1494790108377-be9c29b29330?w=200&h=200&fit=crop",
  },
  {
    name: "Michael Rodriguez",
    role: "Head of AI",
    bio: "PhD in Machine Learning from MIT, pioneered neural network trading strategies.",
    image: "https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?w=200&h=200&fit=crop",
  },
  {
    name: "Emily Zhang",
    role: "Head of Product",
    bio: "Former product lead at Coinbase, passionate about user-centric design.",
    image: "https://images.unsplash.com/photo-1438761681033-6461ffad8d80?w=200&h=200&fit=crop",
  },
];

const values = [
  {
    icon: Target,
    title: "Mission-Driven",
    description: "We're committed to democratizing algorithmic trading for everyone.",
  },
  {
    icon: Heart,
    title: "User-First",
    description: "Every feature we build starts with understanding our users' needs.",
  },
  {
    icon: Lightbulb,
    title: "Innovation",
    description: "We push the boundaries of what's possible with AI and trading.",
  },
  {
    icon: Globe,
    title: "Global Impact",
    description: "Building tools that empower traders worldwide, regardless of background.",
  },
];

const milestones = [
  { year: "2021", event: "Founded in San Francisco" },
  { year: "2022", event: "Launched beta with 1,000 users" },
  { year: "2023", event: "Series A funding, 50,000+ users" },
  { year: "2024", event: "AI trading engine v2.0 released" },
];

const About = () => {
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
        <Badge variant="info" className="mb-4">
          About Us
        </Badge>
        <h1 className="text-4xl md:text-5xl font-black mb-4">
          <GradientText>Building the Future of</GradientText>
          <br />
          <span style={{ color: luxuryColors.textPrimary }}>Automated Trading</span>
        </h1>
        <p className="text-lg max-w-2xl mx-auto" style={{ color: luxuryColors.textMuted }}>
          We believe everyone deserves access to institutional-grade trading technology. Our mission is to democratize algorithmic trading through AI.
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
          { label: "Active Users", value: "50,000+" },
          { label: "Countries", value: "120+" },
          { label: "Trades Executed", value: "10M+" },
          { label: "Total Volume", value: "$2B+" },
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

      {/* Our Values */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.2 }}
        className="mb-16"
      >
        <h2 className="text-2xl font-bold text-center mb-8" style={{ color: luxuryColors.textPrimary }}>
          Our Values
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          {values.map((value, index) => (
            <motion.div
              key={value.title}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.3 + index * 0.1 }}
            >
              <GlassCard noPadding className="p-4 text-center h-full">
                <GlowIcon icon={value.icon} size="md" color={luxuryColors.cyan} className="mx-auto mb-3" />
                <h3 className="font-semibold mb-2" style={{ color: luxuryColors.textPrimary }}>
                  {value.title}
                </h3>
                <p className="text-sm" style={{ color: luxuryColors.textMuted }}>
                  {value.description}
                </p>
              </GlassCard>
            </motion.div>
          ))}
        </div>
      </motion.div>

      {/* Timeline */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.4 }}
        className="mb-16"
      >
        <h2 className="text-2xl font-bold text-center mb-8" style={{ color: luxuryColors.textPrimary }}>
          Our Journey
        </h2>
        <div className="flex flex-wrap justify-center gap-4">
          {milestones.map((milestone, index) => (
            <GlassCard key={milestone.year} noPadding className="p-4 min-w-[200px]">
              <Badge variant="info" size="sm" className="mb-2">{milestone.year}</Badge>
              <p className="text-sm" style={{ color: luxuryColors.textSecondary }}>
                {milestone.event}
              </p>
            </GlassCard>
          ))}
        </div>
      </motion.div>

      {/* Team */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.5 }}
        className="mb-16"
      >
        <h2 className="text-2xl font-bold text-center mb-8" style={{ color: luxuryColors.textPrimary }}>
          Meet Our Team
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          {team.map((member, index) => (
            <motion.div
              key={member.name}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.6 + index * 0.1 }}
            >
              <GlassCard className="text-center">
                <img
                  src={member.image}
                  alt={member.name}
                  className="w-20 h-20 rounded-full mx-auto mb-4 object-cover border-2"
                  style={{ borderColor: luxuryColors.cyan }}
                />
                <h3 className="font-semibold" style={{ color: luxuryColors.textPrimary }}>
                  {member.name}
                </h3>
                <p className="text-sm mb-2" style={{ color: luxuryColors.cyan }}>
                  {member.role}
                </p>
                <p className="text-xs mb-3" style={{ color: luxuryColors.textMuted }}>
                  {member.bio}
                </p>
                <div className="flex justify-center gap-2">
                  <button className="p-2 rounded-full hover:bg-white/10 transition-colors">
                    <Linkedin className="w-4 h-4" style={{ color: luxuryColors.textMuted }} />
                  </button>
                  <button className="p-2 rounded-full hover:bg-white/10 transition-colors">
                    <Twitter className="w-4 h-4" style={{ color: luxuryColors.textMuted }} />
                  </button>
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
        transition={{ delay: 0.8 }}
      >
        <GlassCard className="text-center">
          <GlowIcon icon={Award} size="lg" color={luxuryColors.cyan} className="mx-auto mb-4" />
          <h2 className="text-2xl font-bold mb-4" style={{ color: luxuryColors.textPrimary }}>
            Join Our Mission
          </h2>
          <p className="mb-6" style={{ color: luxuryColors.textMuted }}>
            We're always looking for talented individuals who share our passion for innovation.
          </p>
          <div className="flex justify-center gap-4">
            <Link to="/careers">
              <PremiumButton variant="primary">
                View Open Positions
              </PremiumButton>
            </Link>
            <Link to="/contact">
              <PremiumButton variant="secondary">
                Get in Touch
              </PremiumButton>
            </Link>
          </div>
        </GlassCard>
      </motion.div>
    </PageWrapper>
  );
};

export default About;
