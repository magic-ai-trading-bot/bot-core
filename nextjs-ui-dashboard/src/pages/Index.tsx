import { motion } from 'framer-motion';
import {
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
import { useThemeColors } from '@/hooks/useThemeColors';
import {
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
  PlayCircle,
} from 'lucide-react';
import { BotCoreLogo } from '@/components/BotCoreLogo';
import { useState } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import ChatBot from '@/components/ChatBot';
import { LanguageSelector } from '@/components/LanguageSelector';
import { ThemeToggle } from '@/components/ThemeToggle';
import { lazy, Suspense } from 'react';

// Lazy load 3D component for performance
const HeroScene3D = lazy(() => import('@/components/3d/HeroScene3D'));

const Index = () => {
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const [videoModalOpen, setVideoModalOpen] = useState(false);
  const navigate = useNavigate();
  const { t } = useTranslation('landing');
  const colors = useThemeColors();

  return (
    <div style={{ backgroundColor: colors.bgPrimary, minHeight: '100vh' }}>
      {/* Header */}
      <motion.header
        className="sticky top-0 z-50 border-b backdrop-blur-xl"
        style={{
          backgroundColor: colors.bgHeader,
          borderColor: colors.borderSubtle,
        }}
        initial={{ y: -100, opacity: 0 }}
        animate={{ y: 0, opacity: 1 }}
        transition={{ duration: 0.5 }}
      >
        <div className="container mx-auto px-4">
          <div className="flex items-center h-16">
            {/* Logo - Left section (flex-1 to balance with right) */}
            <div className="flex-1 flex items-center">
              <BotCoreLogo size="md" />
            </div>

            {/* Desktop Nav - Centered (spacer for balance) */}
            <div className="hidden md:flex flex-1" />

            {/* CTA Buttons + Theme & Language - Right section (flex-1 to balance with left) */}
            <div className="flex-1 hidden md:flex items-center justify-end gap-3">
              <ThemeToggle />
              <LanguageSelector />
              <Link
                to="/login"
                className="text-sm font-medium px-4 py-2 rounded-lg transition-colors hover:opacity-80"
                style={{ color: colors.textSecondary }}
              >
                {t('nav.signIn')}
              </Link>
              <PremiumButton variant="primary" size="md" onClick={() => navigate('/register')}>
                {t('nav.startTrial')}
              </PremiumButton>
            </div>

            {/* Mobile Menu Button */}
            <button
              className="md:hidden"
              style={{ color: colors.textPrimary }}
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
              backgroundColor: colors.bgMobileMenu,
              borderColor: colors.borderSubtle,
            }}
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
          >
            <div className="px-4 py-4 space-y-3">
              <div className="flex items-center gap-2">
                <ThemeToggle />
                <LanguageSelector />
              </div>
              <div className="pt-3 flex flex-col gap-2">
                <Link
                  to="/login"
                  className="block text-center text-sm font-medium py-2.5 rounded-lg transition-colors"
                  style={{
                    color: colors.textSecondary,
                    backgroundColor: colors.bgSecondary,
                  }}
                  onClick={() => setMobileMenuOpen(false)}
                >
                  {t('nav.signIn')}
                </Link>
                <PremiumButton variant="primary" size="md" fullWidth onClick={() => { navigate('/register'); setMobileMenuOpen(false); }}>
                  {t('nav.startTrial')}
                </PremiumButton>
              </div>
            </div>
          </motion.div>
        )}
      </motion.header>

      <main>
        {/* Hero Section */}
        <section className="relative overflow-hidden min-h-[90vh] flex items-center justify-center">
          {/* 3D Background Scene */}
          <Suspense fallback={null}>
            <HeroScene3D className="z-0" />
          </Suspense>

          {/* Gradient Overlay for better text readability */}
          <div
            className="absolute inset-0 z-[1] pointer-events-none"
            style={{
              background: `radial-gradient(ellipse at center, transparent 0%, ${colors.bgPrimary}90 70%, ${colors.bgPrimary} 100%)`,
            }}
          />

          <PageWrapper className="relative z-10 w-full">
            <motion.div
              className="container mx-auto text-center py-20 md:py-32"
              variants={containerVariants}
            >
              {/* Badge */}
              <motion.div className="flex justify-center mb-6" variants={itemVariants}>
                <Badge variant="info" glow>
                  {t('hero.badge')}
                </Badge>
              </motion.div>

              {/* Hero Title */}
              <motion.h1
                className="text-4xl md:text-6xl lg:text-7xl font-black mb-6 leading-tight"
                style={{ color: colors.textPrimary }}
                variants={itemVariants}
              >
                {t('hero.title')}{' '}
                <GradientText className="block">{t('hero.subtitle')}</GradientText>
              </motion.h1>

              {/* Hero Description */}
              <motion.p
                className="text-lg md:text-xl max-w-2xl mx-auto mb-10"
                style={{ color: colors.textSecondary }}
                variants={itemVariants}
              >
                {t('hero.description')}
              </motion.p>

              {/* CTA Buttons */}
              <motion.div
                className="flex flex-col sm:flex-row gap-4 justify-center items-center"
                variants={itemVariants}
              >
                <PremiumButton variant="primary" size="lg" onClick={() => navigate('/register')}>
                  {t('hero.startTrading')}
                  <ArrowRight className="w-5 h-5" />
                </PremiumButton>
                <PremiumButton variant="secondary" size="lg" onClick={() => setVideoModalOpen(true)}>
                  <PlayCircle className="w-5 h-5" />
                  {t('hero.watchDemo')}
                </PremiumButton>
              </motion.div>

              {/* Social Proof */}
              <motion.div
                className="flex flex-wrap justify-center items-center gap-8 mt-16"
                variants={itemVariants}
              >
                <div className="text-center">
                  <GradientText className="text-3xl font-black">10K+</GradientText>
                  <p className="text-xs mt-1" style={{ color: colors.textMuted }}>
                    {t('stats.activeTraders')}
                  </p>
                </div>
                <div
                  className="hidden sm:block w-px h-8"
                  style={{ backgroundColor: colors.borderSubtle }}
                />
                <div className="text-center">
                  <GradientText className="text-3xl font-black">$50M+</GradientText>
                  <p className="text-xs mt-1" style={{ color: colors.textMuted }}>
                    {t('stats.tradingVolume')}
                  </p>
                </div>
                <div
                  className="hidden sm:block w-px h-8"
                  style={{ backgroundColor: colors.borderSubtle }}
                />
                <div className="text-center">
                  <GradientText className="text-3xl font-black">99.9%</GradientText>
                  <p className="text-xs mt-1" style={{ color: colors.textMuted }}>
                    {t('stats.uptime')}
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
              <motion.div className="text-center mb-20" variants={itemVariants}>
                <Badge variant="purple" className="mb-10">
                  {t('features.badge')}
                </Badge>
                <h2 className="text-3xl md:text-5xl font-black mb-6">
                  <GradientText>{t('features.title')}</GradientText>{' '}
                  <span style={{ color: colors.textPrimary }}>{t('features.subtitle')}</span>
                </h2>
                <p
                  className="text-lg max-w-2xl mx-auto"
                  style={{ color: colors.textSecondary }}
                >
                  {t('features.description')}
                </p>
              </motion.div>

              {/* Feature Cards */}
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {[
                  {
                    icon: Brain,
                    title: t('features.aiAnalysis.title'),
                    description: t('features.aiAnalysis.description'),
                    color: colors.purple,
                  },
                  {
                    icon: Shield,
                    title: t('features.riskManagement.title'),
                    description: t('features.riskManagement.description'),
                    color: colors.emerald,
                  },
                  {
                    icon: Zap,
                    title: t('features.realTimeExecution.title'),
                    description: t('features.realTimeExecution.description'),
                    color: colors.amber,
                  },
                  {
                    icon: BarChart3,
                    title: t('features.advancedAnalytics.title'),
                    description: t('features.advancedAnalytics.description'),
                    color: colors.cyan,
                  },
                  {
                    icon: TrendingUp,
                    title: t('features.backtesting.title'),
                    description: t('features.backtesting.description'),
                    color: colors.rose,
                  },
                  {
                    icon: Activity,
                    title: t('features.liveData.title'),
                    description: t('features.liveData.description'),
                    color: colors.purple,
                  },
                ].map((feature, index) => (
                  <motion.div key={index} variants={itemVariants}>
                    <GlassCard hoverable glowColor={`0 8px 32px ${feature.color}30`}>
                      <GlowIcon icon={feature.icon} color={feature.color} size="lg" />
                      <h3
                        className="text-lg font-bold mt-4 mb-2"
                        style={{ color: colors.textPrimary }}
                      >
                        {feature.title}
                      </h3>
                      <p className="text-sm" style={{ color: colors.textSecondary }}>
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
                    label={t('stats.totalUsers')}
                    value="10,234"
                    icon={Users}
                    trend={12.5}
                    trendLabel={t('stats.vsLastMonth')}
                    iconColor={colors.cyan}
                    gradient
                  />
                </motion.div>
                <motion.div variants={itemVariants}>
                  <StatCard
                    label={t('stats.tradingVolume')}
                    value="$50.2M"
                    icon={DollarSign}
                    trend={8.3}
                    trendLabel={t('stats.vsLastMonth')}
                    iconColor={colors.emerald}
                    gradient
                  />
                </motion.div>
                <motion.div variants={itemVariants}>
                  <StatCard
                    label={t('stats.winRate')}
                    value="65%"
                    icon={TrendingUp}
                    trend={2.1}
                    trendLabel={t('stats.vsLastMonth')}
                    iconColor={colors.purple}
                    valueColor={colors.profit}
                  />
                </motion.div>
                <motion.div variants={itemVariants}>
                  <StatCard
                    label={t('stats.responseTime')}
                    value="<100ms"
                    icon={Activity}
                    trend={-15.2}
                    trendLabel={t('stats.faster')}
                    iconColor={colors.amber}
                    valueColor={colors.cyan}
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
              <motion.div className="text-center mb-20" variants={itemVariants}>
                <Badge variant="success" className="mb-10">
                  {t('pricing.badge')}
                </Badge>
                <h2 className="text-3xl md:text-5xl font-black mb-6">
                  <GradientText>{t('pricing.title')}</GradientText>{' '}
                  <span style={{ color: colors.textPrimary }}>{t('pricing.subtitle')}</span>
                </h2>
                <p
                  className="text-lg max-w-2xl mx-auto"
                  style={{ color: colors.textSecondary }}
                >
                  {t('pricing.description')}
                </p>
              </motion.div>

              {/* Pricing Cards */}
              <div className="grid grid-cols-1 md:grid-cols-3 gap-6 max-w-6xl mx-auto">
                {[
                  {
                    planKey: 'starter',
                    price: '$29',
                    variant: 'secondary' as const,
                  },
                  {
                    planKey: 'pro',
                    price: '$99',
                    badge: t('pricing.mostPopular'),
                    variant: 'primary' as const,
                  },
                  {
                    planKey: 'enterprise',
                    price: '$299',
                    variant: 'success' as const,
                  },
                ].map((plan, index) => {
                  const planFeatures = t(`pricing.plans.${plan.planKey}.features`, { returnObjects: true }) as string[];
                  const planName = t(`pricing.plans.${plan.planKey}.name`);
                  return (
                    <motion.div key={index} variants={itemVariants}>
                      <GlassCard hoverable noPadding>
                        <div className="p-6">
                          {plan.badge && (
                            <Badge variant="info" glow className="mb-4">
                              {plan.badge}
                            </Badge>
                          )}
                          <h3 className="text-xl font-bold mb-2" style={{ color: colors.textPrimary }}>{planName}</h3>
                          <div className="flex items-baseline mb-6">
                            <GradientText className="text-4xl font-black">
                              {plan.price}
                            </GradientText>
                            <span
                              className="text-sm ml-2"
                              style={{ color: colors.textMuted }}
                            >
                              {t('pricing.month')}
                            </span>
                          </div>
                          <ul className="space-y-3 mb-6">
                            {planFeatures.map((feature, fIndex) => (
                              <li key={fIndex} className="flex items-start gap-2">
                                <CheckCircle2
                                  className="w-4 h-4 mt-0.5 flex-shrink-0"
                                  style={{ color: colors.emerald }}
                                />
                                <span
                                  className="text-sm"
                                  style={{ color: colors.textSecondary }}
                                >
                                  {feature}
                                </span>
                              </li>
                            ))}
                          </ul>
                          <PremiumButton variant={plan.variant} fullWidth>
                            {t('pricing.getStarted')}
                          </PremiumButton>
                        </div>
                      </GlassCard>
                    </motion.div>
                  );
                })}
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
              <motion.div className="text-center mb-20" variants={itemVariants}>
                <Badge variant="warning" className="mb-10">
                  {t('testimonials.badge')}
                </Badge>
                <h2 className="text-3xl md:text-5xl font-black mb-6">
                  <span style={{ color: colors.textPrimary }}>{t('testimonials.title')}</span> <GradientText>{t('testimonials.subtitle')}</GradientText>
                </h2>
              </motion.div>

              {/* Testimonial Cards */}
              <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                {(t('testimonials.reviews', { returnObjects: true }) as Array<{ name: string; role: string; text: string }>).map((testimonial, index) => {
                  const avatars = ['👩‍💼', '👨‍💻', '👨‍🔬'];
                  return { ...testimonial, avatar: avatars[index % avatars.length], rating: 5 };
                }).map((testimonial, index) => (
                  <motion.div key={index} variants={itemVariants}>
                    <GlassCard hoverable>
                      <div className="flex items-center gap-3 mb-4">
                        <div className="text-3xl">{testimonial.avatar}</div>
                        <div>
                          <h4 className="text-sm font-bold" style={{ color: colors.textPrimary }}>
                            {testimonial.name}
                          </h4>
                          <p
                            className="text-xs"
                            style={{ color: colors.textMuted }}
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
                            fill={colors.amber}
                            style={{ color: colors.amber }}
                          />
                        ))}
                      </div>
                      <Quote
                        className="w-6 h-6 mb-2 opacity-20"
                        style={{ color: colors.cyan }}
                      />
                      <p className="text-sm" style={{ color: colors.textSecondary }}>
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
              <motion.div className="text-center mb-20" variants={itemVariants}>
                <Badge variant="info" className="mb-10">
                  {t('faq.badge')}
                </Badge>
                <h2 className="text-3xl md:text-5xl font-black mb-6">
                  <GradientText>{t('faq.title')}</GradientText>{' '}
                  <span style={{ color: colors.textPrimary }}>{t('faq.subtitle')}</span>
                </h2>
              </motion.div>

              {/* FAQ Items */}
              <div className="max-w-3xl mx-auto space-y-4">
                {(t('faq.items', { returnObjects: true }) as Array<{ question: string; answer: string }>).map((faq, index) => (
                  <motion.div key={index} variants={itemVariants}>
                    <GlassCard hoverable>
                      <h3 className="text-base font-bold mb-2" style={{ color: colors.textPrimary }}>{faq.question}</h3>
                      <p className="text-sm" style={{ color: colors.textSecondary }}>
                        {faq.answer}
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
                    <Badge variant="info" glow className="mb-4">
                      {t('cta.badge')}
                    </Badge>
                    <h2 className="text-3xl md:text-5xl font-black mb-6">
                      <span style={{ color: colors.textPrimary }}>{t('cta.title')}</span>{' '}
                      <GradientText>{t('cta.subtitle')}</GradientText>
                    </h2>
                    <p
                      className="text-lg mb-8"
                      style={{ color: colors.textSecondary }}
                    >
                      {t('cta.description')}
                    </p>
                    <div className="flex flex-col sm:flex-row gap-4 justify-center">
                      <PremiumButton variant="primary" size="lg" onClick={() => navigate('/register')}>
                        {t('cta.startTrial')}
                        <ArrowRight className="w-5 h-5" />
                      </PremiumButton>
                      <PremiumButton variant="secondary" size="lg" onClick={() => navigate('/contact')}>
                        {t('cta.scheduleDemo')}
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
          backgroundColor: colors.bgPrimary,
          borderColor: colors.borderSubtle,
        }}
      >
        <div className="container mx-auto px-4 py-12">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-8 mb-8">
            {/* Brand */}
            <div>
              <div className="mb-4">
                <BotCoreLogo size="md" />
              </div>
              <p className="text-xs" style={{ color: colors.textMuted }}>
                {t('footer.description')}
              </p>
            </div>

            {/* Links */}
            {[
              {
                title: t('footer.product'),
                links: [
                  { label: t('footer.features'), path: '/features' },
                  { label: t('footer.pricing'), path: '/pricing' },
                  { label: t('footer.api'), path: '/api' },
                  { label: t('footer.documentation'), path: '/docs' },
                ],
              },
              {
                title: t('footer.company'),
                links: [
                  { label: t('footer.about'), path: '/about' },
                  { label: t('footer.blog'), path: '/blog' },
                  { label: t('footer.careers'), path: '/careers' },
                  { label: t('footer.contact'), path: '/contact' },
                ],
              },
              {
                title: t('footer.legal'),
                links: [
                  { label: t('footer.privacy'), path: '/privacy' },
                  { label: t('footer.terms'), path: '/terms' },
                  { label: t('footer.security'), path: '/security' },
                  { label: t('footer.compliance'), path: '/compliance' },
                ],
              },
            ].map((column, index) => (
              <div key={index}>
                <h4
                  className="text-sm font-bold mb-3"
                  style={{ color: colors.textPrimary }}
                >
                  {column.title}
                </h4>
                <ul className="space-y-2">
                  {column.links.map((link, linkIndex) => (
                    <li key={linkIndex}>
                      <Link
                        to={link.path}
                        className="text-xs transition-colors"
                        style={{ color: colors.textMuted }}
                        onMouseEnter={(e) => e.currentTarget.style.color = colors.textPrimary}
                        onMouseLeave={(e) => e.currentTarget.style.color = colors.textMuted}
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
            style={{ borderColor: colors.borderSubtle }}
          >
            <p className="text-xs" style={{ color: colors.textMuted }}>
              {t('footer.copyright')}
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
                  className="text-xs transition-colors"
                  style={{ color: colors.textMuted }}
                  onMouseEnter={(e) => e.currentTarget.style.color = colors.textPrimary}
                  onMouseLeave={(e) => e.currentTarget.style.color = colors.textMuted}
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

      {/* Video Demo Modal */}
      {videoModalOpen && (
        <motion.div
          className="fixed inset-0 z-[100] flex items-center justify-center p-4"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
        >
          {/* Backdrop */}
          <div
            className="absolute inset-0 bg-black/80 backdrop-blur-sm"
            onClick={() => setVideoModalOpen(false)}
          />

          {/* Modal Content */}
          <motion.div
            className="relative z-10 w-full max-w-4xl rounded-2xl overflow-hidden"
            style={{
              backgroundColor: colors.bgCard,
              boxShadow: `0 25px 50px -12px ${colors.cyan}40`,
            }}
            initial={{ scale: 0.9, opacity: 0, y: 20 }}
            animate={{ scale: 1, opacity: 1, y: 0 }}
            transition={{ type: 'spring', damping: 25, stiffness: 300 }}
          >
            {/* Close Button */}
            <button
              onClick={() => setVideoModalOpen(false)}
              className="absolute top-4 right-4 z-20 p-2 rounded-full transition-colors hover:bg-white/10"
              style={{ color: colors.textPrimary }}
            >
              <X className="w-6 h-6" />
            </button>

            {/* Video Header */}
            <div className="p-6 border-b" style={{ borderColor: colors.borderSubtle }}>
              <div className="flex items-center gap-3">
                <div
                  className="w-10 h-10 rounded-lg flex items-center justify-center"
                  style={{
                    background: colors.gradientPremium,
                    boxShadow: colors.glowCyan,
                  }}
                >
                  <PlayCircle className="w-5 h-5 text-white" />
                </div>
                <div>
                  <h3 className="text-lg font-bold" style={{ color: colors.textPrimary }}>
                    {t('videoModal.title')}
                  </h3>
                  <p className="text-sm" style={{ color: colors.textMuted }}>
                    {t('videoModal.subtitle')}
                  </p>
                </div>
              </div>
            </div>

            {/* Video Player */}
            <div className="relative aspect-video bg-black">
              <video
                className="w-full h-full object-cover"
                controls
                autoPlay
                poster="/demo-poster.jpg"
              >
                <source src="/demo/bot-core-demo.mp4" type="video/mp4" />
                Your browser does not support the video tag.
              </video>
            </div>

            {/* Video Footer */}
            <div className="p-4 flex items-center justify-between" style={{ backgroundColor: colors.bgSecondary }}>
              <p className="text-sm" style={{ color: colors.textSecondary }}>
                {t('videoModal.cta')}
              </p>
              <PremiumButton variant="primary" size="sm" onClick={() => { setVideoModalOpen(false); navigate('/register'); }}>
                {t('videoModal.getStarted')}
                <ArrowRight className="w-4 h-4" />
              </PremiumButton>
            </div>
          </motion.div>
        </motion.div>
      )}
    </div>
  );
};

export default Index;
