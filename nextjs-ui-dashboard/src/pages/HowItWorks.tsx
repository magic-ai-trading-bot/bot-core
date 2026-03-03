/**
 * How It Works Page - Luxury OLED Design
 *
 * Trang giải thích cách bot hoạt động với UI cao cấp OLED
 * Verified data against rust-core-engine/src/paper_trading/settings.rs
 */

import { useState, useEffect, Fragment } from 'react';
import { useTranslation } from 'react-i18next';
import { motion } from 'framer-motion';
import {
  GlassCard,
  GradientText,
  Badge,
  GlowIcon,
  SectionHeader,
  containerVariants,
  itemVariants,
  Divider,
} from '@/styles/luxury-design-system';
import { useThemeColors } from '@/hooks/useThemeColors';
import {
  Database,
  TrendingUp,
  Brain,
  Shield,
  CheckCircle,
  Zap,
  BarChart3,
  Activity,
  Target,
  Clock,
  AlertTriangle,
  ChevronRight,
  Play,
  Sparkles,
  Lock,
  TrendingDown,
  Percent,
  Timer,
  Layers,
  LineChart,
} from 'lucide-react';

const HowItWorks = () => {
  const { t } = useTranslation('pages');
  const colors = useThemeColors();
  const [activeStep, setActiveStep] = useState(0);
  const [liveSettings, setLiveSettings] = useState<any>(null);
  const [atrDiag, setAtrDiag] = useState<any>(null);

  useEffect(() => {
    const API_BASE = import.meta.env.VITE_API_URL || 'http://localhost:8080';
    Promise.all([
      fetch(`${API_BASE}/api/paper-trading/basic-settings`).then(r => r.json()).catch(() => null),
      fetch(`${API_BASE}/api/paper-trading/atr-diagnostics`).then(r => r.json()).catch(() => null),
    ]).then(([settings, atr]) => {
      setLiveSettings(settings?.data || null);
      setAtrDiag(atr?.data || null);
    });
  }, []);

  // Verified against rust-core-engine settings — 8 risk layers (added weekly drawdown)
  const stats = [
    { value: '5', label: t('howItWorks.stats.strategies'), icon: BarChart3, color: colors.cyan },
    { value: '8', label: t('howItWorks.stats.riskLayers'), icon: Shield, color: colors.emerald },
    { value: '24/7', label: t('howItWorks.stats.uptime'), icon: Clock, color: colors.amber },
    {
      value: atrDiag?.kelly?.win_rate != null ? `${atrDiag.kelly.win_rate.toFixed(1)}%` : '—',
      label: t('howItWorks.stats.winRate'),
      icon: Brain,
      color: colors.purple,
    },
  ];

  const steps = [
    {
      number: 1,
      title: t('howItWorks.steps.dataCollection.title'),
      subtitle: t('howItWorks.steps.dataCollection.subtitle'),
      icon: Database,
      color: colors.cyan,
      description: t('howItWorks.steps.dataCollection.description'),
      details: (t('howItWorks.steps.dataCollection.details', { returnObjects: true }) as string[]).map((text, idx) => ({
        icon: [Activity, BarChart3, Clock, Zap][idx],
        text
      }))
    },
    {
      number: 2,
      title: t('howItWorks.steps.technicalAnalysis.title'),
      subtitle: t('howItWorks.steps.technicalAnalysis.subtitle'),
      icon: LineChart,
      color: colors.emerald,
      description: t('howItWorks.steps.technicalAnalysis.description'),
      details: (t('howItWorks.steps.technicalAnalysis.details', { returnObjects: true }) as string[]).map((text, idx) => ({
        icon: [TrendingUp, BarChart3, Activity, Layers][idx],
        text
      }))
    },
    {
      number: 3,
      title: t('howItWorks.steps.signalGeneration.title'),
      subtitle: t('howItWorks.steps.signalGeneration.subtitle'),
      icon: Brain,
      color: colors.purple,
      description: t('howItWorks.steps.signalGeneration.description'),
      details: (t('howItWorks.steps.signalGeneration.details', { returnObjects: true }) as string[]).map((text, idx) => ({
        icon: [Target, Percent, Clock, Sparkles][idx],
        text
      }))
    },
    {
      number: 4,
      title: t('howItWorks.steps.riskManagement.title'),
      subtitle: t('howItWorks.steps.riskManagement.subtitle'),
      icon: Shield,
      color: colors.rose,
      description: t('howItWorks.steps.riskManagement.description'),
      details: (t('howItWorks.steps.riskManagement.details', { returnObjects: true }) as string[]).map((text, idx) => ({
        icon: [Lock, AlertTriangle, Timer, TrendingDown][idx],
        text
      }))
    },
  ];

  // Verified against settings.rs
  const strategies = [
    {
      name: t('howItWorks.strategies.items.rsi.name'),
      winRate: 65,
      description: t('howItWorks.strategies.items.rsi.description'),
      icon: TrendingUp,
      color: colors.cyan,
      signals: { buy: 'RSI < 30', sell: 'RSI > 70' }
    },
    {
      name: t('howItWorks.strategies.items.macd.name'),
      winRate: 61,
      description: t('howItWorks.strategies.items.macd.description'),
      icon: BarChart3,
      color: colors.emerald,
      signals: { buy: 'MACD crosses up', sell: 'MACD crosses down' }
    },
    {
      name: t('howItWorks.strategies.items.bollinger.name'),
      winRate: 63,
      description: t('howItWorks.strategies.items.bollinger.description'),
      icon: Activity,
      color: colors.amber,
      signals: { buy: 'Touch lower band', sell: 'Touch upper band' }
    },
    {
      name: t('howItWorks.strategies.items.volume.name'),
      winRate: 58,
      description: t('howItWorks.strategies.items.volume.description'),
      icon: Layers,
      color: colors.purple,
      signals: { buy: 'Volume spike + up', sell: 'Volume spike + down' }
    },
    {
      name: t('howItWorks.strategies.items.stochastic.name'),
      winRate: 64,
      description: t('howItWorks.strategies.items.stochastic.description'),
      icon: Target,
      color: colors.rose,
      signals: { buy: '%K crosses %D < 20', sell: '%K crosses %D > 80' }
    },
  ];

  // Verified against settings.rs defaults — uses live settings when available
  const riskLayers = [
    {
      layer: 1,
      name: t('howItWorks.risk.layers.position.title'),
      value: liveSettings ? `≤${liveSettings.risk?.default_position_size_pct ?? 5}%` : '≤5%',
      desc: t('howItWorks.risk.layers.position.description'),
      icon: Percent,
    },
    {
      layer: 2,
      name: t('howItWorks.risk.layers.stopLoss.title'),
      value: liveSettings ? `${liveSettings.risk?.default_stop_loss_pct ?? 10}%` : '10%',
      desc: t('howItWorks.risk.layers.stopLoss.description'),
      icon: AlertTriangle,
    },
    {
      layer: 3,
      name: t('howItWorks.risk.layers.portfolio.title'),
      value: liveSettings ? `≤${liveSettings.risk?.max_portfolio_risk_pct ?? 10}%` : '≤10%',
      desc: t('howItWorks.risk.layers.portfolio.description'),
      icon: Layers,
    },
    {
      layer: 4,
      name: t('howItWorks.risk.layers.daily.title'),
      value: liveSettings ? `${liveSettings.risk?.daily_loss_limit_pct ?? 3}%` : '3%',
      desc: t('howItWorks.risk.layers.daily.description'),
      icon: TrendingDown,
    },
    {
      layer: 5,
      name: t('howItWorks.risk.layers.consecutiveLosses.title'),
      value: liveSettings ? `${liveSettings.risk?.max_consecutive_losses ?? 3} max` : '3 max',
      desc: t('howItWorks.risk.layers.consecutiveLosses.description'),
      icon: Timer,
    },
    {
      layer: 6,
      name: t('howItWorks.risk.layers.cooldown.title'),
      value: liveSettings ? `${liveSettings.risk?.cool_down_minutes ?? 60} min` : '60 min',
      desc: t('howItWorks.risk.layers.cooldown.description'),
      icon: Clock,
    },
    {
      layer: 7,
      name: t('howItWorks.risk.layers.correlation.title'),
      value: liveSettings ? `${(liveSettings.risk?.correlation_limit ?? 0.7) * 100}%` : '70%',
      desc: t('howItWorks.risk.layers.correlation.description'),
      icon: Activity,
    },
    {
      layer: 8,
      name: t('howItWorks.risk.layers.weeklyDrawdown.title'),
      value: atrDiag ? `${atrDiag.weekly_drawdown?.limit_pct ?? 7}%` : '7%',
      desc: t('howItWorks.risk.layers.weeklyDrawdown.description'),
      icon: TrendingDown,
    },
  ];

  return (
      <div className="min-h-screen" style={{ backgroundColor: colors.bgPrimary }}>
        {/* Hero Section */}
        <section className="relative overflow-hidden">
          <div
            className="absolute inset-0"
            style={{
              background: 'radial-gradient(ellipse at top, rgba(0, 217, 255, 0.1), transparent)',
            }}
          />
          <div
            className="absolute inset-0"
            style={{
              background: 'radial-gradient(ellipse at bottom right, rgba(139, 92, 246, 0.08), transparent)',
            }}
          />

          <motion.div
            className="relative max-w-7xl mx-auto px-4 py-16 lg:py-24"
            variants={containerVariants}
            initial="hidden"
            animate="visible"
          >
            <div className="text-center space-y-6">
              <motion.div variants={itemVariants} className="flex justify-center">
                <Badge variant="info" glow>
                  <Sparkles className="h-3 w-3 mr-1.5" />
                  {t('howItWorks.badge')}
                </Badge>
              </motion.div>

              <motion.h1
                variants={itemVariants}
                className="text-4xl lg:text-6xl font-black tracking-tight"
                style={{ color: colors.textPrimary }}
              >
                {t('howItWorks.hero.title')}{' '}
                <GradientText className="text-4xl lg:text-6xl font-black">
                  {t('howItWorks.hero.titleHighlight')}
                </GradientText>
              </motion.h1>

              <motion.p
                variants={itemVariants}
                className="text-lg lg:text-xl max-w-2xl mx-auto"
                style={{ color: colors.textSecondary }}
              >
                {t('howItWorks.hero.subtitle')}
              </motion.p>

              {/* Stats */}
              <motion.div
                variants={containerVariants}
                className="grid grid-cols-2 md:grid-cols-4 gap-4 max-w-3xl mx-auto pt-8"
              >
                {stats.map((stat, index) => (
                  <GlassCard key={index} hoverable>
                    <div className="text-center">
                      <div className="flex justify-center mb-2">
                        <GlowIcon icon={stat.icon} color={stat.color} size="md" />
                      </div>
                      <div
                        className="text-2xl font-black mb-1"
                        style={{ color: stat.color }}
                      >
                        {stat.value}
                      </div>
                      <div
                        className="text-[10px] uppercase tracking-wider"
                        style={{ color: colors.textMuted }}
                      >
                        {stat.label}
                      </div>
                    </div>
                  </GlassCard>
                ))}
              </motion.div>

            </div>
          </motion.div>
        </section>

        <motion.div
          className="max-w-7xl mx-auto px-4 pt-20 pb-16 space-y-16"
          variants={containerVariants}
          initial="hidden"
          animate="visible"
        >

          {/* How It Works - 4 Steps */}
          <section>
            <motion.div variants={itemVariants}>
              <SectionHeader
                title={t('howItWorks.process.title')}
                subtitle={t('howItWorks.process.subtitle')}
                icon={Zap}
                gradient
              />
            </motion.div>

            {/* Step Cards */}
            <motion.div
              variants={containerVariants}
              className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8"
            >
              {steps.map((step, index) => (
                <GlassCard
                  key={step.number}
                  hoverable
                  onClick={() => setActiveStep(index)}
                  glowColor={activeStep === index ? `0 8px 32px ${step.color}30` : undefined}
                  className={activeStep === index ? 'ring-2' : ''}
                  style={{
                    borderColor: activeStep === index ? step.color : undefined,
                  }}
                >
                  <div className="flex items-center gap-3 mb-3">
                    <div
                      className="p-2.5 rounded-xl"
                      style={{
                        background: `${step.color}20`,
                        border: `1px solid ${step.color}40`,
                      }}
                    >
                      <step.icon className="h-5 w-5" style={{ color: step.color }} />
                    </div>
                    <div className="flex-1">
                      <Badge size="sm">STEP {step.number}</Badge>
                      <h3
                        className="text-sm font-bold mt-1"
                        style={{ color: colors.textPrimary }}
                      >
                        {step.title}
                      </h3>
                    </div>
                  </div>
                  <p
                    className="text-xs"
                    style={{ color: colors.textSecondary }}
                  >
                    {step.subtitle}
                  </p>
                </GlassCard>
              ))}
            </motion.div>

            {/* Step Details */}
            <motion.div variants={itemVariants}>
              <GlassCard
                className="border-2"
                style={{
                  borderColor: steps[activeStep].color,
                  backgroundColor: `${steps[activeStep].color}05`,
                }}
              >
                <div className="flex items-center gap-4 mb-6">
                  <GlowIcon
                    icon={steps[activeStep].icon}
                    color={steps[activeStep].color}
                    size="lg"
                  />
                  <div>
                    <h3
                      className="text-xl font-black"
                      style={{ color: steps[activeStep].color }}
                    >
                      Step {steps[activeStep].number}: {steps[activeStep].title}
                    </h3>
                    <p
                      className="text-sm mt-1"
                      style={{ color: colors.textSecondary }}
                    >
                      {steps[activeStep].description}
                    </p>
                  </div>
                </div>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                  {steps[activeStep].details.map((detail, idx) => (
                    <div
                      key={idx}
                      className="flex items-center gap-3 p-3 rounded-xl"
                      style={{
                        backgroundColor: colors.bgSecondary,
                        border: `1px solid ${colors.borderSubtle}`,
                      }}
                    >
                      <GlowIcon
                        icon={detail.icon}
                        color={steps[activeStep].color}
                        size="sm"
                      />
                      <span
                        className="text-sm"
                        style={{ color: colors.textPrimary }}
                      >
                        {detail.text}
                      </span>
                    </div>
                  ))}
                </div>
              </GlassCard>
            </motion.div>
          </section>

          {/* Trading Strategies */}
          <section>
            <motion.div variants={itemVariants}>
              <SectionHeader
                title={t('howItWorks.strategies.title')}
                subtitle={t('howItWorks.strategies.subtitle')}
                icon={LineChart}
                gradient
              />
            </motion.div>

            <motion.div
              variants={containerVariants}
              className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4"
            >
              {strategies.map((strategy) => (
                <GlassCard key={strategy.name} hoverable>
                  <div className="flex justify-center mb-4">
                    <motion.div
                      whileHover={{ scale: 1.1, rotate: 5 }}
                      transition={{ type: 'spring', stiffness: 300 }}
                    >
                      <GlowIcon icon={strategy.icon} color={strategy.color} size="lg" />
                    </motion.div>
                  </div>
                  <h3
                    className="text-sm font-black mb-1"
                    style={{ color: colors.textPrimary }}
                  >
                    {strategy.name}
                  </h3>
                  <p
                    className="text-[10px] mb-4"
                    style={{ color: colors.textMuted }}
                  >
                    {strategy.description}
                  </p>

                  <div className="flex items-center justify-between mb-3">
                    <span
                      className="text-[10px] uppercase tracking-wider"
                      style={{ color: colors.textMuted }}
                    >
                      {t('howItWorks.strategies.winRate')}
                    </span>
                    <Badge variant="success" size="sm">
                      {strategy.winRate}%
                    </Badge>
                  </div>

                  <Divider className="mb-3" />

                  <div className="space-y-2 text-xs">
                    <div className="flex items-center gap-2">
                      <div
                        className="w-1.5 h-1.5 rounded-full"
                        style={{ backgroundColor: colors.emerald }}
                      />
                      <span style={{ color: colors.textMuted }}>{t('howItWorks.strategies.buy')}:</span>
                      <span style={{ color: colors.textSecondary }}>
                        {strategy.signals.buy}
                      </span>
                    </div>
                    <div className="flex items-center gap-2">
                      <div
                        className="w-1.5 h-1.5 rounded-full"
                        style={{ backgroundColor: colors.rose }}
                      />
                      <span style={{ color: colors.textMuted }}>{t('howItWorks.strategies.sell')}:</span>
                      <span style={{ color: colors.textSecondary }}>
                        {strategy.signals.sell}
                      </span>
                    </div>
                  </div>
                </GlassCard>
              ))}
            </motion.div>

            {/* Strategy Note */}
            <motion.div variants={itemVariants} className="mt-6">
              <GlassCard
                className="border"
                style={{
                  borderColor: `${colors.cyan}40`,
                  backgroundColor: `${colors.cyan}08`,
                }}
              >
                <div className="flex items-start gap-3">
                  <CheckCircle className="h-5 w-5 mt-0.5" style={{ color: colors.cyan }} />
                  <div>
                    <p className="font-bold text-sm" style={{ color: colors.textPrimary }}>
                      {t('howItWorks.strategies.multiConfirmation.title')}
                    </p>
                    <p className="text-xs mt-1" style={{ color: colors.textSecondary }}>
                      {t('howItWorks.strategies.multiConfirmation.description')}
                    </p>
                  </div>
                </div>
              </GlassCard>
            </motion.div>
          </section>

          {/* Risk Management */}
          <section>
            <motion.div variants={itemVariants}>
              <SectionHeader
                title={t('howItWorks.risk.title')}
                subtitle={t('howItWorks.risk.subtitle')}
                icon={Shield}
                gradient
              />
            </motion.div>

            <motion.div
              variants={containerVariants}
              className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4"
            >
              {riskLayers.slice(0, 4).map((layer) => (
                <GlassCard key={layer.layer} hoverable className="relative overflow-hidden">
                  <div
                    className="absolute top-0 left-0 w-1 h-full"
                    style={{
                      background: colors.gradientPremium,
                    }}
                  />
                  <div className="pl-3">
                    <div className="flex items-center justify-between mb-3">
                      <Badge size="sm">LAYER {layer.layer}</Badge>
                      <GlowIcon icon={layer.icon} color={colors.emerald} size="sm" />
                    </div>
                    <h3
                      className="text-sm font-bold mb-2"
                      style={{ color: colors.textPrimary }}
                    >
                      {layer.name}
                    </h3>
                    <div
                      className="text-2xl font-black mb-1"
                      style={{ color: colors.emerald }}
                    >
                      {layer.value}
                    </div>
                    <p
                      className="text-[10px]"
                      style={{ color: colors.textMuted }}
                    >
                      {layer.desc}
                    </p>
                  </div>
                </GlassCard>
              ))}
            </motion.div>

            <motion.div
              variants={containerVariants}
              className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mt-4"
            >
              {riskLayers.slice(4).map((layer) => (
                <GlassCard key={layer.layer} hoverable className="relative overflow-hidden">
                  <div
                    className="absolute top-0 left-0 w-1 h-full"
                    style={{
                      background: colors.gradientPremium,
                    }}
                  />
                  <div className="pl-3 flex items-center gap-4">
                    <GlowIcon icon={layer.icon} color={colors.emerald} size="md" />
                    <div className="flex-1">
                      <div className="flex items-center justify-between mb-1">
                        <span
                          className="text-sm font-bold"
                          style={{ color: colors.textPrimary }}
                        >
                          {layer.name}
                        </span>
                        <Badge variant="success" size="sm">
                          {layer.value}
                        </Badge>
                      </div>
                      <p
                        className="text-[10px]"
                        style={{ color: colors.textMuted }}
                      >
                        {layer.desc}
                      </p>
                    </div>
                  </div>
                </GlassCard>
              ))}
            </motion.div>
          </section>

          {/* Adaptive Position Sizing */}
          <section>
            <motion.div variants={itemVariants}>
              <SectionHeader
                title={t('howItWorks.adaptive.title')}
                subtitle={t('howItWorks.adaptive.subtitle')}
                icon={Target}
                gradient
              />
            </motion.div>

            <motion.div
              variants={containerVariants}
              className="grid grid-cols-1 md:grid-cols-3 gap-4"
            >
              {/* ATR Card */}
              <GlassCard hoverable>
                <div className="flex items-center gap-3 mb-3">
                  <GlowIcon icon={Activity} color={colors.cyan} size="md" />
                  <div>
                    <h3 className="text-sm font-bold" style={{ color: colors.textPrimary }}>
                      {t('howItWorks.adaptive.atr.title')}
                    </h3>
                    <Badge variant={atrDiag?.atr_stop_enabled ? 'success' : 'info'} size="sm">
                      {atrDiag?.atr_stop_enabled ? 'ACTIVE' : 'STANDBY'}
                    </Badge>
                  </div>
                </div>
                <p className="text-xs mb-3" style={{ color: colors.textSecondary }}>
                  {t('howItWorks.adaptive.atr.description')}
                </p>
                <div className="space-y-1 text-xs" style={{ color: colors.textMuted }}>
                  <div>SL Multiplier: <span style={{ color: colors.textPrimary }}>{atrDiag?.atr_stop_multiplier ?? '1.2'}x ATR</span></div>
                  <div>TP Multiplier: <span style={{ color: colors.textPrimary }}>{atrDiag?.atr_tp_multiplier ?? '2.4'}x ATR</span></div>
                  <div>Base Risk: <span style={{ color: colors.textPrimary }}>{atrDiag?.base_risk_pct ?? '2.0'}%</span></div>
                </div>
              </GlassCard>

              {/* Kelly Card */}
              <GlassCard hoverable>
                <div className="flex items-center gap-3 mb-3">
                  <GlowIcon icon={Brain} color={colors.purple} size="md" />
                  <div>
                    <h3 className="text-sm font-bold" style={{ color: colors.textPrimary }}>
                      {t('howItWorks.adaptive.kelly.title')}
                    </h3>
                    <Badge variant={atrDiag?.kelly?.enabled ? 'success' : 'info'} size="sm">
                      {atrDiag?.kelly?.enabled
                        ? 'ACTIVE'
                        : `${atrDiag?.kelly?.total_closed_trades ?? 0}/${atrDiag?.kelly?.min_trades ?? 200} trades`}
                    </Badge>
                  </div>
                </div>
                <p className="text-xs mb-3" style={{ color: colors.textSecondary }}>
                  {t('howItWorks.adaptive.kelly.description')}
                </p>
                <div className="space-y-1 text-xs" style={{ color: colors.textMuted }}>
                  <div>Win Rate: <span style={{ color: colors.textPrimary }}>{atrDiag?.kelly?.win_rate?.toFixed(1) ?? '—'}%</span></div>
                  <div>Fraction: <span style={{ color: colors.textPrimary }}>{atrDiag?.kelly?.fraction ?? 0.5} (Half-Kelly)</span></div>
                  <div>Lookback: <span style={{ color: colors.textPrimary }}>{atrDiag?.kelly?.lookback ?? 100} trades</span></div>
                </div>
              </GlassCard>

              {/* Regime Filters Card */}
              <GlassCard hoverable>
                <div className="flex items-center gap-3 mb-3">
                  <GlowIcon icon={Shield} color={colors.emerald} size="md" />
                  <div>
                    <h3 className="text-sm font-bold" style={{ color: colors.textPrimary }}>
                      {t('howItWorks.adaptive.regime.title')}
                    </h3>
                    <Badge variant="success" size="sm">
                      {[
                        atrDiag?.regime_filters?.funding_spike?.enabled,
                        atrDiag?.regime_filters?.atr_spike?.enabled,
                        atrDiag?.regime_filters?.consecutive_loss?.enabled,
                      ].filter(Boolean).length}/3 ACTIVE
                    </Badge>
                  </div>
                </div>
                <p className="text-xs mb-3" style={{ color: colors.textSecondary }}>
                  {t('howItWorks.adaptive.regime.description')}
                </p>
                <div className="space-y-1.5 text-xs" style={{ color: colors.textMuted }}>
                  <div className="flex items-center gap-1.5">
                    <div
                      className="w-1.5 h-1.5 rounded-full"
                      style={{ backgroundColor: atrDiag?.regime_filters?.funding_spike?.enabled ? colors.emerald : colors.textMuted }}
                    />
                    Funding Spike Filter
                  </div>
                  <div className="flex items-center gap-1.5">
                    <div
                      className="w-1.5 h-1.5 rounded-full"
                      style={{ backgroundColor: atrDiag?.regime_filters?.atr_spike?.enabled ? colors.emerald : colors.textMuted }}
                    />
                    ATR Spike Filter
                  </div>
                  <div className="flex items-center gap-1.5">
                    <div
                      className="w-1.5 h-1.5 rounded-full"
                      style={{ backgroundColor: atrDiag?.regime_filters?.consecutive_loss?.enabled ? colors.emerald : colors.textMuted }}
                    />
                    Consecutive Loss Reduction
                  </div>
                </div>
              </GlassCard>
            </motion.div>
          </section>

          {/* Trailing Stop Example */}
          <section>
            <motion.div variants={itemVariants}>
              <GlassCard className="overflow-hidden">
                <div
                  className="p-4 rounded-t-2xl"
                  style={{
                    background: `linear-gradient(135deg, ${colors.emerald}15, ${colors.cyan}10)`,
                  }}
                >
                  <div className="flex items-center gap-3 mb-2">
                    <GlowIcon icon={TrendingUp} color={colors.emerald} size="md" />
                    <h3 className="text-lg font-black" style={{ color: colors.textPrimary }}>
                      {t('howItWorks.trailingStop.title')}
                    </h3>
                  </div>
                  <p className="text-xs" style={{ color: colors.textSecondary }}>
                    {t('howItWorks.trailingStop.subtitle')}
                  </p>
                </div>

                <div className="p-6">
                  <div className="flex flex-wrap items-center justify-center gap-2 md:gap-4">
                    {[
                      { label: t('howItWorks.trailingStop.steps.entry'), value: '$100,000', color: colors.textMuted, icon: Play },
                      { label: t('howItWorks.trailingStop.steps.profit'), value: '$100,100', color: colors.emerald, icon: TrendingUp },
                      { label: t('howItWorks.trailingStop.steps.trailing'), value: '$99,320', color: colors.amber, icon: Zap },
                      { label: t('howItWorks.trailingStop.steps.peak'), value: '$102,000', color: colors.emerald, icon: TrendingUp },
                      { label: t('howItWorks.trailingStop.steps.exit'), value: '$101,184', color: colors.emerald, icon: CheckCircle, highlight: true },
                    ].map((step, idx) => (
                      <Fragment key={idx}>
                        <motion.div
                          whileHover={{ scale: 1.05 }}
                          className={`p-4 rounded-xl text-center min-w-[100px] ${step.highlight ? 'ring-2' : ''}`}
                          style={{
                            backgroundColor: `${step.color}15`,
                            border: `1px solid ${step.color}30`,
                            borderColor: step.highlight ? step.color : undefined,
                          }}
                        >
                          <div className="flex justify-center mb-2">
                            <step.icon className="h-4 w-4" style={{ color: step.color }} />
                          </div>
                          <div className="text-lg font-black" style={{ color: step.color }}>
                            {step.value}
                          </div>
                          <div className="text-[10px]" style={{ color: colors.textMuted }}>
                            {step.label}
                          </div>
                        </motion.div>
                        {idx < 4 && (
                          <ChevronRight
                            className="h-5 w-5 hidden md:block"
                            style={{ color: colors.textMuted }}
                          />
                        )}
                      </Fragment>
                    ))}
                  </div>

                  <div
                    className="mt-6 p-4 rounded-xl"
                    style={{
                      backgroundColor: `${colors.emerald}15`,
                      border: `1px solid ${colors.emerald}30`,
                    }}
                  >
                    <div className="flex items-center gap-2">
                      <CheckCircle className="h-5 w-5" style={{ color: colors.emerald }} />
                      <span
                        className="font-bold text-sm"
                        style={{ color: colors.emerald }}
                      >
                        {t('howItWorks.trailingStop.result')}
                      </span>
                    </div>
                  </div>
                </div>
              </GlassCard>
            </motion.div>
          </section>

        </motion.div>
      </div>
  );
};

export default HowItWorks;
