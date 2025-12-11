import { motion } from "framer-motion";
import { Link } from "react-router-dom";
import { useTranslation } from "react-i18next";
import {
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
  GlassCard,
  GradientText,
  PremiumButton,
  Badge,
  GlowIcon,
  PageWrapper,
} from "@/styles/luxury-design-system";
import { useThemeColors } from "@/hooks/useThemeColors";

const teamImages = [
  "https://images.unsplash.com/photo-1507003211169-0a1dd7228f2d?w=200&h=200&fit=crop",
  "https://images.unsplash.com/photo-1494790108377-be9c29b29330?w=200&h=200&fit=crop",
  "https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?w=200&h=200&fit=crop",
  "https://images.unsplash.com/photo-1438761681033-6461ffad8d80?w=200&h=200&fit=crop",
];

const valueIcons = [Target, Heart, Lightbulb, Globe];
const valueKeys = ["missionDriven", "userFirst", "innovation", "globalImpact"];

const About = () => {
  const colors = useThemeColors();
  const { t } = useTranslation('pages');

  const stats = [
    { key: "activeUsers", value: "50,000+" },
    { key: "countries", value: "120+" },
    { key: "tradesExecuted", value: "10M+" },
    { key: "totalVolume", value: "$2B+" },
  ];

  const milestones = t('about.journey.milestones', { returnObjects: true }) as Array<{ year: string; event: string }>;
  const teamMembers = t('about.team.members', { returnObjects: true }) as Array<{ name: string; role: string; bio: string }>;

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
            {t('common.backToHome')}
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
          {t('about.badge')}
        </Badge>
        <h1 className="text-4xl md:text-5xl font-black mb-4">
          <GradientText>{t('about.title')}</GradientText>
          <br />
          <span style={{ color: colors.textPrimary }}>{t('about.subtitle')}</span>
        </h1>
        <p className="text-lg max-w-2xl mx-auto" style={{ color: colors.textMuted }}>
          {t('about.description')}
        </p>
      </motion.div>

      {/* Stats */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.1 }}
        className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-16"
      >
        {stats.map((stat) => (
          <GlassCard key={stat.key} noPadding className="p-4 text-center">
            <div className="text-2xl font-black mb-1" style={{ color: colors.cyan }}>
              {stat.value}
            </div>
            <div className="text-sm" style={{ color: colors.textMuted }}>
              {t(`about.stats.${stat.key}`)}
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
        <h2 className="text-2xl font-bold text-center mb-8" style={{ color: colors.textPrimary }}>
          {t('about.values.title')}
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          {valueKeys.map((key, index) => {
            const Icon = valueIcons[index];
            return (
              <motion.div
                key={key}
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.3 + index * 0.1 }}
              >
                <GlassCard noPadding className="p-4 text-center h-full">
                  <GlowIcon icon={Icon} size="md" color={colors.cyan} className="mx-auto mb-3" />
                  <h3 className="font-semibold mb-2" style={{ color: colors.textPrimary }}>
                    {t(`about.values.${key}.title`)}
                  </h3>
                  <p className="text-sm" style={{ color: colors.textMuted }}>
                    {t(`about.values.${key}.description`)}
                  </p>
                </GlassCard>
              </motion.div>
            );
          })}
        </div>
      </motion.div>

      {/* Timeline */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.4 }}
        className="mb-16"
      >
        <h2 className="text-2xl font-bold text-center mb-8" style={{ color: colors.textPrimary }}>
          {t('about.journey.title')}
        </h2>
        <div className="flex flex-wrap justify-center gap-4">
          {milestones.map((milestone) => (
            <GlassCard key={milestone.year} noPadding className="p-4 min-w-[200px]">
              <Badge variant="info" size="sm" className="mb-2">{milestone.year}</Badge>
              <p className="text-sm" style={{ color: colors.textSecondary }}>
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
        <h2 className="text-2xl font-bold text-center mb-8" style={{ color: colors.textPrimary }}>
          {t('about.team.title')}
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          {teamMembers.map((member, index) => (
            <motion.div
              key={member.name}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.6 + index * 0.1 }}
            >
              <GlassCard className="text-center">
                <img
                  src={teamImages[index] || teamImages[0]}
                  alt={member.name}
                  className="w-20 h-20 rounded-full mx-auto mb-4 object-cover border-2"
                  style={{ borderColor: colors.cyan }}
                />
                <h3 className="font-semibold" style={{ color: colors.textPrimary }}>
                  {member.name}
                </h3>
                <p className="text-sm mb-2" style={{ color: colors.cyan }}>
                  {member.role}
                </p>
                <p className="text-xs mb-3" style={{ color: colors.textMuted }}>
                  {member.bio}
                </p>
                <div className="flex justify-center gap-2">
                  <button className="p-2 rounded-full hover:bg-white/10 transition-colors">
                    <Linkedin className="w-4 h-4" style={{ color: colors.textMuted }} />
                  </button>
                  <button className="p-2 rounded-full hover:bg-white/10 transition-colors">
                    <Twitter className="w-4 h-4" style={{ color: colors.textMuted }} />
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
          <GlowIcon icon={Award} size="lg" color={colors.cyan} className="mx-auto mb-4" />
          <h2 className="text-2xl font-bold mb-4" style={{ color: colors.textPrimary }}>
            {t('about.cta.title')}
          </h2>
          <p className="mb-6" style={{ color: colors.textMuted }}>
            {t('about.cta.description')}
          </p>
          <div className="flex justify-center gap-4">
            <Link to="/careers">
              <PremiumButton variant="primary">
                {t('about.cta.viewPositions')}
              </PremiumButton>
            </Link>
            <Link to="/contact">
              <PremiumButton variant="secondary">
                {t('about.cta.getInTouch')}
              </PremiumButton>
            </Link>
          </div>
        </GlassCard>
      </motion.div>
    </PageWrapper>
  );
};

export default About;
