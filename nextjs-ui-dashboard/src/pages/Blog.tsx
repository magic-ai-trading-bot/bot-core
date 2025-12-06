import { motion } from "framer-motion";
import { Link } from "react-router-dom";
import {
  Calendar,
  Clock,
  User,
  ArrowLeft,
  ChevronRight,
  TrendingUp,
  Brain,
  Shield,
  Zap,
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

const featuredPost = {
  title: "The Future of AI in Cryptocurrency Trading",
  excerpt: "Explore how machine learning models are revolutionizing the way we approach crypto markets, from sentiment analysis to predictive modeling.",
  image: "https://images.unsplash.com/photo-1639762681485-074b7f938ba0?w=800&h=400&fit=crop",
  author: "Alex Chen",
  date: "Dec 1, 2024",
  readTime: "8 min read",
  category: "AI & Machine Learning",
};

const posts = [
  {
    title: "Understanding RSI Strategy in Volatile Markets",
    excerpt: "A deep dive into how RSI-based trading strategies perform during high volatility periods.",
    icon: TrendingUp,
    author: "Sarah Kim",
    date: "Nov 28, 2024",
    readTime: "5 min read",
    category: "Trading Strategies",
  },
  {
    title: "How Our LSTM Model Predicts Price Movements",
    excerpt: "Behind the scenes look at the neural network architecture powering our AI predictions.",
    icon: Brain,
    author: "Michael Rodriguez",
    date: "Nov 25, 2024",
    readTime: "10 min read",
    category: "Technical Deep Dive",
  },
  {
    title: "Best Practices for API Key Security",
    excerpt: "Essential security measures every trader should implement when using trading APIs.",
    icon: Shield,
    author: "Emily Zhang",
    date: "Nov 22, 2024",
    readTime: "4 min read",
    category: "Security",
  },
  {
    title: "Optimizing Your Bot for Maximum Performance",
    excerpt: "Tips and tricks to get the most out of your automated trading strategies.",
    icon: Zap,
    author: "Alex Chen",
    date: "Nov 18, 2024",
    readTime: "6 min read",
    category: "Performance",
  },
];

const categories = [
  { name: "All Posts", count: 24 },
  { name: "Trading Strategies", count: 8 },
  { name: "AI & Machine Learning", count: 6 },
  { name: "Security", count: 4 },
  { name: "Product Updates", count: 6 },
];

const Blog = () => {
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
          Blog
        </Badge>
        <h1 className="text-4xl md:text-5xl font-black mb-4">
          <GradientText>Insights & Updates</GradientText>
        </h1>
        <p className="text-lg max-w-2xl mx-auto" style={{ color: luxuryColors.textMuted }}>
          Stay informed with the latest in algorithmic trading, AI developments, and market analysis.
        </p>
      </motion.div>

      {/* Categories */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.1 }}
        className="flex flex-wrap justify-center gap-2 mb-12"
      >
        {categories.map((category, index) => (
          <button
            key={category.name}
            className={`px-4 py-2 rounded-full text-sm transition-all ${
              index === 0
                ? 'bg-cyan-500/20 border border-cyan-500/50'
                : 'bg-white/5 hover:bg-white/10 border border-transparent'
            }`}
            style={{ color: index === 0 ? luxuryColors.cyan : luxuryColors.textSecondary }}
          >
            {category.name} ({category.count})
          </button>
        ))}
      </motion.div>

      {/* Featured Post */}
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
                src={featuredPost.image}
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
              <h2 className="text-2xl font-bold mb-3" style={{ color: luxuryColors.textPrimary }}>
                {featuredPost.title}
              </h2>
              <p className="mb-4" style={{ color: luxuryColors.textMuted }}>
                {featuredPost.excerpt}
              </p>
              <div className="flex items-center gap-4 mb-4">
                <div className="flex items-center gap-2">
                  <User className="w-4 h-4" style={{ color: luxuryColors.textMuted }} />
                  <span className="text-sm" style={{ color: luxuryColors.textSecondary }}>
                    {featuredPost.author}
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  <Calendar className="w-4 h-4" style={{ color: luxuryColors.textMuted }} />
                  <span className="text-sm" style={{ color: luxuryColors.textSecondary }}>
                    {featuredPost.date}
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  <Clock className="w-4 h-4" style={{ color: luxuryColors.textMuted }} />
                  <span className="text-sm" style={{ color: luxuryColors.textSecondary }}>
                    {featuredPost.readTime}
                  </span>
                </div>
              </div>
              <PremiumButton variant="primary" className="w-fit">
                Read Article
                <ChevronRight className="w-4 h-4" />
              </PremiumButton>
            </div>
          </div>
        </GlassCard>
      </motion.div>

      {/* Recent Posts */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.3 }}
        className="mb-12"
      >
        <h2 className="text-2xl font-bold mb-6" style={{ color: luxuryColors.textPrimary }}>
          Recent Posts
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {posts.map((post, index) => (
            <motion.div
              key={post.title}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.4 + index * 0.1 }}
            >
              <GlassCard className="h-full hover:border-cyan-500/30 transition-all cursor-pointer">
                <div className="flex items-start gap-4">
                  <GlowIcon icon={post.icon} size="md" color={luxuryColors.cyan} />
                  <div className="flex-1">
                    <Badge variant="default" size="sm" className="mb-2">
                      {post.category}
                    </Badge>
                    <h3 className="font-semibold mb-2" style={{ color: luxuryColors.textPrimary }}>
                      {post.title}
                    </h3>
                    <p className="text-sm mb-3" style={{ color: luxuryColors.textMuted }}>
                      {post.excerpt}
                    </p>
                    <div className="flex items-center gap-3 text-xs" style={{ color: luxuryColors.textMuted }}>
                      <span>{post.author}</span>
                      <span>•</span>
                      <span>{post.date}</span>
                      <span>•</span>
                      <span>{post.readTime}</span>
                    </div>
                  </div>
                </div>
              </GlassCard>
            </motion.div>
          ))}
        </div>
      </motion.div>

      {/* Newsletter CTA */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.6 }}
      >
        <GlassCard className="text-center">
          <h2 className="text-2xl font-bold mb-4" style={{ color: luxuryColors.textPrimary }}>
            Subscribe to Our Newsletter
          </h2>
          <p className="mb-6" style={{ color: luxuryColors.textMuted }}>
            Get weekly insights on trading strategies, market analysis, and product updates.
          </p>
          <div className="flex flex-col sm:flex-row gap-3 justify-center max-w-md mx-auto">
            <input
              type="email"
              placeholder="Enter your email"
              className="flex-1 px-4 py-2 rounded-lg border bg-transparent outline-none text-sm"
              style={{
                borderColor: luxuryColors.borderSubtle,
                color: luxuryColors.textPrimary,
              }}
            />
            <PremiumButton variant="primary">
              Subscribe
            </PremiumButton>
          </div>
        </GlassCard>
      </motion.div>
    </PageWrapper>
  );
};

export default Blog;
