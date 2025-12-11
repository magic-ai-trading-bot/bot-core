import { motion } from "framer-motion";
import { Link } from "react-router-dom";
import { useTranslation } from "react-i18next";
import {
  Clock,
  ArrowLeft,
  ChevronRight,
  TrendingUp,
  Brain,
  Shield,
  Zap,
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

const postIcons = [TrendingUp, Brain, Shield, Zap];

const Blog = () => {
  const colors = useThemeColors();
  const { t } = useTranslation('pages');

  const posts = t('blog.posts', { returnObjects: true }) as Array<{
    title: string;
    excerpt: string;
    category: string;
    readTime: string;
  }>;

  const categoryKeys = ["all", "trading", "ai", "market", "tutorials"];

  // Featured post is the first one
  const featuredPost = posts[0];
  const recentPosts = posts.slice(1, 5);

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
        className="text-center mb-12"
      >
        <Badge variant="info" className="mb-4">
          {t('blog.badge')}
        </Badge>
        <h1 className="text-4xl md:text-5xl font-black mb-4">
          <GradientText>{t('blog.title')}</GradientText>
          <br />
          <span style={{ color: colors.textPrimary }}>{t('blog.subtitle')}</span>
        </h1>
        <p className="text-lg max-w-2xl mx-auto" style={{ color: colors.textMuted }}>
          {t('blog.description')}
        </p>
      </motion.div>

      {/* Categories */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.1 }}
        className="flex flex-wrap justify-center gap-2 mb-12"
      >
        {categoryKeys.map((key, index) => (
          <button
            key={key}
            className={`px-4 py-2 rounded-full text-sm transition-all ${
              index === 0
                ? 'bg-cyan-500/20 border border-cyan-500/50'
                : 'bg-white/5 hover:bg-white/10 border border-transparent'
            }`}
            style={{ color: index === 0 ? colors.cyan : colors.textSecondary }}
          >
            {t(`blog.categories.${key}`)}
          </button>
        ))}
      </motion.div>

      {/* Featured Post */}
      {featuredPost && (
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.2 }}
          className="mb-12"
        >
          <GlassCard className="overflow-hidden">
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              <div className="relative h-64 lg:h-auto">
                <img
                  src="https://images.unsplash.com/photo-1639762681485-074b7f938ba0?w=800&h=400&fit=crop"
                  alt={featuredPost.title}
                  className="w-full h-full object-cover rounded-lg"
                />
                <Badge variant="warning" className="absolute top-4 left-4">
                  Featured
                </Badge>
              </div>
              <div className="flex flex-col justify-center">
                <Badge variant="info" size="sm" className="w-fit mb-3">
                  {featuredPost.category}
                </Badge>
                <h2 className="text-2xl font-bold mb-3" style={{ color: colors.textPrimary }}>
                  {featuredPost.title}
                </h2>
                <p className="mb-4" style={{ color: colors.textMuted }}>
                  {featuredPost.excerpt}
                </p>
                <div className="flex items-center gap-4 mb-4">
                  <div className="flex items-center gap-2">
                    <Clock className="w-4 h-4" style={{ color: colors.textMuted }} />
                    <span className="text-sm" style={{ color: colors.textSecondary }}>
                      {featuredPost.readTime}
                    </span>
                  </div>
                </div>
                <PremiumButton variant="primary" className="w-fit">
                  {t('blog.readMore')}
                  <ChevronRight className="w-4 h-4" />
                </PremiumButton>
              </div>
            </div>
          </GlassCard>
        </motion.div>
      )}

      {/* Recent Posts */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.3 }}
        className="mb-12"
      >
        <h2 className="text-2xl font-bold mb-6" style={{ color: colors.textPrimary }}>
          {t('blog.viewAll')}
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {recentPosts.map((post, index) => {
            const Icon = postIcons[index % postIcons.length];
            return (
              <motion.div
                key={post.title}
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.4 + index * 0.1 }}
              >
                <GlassCard className="h-full hover:border-cyan-500/30 transition-all cursor-pointer">
                  <div className="flex items-start gap-4">
                    <GlowIcon icon={Icon} size="md" color={colors.cyan} />
                    <div className="flex-1">
                      <Badge variant="default" size="sm" className="mb-2">
                        {post.category}
                      </Badge>
                      <h3 className="font-semibold mb-2" style={{ color: colors.textPrimary }}>
                        {post.title}
                      </h3>
                      <p className="text-sm mb-3" style={{ color: colors.textMuted }}>
                        {post.excerpt}
                      </p>
                      <div className="flex items-center gap-3 text-xs" style={{ color: colors.textMuted }}>
                        <span>{post.readTime}</span>
                      </div>
                    </div>
                  </div>
                </GlassCard>
              </motion.div>
            );
          })}
        </div>
      </motion.div>
    </PageWrapper>
  );
};

export default Blog;
