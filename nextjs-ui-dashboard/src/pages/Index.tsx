import { motion } from 'framer-motion';
import {
  luxuryColors,
  GlassCard,
  GradientText,
  PremiumButton,
  Badge,
  GlowIcon,
  PageWrapper,
  containerVariants,
  itemVariants,
  StatCard,
} from '@/styles/luxury-design-system';
import {
  Sparkles,
  TrendingUp,
  Shield,
  Zap,
  Brain,
  BarChart3,
  CheckCircle2,
  ArrowRight,
  Users,
  DollarSign,
  Activity,
  Star,
  Quote,
  Menu,
  X,
} from 'lucide-react';
import { useState } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import ChatBot from '@/components/ChatBot';

const Index = () => {
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const navigate = useNavigate();

  return (
    <div style={{ backgroundColor: luxuryColors.bgPrimary, minHeight: '100vh' }}>
      {/* Header */}
      <motion.header
        className="sticky top-0 z-50 border-b backdrop-blur-xl"
        style={{
          backgroundColor: 'rgba(0, 0, 0, 0.8)',
          borderColor: luxuryColors.borderSubtle,
        }}
        initial={{ y: -100, opacity: 0 }}
        animate={{ y: 0, opacity: 1 }}
        transition={{ duration: 0.5 }}
      >
        <div className="container mx-auto px-4">
          <div className="flex items-center justify-between h-16">
            {/* Logo */}
            <div className="flex items-center gap-2">
              <div
                className="w-8 h-8 rounded-lg flex items-center justify-center"
                style={{
                  background: luxuryColors.gradientPremium,
                  boxShadow: luxuryColors.glowCyan,
                }}
              >
                <Sparkles className="w-5 h-5 text-white" />
              </div>
              <GradientText className="text-xl font-black">Bot Core</GradientText>
            </div>

            {/* Desktop Nav */}
            <nav className="hidden md:flex items-center gap-6">
              {['Features', 'Stats', 'Pricing', 'Testimonials', 'FAQ'].map((item) => (
                <a
                  key={item}
                  href={`#${item.toLowerCase()}`}
                  className="text-sm font-medium transition-colors hover:text-white"
                  style={{ color: luxuryColors.textSecondary }}
                >
                  {item}
                </a>
              ))}
            </nav>

            {/* CTA Buttons */}
            <div className="hidden md:flex items-center gap-4">
              <PremiumButton variant="ghost" size="md" onClick={() => navigate('/login')}>
                Sign In
              </PremiumButton>
              <PremiumButton variant="primary" size="md" onClick={() => navigate('/register')}>
                Get Started
              </PremiumButton>
            </div>

            {/* Mobile Menu Button */}
            <button
              className="md:hidden text-white"
              onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
            >
              {mobileMenuOpen ? <X /> : <Menu />}
            </button>
          </div>
        </div>

        {/* Mobile Menu */}
        {mobileMenuOpen && (
          <motion.div
            className="md:hidden border-t"
            style={{
              backgroundColor: 'rgba(0, 0, 0, 0.95)',
              borderColor: luxuryColors.borderSubtle,
            }}
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
          >
            <div className="px-4 py-4 space-y-3">
              {['Features', 'Stats', 'Pricing', 'Testimonials', 'FAQ'].map((item) => (
                <a
                  key={item}
                  href={`#${item.toLowerCase()}`}
                  className="block text-sm font-medium py-2"
                  style={{ color: luxuryColors.textSecondary }}
                  onClick={() => setMobileMenuOpen(false)}
                >
                  {item}
                </a>
              ))}
              <div className="pt-3 space-y-2">
                <PremiumButton variant="ghost" size="md" fullWidth onClick={() => { navigate('/login'); setMobileMenuOpen(false); }}>
                  Sign In
                </PremiumButton>
                <PremiumButton variant="primary" size="md" fullWidth onClick={() => { navigate('/register'); setMobileMenuOpen(false); }}>
                  Get Started
                </PremiumButton>
              </div>
            </div>
          </motion.div>
        )}
      </motion.header>

      <main>
        {/* Hero Section */}
        <section className="relative overflow-hidden">
          {/* Background Decorations */}
          <div className="absolute inset-0 overflow-hidden pointer-events-none">
            {/* Gradient Orbs */}
            <div
              className="absolute top-0 left-1/4 w-96 h-96 rounded-full blur-3xl opacity-20"
              style={{ background: luxuryColors.gradientCyan }}
            />
            <div
              className="absolute bottom-0 right-1/4 w-96 h-96 rounded-full blur-3xl opacity-20"
              style={{ background: luxuryColors.gradientProfit }}
            />
            {/* Grid Pattern */}
            <div
              className="absolute inset-0 opacity-[0.02]"
              style={{
                backgroundImage: `linear-gradient(${luxuryColors.borderSubtle} 1px, transparent 1px),
                                  linear-gradient(90deg, ${luxuryColors.borderSubtle} 1px, transparent 1px)`,
                backgroundSize: '50px 50px',
              }}
            />
          </div>

          <PageWrapper>
            <motion.div
              className="container mx-auto text-center py-20 md:py-32"
              variants={containerVariants}
            >
              {/* Badge */}
              <motion.div className="flex justify-center mb-6" variants={itemVariants}>
                <Badge variant="info" glow>
                  AI-Powered Trading Platform
                </Badge>
              </motion.div>

              {/* Hero Title */}
              <motion.h1
                className="text-4xl md:text-6xl lg:text-7xl font-black mb-6 leading-tight"
                variants={itemVariants}
              >
                Trade Smarter with{' '}
                <GradientText className="block">AI-Driven Insights</GradientText>
              </motion.h1>

              {/* Hero Description */}
              <motion.p
                className="text-lg md:text-xl max-w-2xl mx-auto mb-10"
                style={{ color: luxuryColors.textSecondary }}
                variants={itemVariants}
              >
                Advanced AI algorithms, real-time market analysis, and automated risk
                management. Your competitive edge in cryptocurrency trading.
              </motion.p>

              {/* CTA Buttons */}
              <motion.div
                className="flex flex-col sm:flex-row gap-4 justify-center items-center"
                variants={itemVariants}
              >
                <PremiumButton variant="primary" size="lg">
                  Start Trading Now
                  <ArrowRight className="w-5 h-5" />
                </PremiumButton>
                <PremiumButton variant="secondary" size="lg">
                  Watch Demo
                </PremiumButton>
              </motion.div>

              {/* Social Proof */}
              <motion.div
                className="flex flex-wrap justify-center items-center gap-8 mt-16"
                variants={itemVariants}
              >
                <div className="text-center">
                  <GradientText className="text-3xl font-black">10K+</GradientText>
                  <p className="text-xs mt-1" style={{ color: luxuryColors.textMuted }}>
                    Active Traders
                  </p>
                </div>
                <div
                  className="hidden sm:block w-px h-8"
                  style={{ backgroundColor: luxuryColors.borderSubtle }}
                />
                <div className="text-center">
                  <GradientText className="text-3xl font-black">$50M+</GradientText>
                  <p className="text-xs mt-1" style={{ color: luxuryColors.textMuted }}>
                    Trading Volume
                  </p>
                </div>
                <div
                  className="hidden sm:block w-px h-8"
                  style={{ backgroundColor: luxuryColors.borderSubtle }}
                />
                <div className="text-center">
                  <GradientText className="text-3xl font-black">99.9%</GradientText>
                  <p className="text-xs mt-1" style={{ color: luxuryColors.textMuted }}>
                    Uptime
                  </p>
                </div>
              </motion.div>
            </motion.div>
          </PageWrapper>
        </section>

        {/* Features Section */}
        <section id="features" className="relative">
          <PageWrapper>
            <motion.div
              className="container mx-auto py-20"
              initial="hidden"
              whileInView="visible"
              viewport={{ once: true }}
              variants={containerVariants}
            >
              {/* Section Header */}
              <motion.div className="text-center mb-16" variants={itemVariants}>
                <Badge variant="purple" className="mb-4">
                  Features
                </Badge>
                <h2 className="text-3xl md:text-5xl font-black mb-4">
                  <GradientText>Everything You Need</GradientText> to Succeed
                </h2>
                <p
                  className="text-lg max-w-2xl mx-auto"
                  style={{ color: luxuryColors.textSecondary }}
                >
                  Professional-grade tools designed for both beginners and experienced traders.
                </p>
              </motion.div>

              {/* Feature Cards */}
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {[
                  {
                    icon: Brain,
                    title: 'AI-Powered Analysis',
                    description: 'Advanced machine learning models analyze market trends and predict price movements with 70%+ accuracy.',
                    color: luxuryColors.purple,
                  },
                  {
                    icon: Shield,
                    title: 'Risk Management',
                    description: 'Automated stop-loss, position sizing, and portfolio rebalancing to protect your capital.',
                    color: luxuryColors.emerald,
                  },
                  {
                    icon: Zap,
                    title: 'Real-Time Execution',
                    description: 'Lightning-fast order execution with <100ms latency for optimal entry and exit points.',
                    color: luxuryColors.amber,
                  },
                  {
                    icon: BarChart3,
                    title: 'Advanced Analytics',
                    description: 'Comprehensive charts, technical indicators, and custom dashboards for data-driven decisions.',
                    color: luxuryColors.cyan,
                  },
                  {
                    icon: TrendingUp,
                    title: 'Strategy Backtesting',
                    description: 'Test your strategies against historical data before risking real capital.',
                    color: luxuryColors.rose,
                  },
                  {
                    icon: Activity,
                    title: 'Live Market Data',
                    description: 'Real-time market data feeds from multiple exchanges with WebSocket connectivity.',
                    color: luxuryColors.purple,
                  },
                ].map((feature, index) => (
                  <motion.div key={index} variants={itemVariants}>
                    <GlassCard hoverable glowColor={`0 8px 32px ${feature.color}30`}>
                      <GlowIcon icon={feature.icon} color={feature.color} size="lg" />
                      <h3 className="text-lg font-bold text-white mt-4 mb-2">
                        {feature.title}
                      </h3>
                      <p className="text-sm" style={{ color: luxuryColors.textSecondary }}>
                        {feature.description}
                      </p>
                    </GlassCard>
                  </motion.div>
                ))}
              </div>
            </motion.div>
          </PageWrapper>
        </section>

        {/* Stats Section */}
        <section id="stats" className="relative">
          <PageWrapper>
            <motion.div
              className="container mx-auto py-20"
              initial="hidden"
              whileInView="visible"
              viewport={{ once: true }}
              variants={containerVariants}
            >
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                <motion.div variants={itemVariants}>
                  <StatCard
                    label="Total Users"
                    value="10,234"
                    icon={Users}
                    trend={12.5}
                    trendLabel="vs last month"
                    iconColor={luxuryColors.cyan}
                    gradient
                  />
                </motion.div>
                <motion.div variants={itemVariants}>
                  <StatCard
                    label="Trading Volume"
                    value="$50.2M"
                    icon={DollarSign}
                    trend={8.3}
                    trendLabel="vs last month"
                    iconColor={luxuryColors.emerald}
                    gradient
                  />
                </motion.div>
                <motion.div variants={itemVariants}>
                  <StatCard
                    label="Win Rate"
                    value="65%"
                    icon={TrendingUp}
                    trend={2.1}
                    trendLabel="vs last month"
                    iconColor={luxuryColors.purple}
                    valueColor={luxuryColors.profit}
                  />
                </motion.div>
                <motion.div variants={itemVariants}>
                  <StatCard
                    label="Avg Response Time"
                    value="<100ms"
                    icon={Activity}
                    iconColor={luxuryColors.amber}
                    valueColor={luxuryColors.cyan}
                  />
                </motion.div>
              </div>
            </motion.div>
          </PageWrapper>
        </section>

        {/* Pricing Section */}
        <section id="pricing" className="relative">
          <PageWrapper>
            <motion.div
              className="container mx-auto py-20"
              initial="hidden"
              whileInView="visible"
              viewport={{ once: true }}
              variants={containerVariants}
            >
              {/* Section Header */}
              <motion.div className="text-center mb-16" variants={itemVariants}>
                <Badge variant="success" className="mb-4">
                  Pricing
                </Badge>
                <h2 className="text-3xl md:text-5xl font-black mb-4">
                  <GradientText>Simple, Transparent</GradientText> Pricing
                </h2>
                <p
                  className="text-lg max-w-2xl mx-auto"
                  style={{ color: luxuryColors.textSecondary }}
                >
                  Choose the plan that fits your trading needs. No hidden fees.
                </p>
              </motion.div>

              {/* Pricing Cards */}
              <div className="grid grid-cols-1 md:grid-cols-3 gap-6 max-w-6xl mx-auto">
                {[
                  {
                    name: 'Starter',
                    price: '$29',
                    period: '/month',
                    features: [
                      'Basic AI Trading Signals',
                      'Up to 5 Active Positions',
                      'Standard Support',
                      'Basic Analytics',
                    ],
                    variant: 'secondary' as const,
                  },
                  {
                    name: 'Pro',
                    price: '$99',
                    period: '/month',
                    badge: 'Most Popular',
                    features: [
                      'Advanced AI Trading Signals',
                      'Unlimited Active Positions',
                      'Priority Support',
                      'Advanced Analytics & Backtesting',
                      'Custom Strategies',
                      'API Access',
                    ],
                    variant: 'primary' as const,
                  },
                  {
                    name: 'Enterprise',
                    price: '$299',
                    period: '/month',
                    features: [
                      'Everything in Pro',
                      'Dedicated Account Manager',
                      'Custom AI Model Training',
                      'White-Label Solution',
                      'SLA Guarantee',
                    ],
                    variant: 'success' as const,
                  },
                ].map((plan, index) => (
                  <motion.div key={index} variants={itemVariants}>
                    <GlassCard hoverable noPadding>
                      <div className="p-6">
                        {plan.badge && (
                          <Badge variant="info" glow className="mb-4">
                            {plan.badge}
                          </Badge>
                        )}
                        <h3 className="text-xl font-bold text-white mb-2">{plan.name}</h3>
                        <div className="flex items-baseline mb-6">
                          <GradientText className="text-4xl font-black">
                            {plan.price}
                          </GradientText>
                          <span
                            className="text-sm ml-2"
                            style={{ color: luxuryColors.textMuted }}
                          >
                            {plan.period}
                          </span>
                        </div>
                        <ul className="space-y-3 mb-6">
                          {plan.features.map((feature, fIndex) => (
                            <li key={fIndex} className="flex items-start gap-2">
                              <CheckCircle2
                                className="w-4 h-4 mt-0.5 flex-shrink-0"
                                style={{ color: luxuryColors.emerald }}
                              />
                              <span
                                className="text-sm"
                                style={{ color: luxuryColors.textSecondary }}
                              >
                                {feature}
                              </span>
                            </li>
                          ))}
                        </ul>
                        <PremiumButton variant={plan.variant} fullWidth>
                          Get Started
                        </PremiumButton>
                      </div>
                    </GlassCard>
                  </motion.div>
                ))}
              </div>
            </motion.div>
          </PageWrapper>
        </section>

        {/* Testimonials Section */}
        <section id="testimonials" className="relative">
          <PageWrapper>
            <motion.div
              className="container mx-auto py-20"
              initial="hidden"
              whileInView="visible"
              viewport={{ once: true }}
              variants={containerVariants}
            >
              {/* Section Header */}
              <motion.div className="text-center mb-16" variants={itemVariants}>
                <Badge variant="warning" className="mb-4">
                  Testimonials
                </Badge>
                <h2 className="text-3xl md:text-5xl font-black mb-4">
                  Loved by <GradientText>Traders Worldwide</GradientText>
                </h2>
              </motion.div>

              {/* Testimonial Cards */}
              <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                {[
                  {
                    name: 'Sarah Chen',
                    role: 'Day Trader',
                    avatar: '👩‍💼',
                    rating: 5,
                    text: 'The AI signals are incredibly accurate. Increased my win rate from 45% to 68% in just 3 months!',
                  },
                  {
                    name: 'Michael Rodriguez',
                    role: 'Crypto Investor',
                    avatar: '👨‍💻',
                    rating: 5,
                    text: 'Best trading platform I have used. The risk management features saved me from major losses multiple times.',
                  },
                  {
                    name: 'Alex Thompson',
                    role: 'Algorithmic Trader',
                    avatar: '👨‍🔬',
                    rating: 5,
                    text: 'The API is robust and the backtesting features are top-notch. Perfect for testing strategies before going live.',
                  },
                ].map((testimonial, index) => (
                  <motion.div key={index} variants={itemVariants}>
                    <GlassCard hoverable>
                      <div className="flex items-center gap-3 mb-4">
                        <div className="text-3xl">{testimonial.avatar}</div>
                        <div>
                          <h4 className="text-sm font-bold text-white">
                            {testimonial.name}
                          </h4>
                          <p
                            className="text-xs"
                            style={{ color: luxuryColors.textMuted }}
                          >
                            {testimonial.role}
                          </p>
                        </div>
                      </div>
                      <div className="flex gap-1 mb-3">
                        {Array.from({ length: testimonial.rating }).map((_, i) => (
                          <Star
                            key={i}
                            className="w-4 h-4"
                            fill={luxuryColors.amber}
                            style={{ color: luxuryColors.amber }}
                          />
                        ))}
                      </div>
                      <Quote
                        className="w-6 h-6 mb-2 opacity-20"
                        style={{ color: luxuryColors.cyan }}
                      />
                      <p className="text-sm" style={{ color: luxuryColors.textSecondary }}>
                        {testimonial.text}
                      </p>
                    </GlassCard>
                  </motion.div>
                ))}
              </div>
            </motion.div>
          </PageWrapper>
        </section>

        {/* FAQ Section */}
        <section id="faq" className="relative">
          <PageWrapper>
            <motion.div
              className="container mx-auto py-20"
              initial="hidden"
              whileInView="visible"
              viewport={{ once: true }}
              variants={containerVariants}
            >
              {/* Section Header */}
              <motion.div className="text-center mb-16" variants={itemVariants}>
                <Badge variant="info" className="mb-4">
                  FAQ
                </Badge>
                <h2 className="text-3xl md:text-5xl font-black mb-4">
                  <GradientText>Frequently Asked</GradientText> Questions
                </h2>
              </motion.div>

              {/* FAQ Items */}
              <div className="max-w-3xl mx-auto space-y-4">
                {[
                  {
                    q: 'How accurate are the AI trading signals?',
                    a: 'Our AI models achieve 70%+ directional accuracy across multiple timeframes and market conditions. We continuously train and improve our models using the latest data.',
                  },
                  {
                    q: 'Is my capital safe?',
                    a: 'We never hold your funds. You maintain full custody of your assets on your exchange. We only execute trades through secure API connections with read and trade permissions.',
                  },
                  {
                    q: 'Can I use my own strategies?',
                    a: 'Absolutely! Pro and Enterprise plans include custom strategy support. You can backtest and deploy your own algorithms alongside our AI signals.',
                  },
                  {
                    q: 'What exchanges do you support?',
                    a: 'We currently support Binance, Coinbase Pro, Kraken, and Bybit. More exchanges are being added regularly based on user demand.',
                  },
                ].map((faq, index) => (
                  <motion.div key={index} variants={itemVariants}>
                    <GlassCard hoverable>
                      <h3 className="text-base font-bold text-white mb-2">{faq.q}</h3>
                      <p className="text-sm" style={{ color: luxuryColors.textSecondary }}>
                        {faq.a}
                      </p>
                    </GlassCard>
                  </motion.div>
                ))}
              </div>
            </motion.div>
          </PageWrapper>
        </section>

        {/* CTA Section */}
        <section className="relative">
          <PageWrapper>
            <motion.div
              className="container mx-auto py-20"
              initial="hidden"
              whileInView="visible"
              viewport={{ once: true }}
              variants={containerVariants}
            >
              <motion.div variants={itemVariants}>
                <GlassCard className="text-center">
                  <div className="max-w-3xl mx-auto py-8">
                    <h2 className="text-3xl md:text-5xl font-black mb-6">
                      Ready to <GradientText>Transform</GradientText> Your Trading?
                    </h2>
                    <p
                      className="text-lg mb-8"
                      style={{ color: luxuryColors.textSecondary }}
                    >
                      Join thousands of traders who are already using AI to gain a competitive edge in the market.
                    </p>
                    <div className="flex flex-col sm:flex-row gap-4 justify-center">
                      <PremiumButton variant="primary" size="lg">
                        Start Free Trial
                        <ArrowRight className="w-5 h-5" />
                      </PremiumButton>
                      <PremiumButton variant="secondary" size="lg">
                        Schedule Demo
                      </PremiumButton>
                    </div>
                  </div>
                </GlassCard>
              </motion.div>
            </motion.div>
          </PageWrapper>
        </section>
      </main>

      {/* Footer */}
      <footer
        className="border-t"
        style={{
          backgroundColor: luxuryColors.bgPrimary,
          borderColor: luxuryColors.borderSubtle,
        }}
      >
        <div className="container mx-auto px-4 py-12">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-8 mb-8">
            {/* Brand */}
            <div>
              <div className="flex items-center gap-2 mb-4">
                <div
                  className="w-8 h-8 rounded-lg flex items-center justify-center"
                  style={{
                    background: luxuryColors.gradientPremium,
                    boxShadow: luxuryColors.glowCyan,
                  }}
                >
                  <Sparkles className="w-5 h-5 text-white" />
                </div>
                <GradientText className="text-xl font-black">Bot Core</GradientText>
              </div>
              <p className="text-xs" style={{ color: luxuryColors.textMuted }}>
                AI-powered trading platform for the modern trader.
              </p>
            </div>

            {/* Links */}
            {[
              {
                title: 'Product',
                links: [
                  { label: 'Features', path: '/features' },
                  { label: 'Pricing', path: '/pricing' },
                  { label: 'API', path: '/api' },
                  { label: 'Documentation', path: '/docs' },
                ],
              },
              {
                title: 'Company',
                links: [
                  { label: 'About', path: '/about' },
                  { label: 'Blog', path: '/blog' },
                  { label: 'Careers', path: '/careers' },
                  { label: 'Contact', path: '/contact' },
                ],
              },
              {
                title: 'Legal',
                links: [
                  { label: 'Privacy', path: '/privacy' },
                  { label: 'Terms', path: '/terms' },
                  { label: 'Security', path: '/security' },
                  { label: 'Compliance', path: '/compliance' },
                ],
              },
            ].map((column, index) => (
              <div key={index}>
                <h4
                  className="text-sm font-bold text-white mb-3"
                  style={{ color: luxuryColors.textPrimary }}
                >
                  {column.title}
                </h4>
                <ul className="space-y-2">
                  {column.links.map((link, linkIndex) => (
                    <li key={linkIndex}>
                      <Link
                        to={link.path}
                        className="text-xs hover:text-white transition-colors"
                        style={{ color: luxuryColors.textMuted }}
                      >
                        {link.label}
                      </Link>
                    </li>
                  ))}
                </ul>
              </div>
            ))}
          </div>

          {/* Bottom Bar */}
          <div
            className="pt-8 border-t flex flex-col md:flex-row justify-between items-center gap-4"
            style={{ borderColor: luxuryColors.borderSubtle }}
          >
            <p className="text-xs" style={{ color: luxuryColors.textMuted }}>
              © 2025 Bot Core. All rights reserved.
            </p>
            <div className="flex gap-6">
              {[
                { name: 'Twitter', url: 'https://twitter.com/botcore' },
                { name: 'Discord', url: 'https://discord.gg/botcore' },
                { name: 'GitHub', url: 'https://github.com/botcore' },
              ].map((social) => (
                <a
                  key={social.name}
                  href={social.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-xs hover:text-white transition-colors"
                  style={{ color: luxuryColors.textMuted }}
                >
                  {social.name}
                </a>
              ))}
            </div>
          </div>
        </div>
      </footer>

      {/* Chatbot Widget */}
      <ChatBot />
    </div>
  );
};

export default Index;
