/**
 * How It Works Page - Luxury OLED Design
 *
 * Trang giải thích cách bot hoạt động với UI cao cấp OLED
 * Verified data against rust-core-engine/src/paper_trading/settings.rs
 */

import { useState, Fragment } from 'react';
import { motion } from 'framer-motion';
import {
  luxuryColors,
  GlassCard,
  GradientText,
  Badge,
  GlowIcon,
  SectionHeader,
  containerVariants,
  itemVariants,
  Divider,
} from '@/styles/luxury-design-system';
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
  const [activeStep, setActiveStep] = useState(0);

  // Verified against rust-core-engine settings
  const stats = [
    { value: '5', label: 'Strategies', icon: BarChart3, color: luxuryColors.cyan },
    { value: '7', label: 'Risk Layers', icon: Shield, color: luxuryColors.emerald },
    { value: '24/7', label: 'Uptime', icon: Clock, color: luxuryColors.amber },
    { value: '72%', label: 'AI Accuracy', icon: Brain, color: luxuryColors.purple },
  ];

  const steps = [
    {
      number: 1,
      title: 'Data Collection',
      subtitle: 'Real-time Market Data',
      icon: Database,
      color: luxuryColors.cyan,
      description: 'Continuous streaming from Binance exchange',
      details: [
        { icon: Activity, text: 'OHLC price data every second' },
        { icon: BarChart3, text: 'Volume & market depth analysis' },
        { icon: Clock, text: '1h & 4h timeframe monitoring' },
        { icon: Zap, text: 'WebSocket real-time updates' },
      ]
    },
    {
      number: 2,
      title: 'Technical Analysis',
      subtitle: '5 Optimized Strategies',
      icon: LineChart,
      color: luxuryColors.emerald,
      description: 'Multi-strategy analysis with AI enhancement',
      details: [
        { icon: TrendingUp, text: 'RSI: Overbought/Oversold (65%)' },
        { icon: BarChart3, text: 'MACD: Trend & Momentum (61%)' },
        { icon: Activity, text: 'Bollinger: Volatility (63%)' },
        { icon: Layers, text: 'Volume & Stochastic (58-64%)' },
      ]
    },
    {
      number: 3,
      title: 'Signal Generation',
      subtitle: 'AI-Powered Decisions',
      icon: Brain,
      color: luxuryColors.purple,
      description: 'Smart signal generation with multi-confirmation',
      details: [
        { icon: Target, text: 'Requires 4/5 strategy agreement' },
        { icon: Percent, text: '65-100% confidence threshold' },
        { icon: Clock, text: '60-minute signal interval' },
        { icon: Sparkles, text: 'Multi-timeframe validation' },
      ]
    },
    {
      number: 4,
      title: 'Risk Management',
      subtitle: '7 Protection Layers',
      icon: Shield,
      color: luxuryColors.rose,
      description: 'Comprehensive risk control before execution',
      details: [
        { icon: Lock, text: 'Max 1% risk per trade' },
        { icon: AlertTriangle, text: '5% stop loss mandatory' },
        { icon: Timer, text: '60min cool-down after losses' },
        { icon: TrendingDown, text: '3% daily loss limit' },
      ]
    },
  ];

  // Verified against settings.rs
  const strategies = [
    {
      name: 'RSI Strategy',
      winRate: 65,
      description: 'Relative Strength Index',
      icon: TrendingUp,
      color: luxuryColors.cyan,
      signals: { buy: 'RSI < 25', sell: 'RSI > 75' }
    },
    {
      name: 'MACD Strategy',
      winRate: 61,
      description: 'Moving Average Convergence',
      icon: BarChart3,
      color: luxuryColors.emerald,
      signals: { buy: 'MACD crosses up', sell: 'MACD crosses down' }
    },
    {
      name: 'Bollinger Bands',
      winRate: 63,
      description: 'Volatility & Breakouts',
      icon: Activity,
      color: luxuryColors.amber,
      signals: { buy: 'Touch lower band', sell: 'Touch upper band' }
    },
    {
      name: 'Volume Strategy',
      winRate: 58,
      description: 'Trend Strength Confirmation',
      icon: Layers,
      color: luxuryColors.purple,
      signals: { buy: 'Volume spike + up', sell: 'Volume spike + down' }
    },
    {
      name: 'Stochastic',
      winRate: 64,
      description: 'Momentum Oscillator',
      icon: Target,
      color: luxuryColors.rose,
      signals: { buy: '%K crosses %D < 15', sell: '%K crosses %D > 85' }
    },
  ];

  // Verified against settings.rs defaults
  const riskLayers = [
    { layer: 1, name: 'Position Risk', value: '≤1%', desc: 'Max risk per trade', icon: Percent },
    { layer: 2, name: 'Stop Loss', value: '5%', desc: 'Mandatory stop loss', icon: AlertTriangle },
    { layer: 3, name: 'Portfolio Risk', value: '≤10%', desc: 'Total exposure limit', icon: Layers },
    { layer: 4, name: 'Daily Loss', value: '3%', desc: 'Daily loss limit', icon: TrendingDown },
    { layer: 5, name: 'Consecutive Losses', value: '3 max', desc: 'Before cool-down', icon: Timer },
    { layer: 6, name: 'Cool-Down', value: '60 min', desc: 'Rest period', icon: Clock },
    { layer: 7, name: 'Correlation', value: '70%', desc: 'Position diversity', icon: Activity },
  ];

  return (
      <div className="min-h-screen" style={{ backgroundColor: luxuryColors.bgPrimary }}>
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
                  AI-POWERED TRADING BOT
                </Badge>
              </motion.div>

              <motion.h1
                variants={itemVariants}
                className="text-4xl lg:text-6xl font-black tracking-tight"
                style={{ color: luxuryColors.textPrimary }}
              >
                Smart Trading,{' '}
                <GradientText className="text-4xl lg:text-6xl font-black">
                  Zero Emotion
                </GradientText>
              </motion.h1>

              <motion.p
                variants={itemVariants}
                className="text-lg lg:text-xl max-w-2xl mx-auto"
                style={{ color: luxuryColors.textSecondary }}
              >
                Automated cryptocurrency trading with advanced AI analysis,
                multi-strategy confirmation, and comprehensive risk management.
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
                        style={{ color: luxuryColors.textMuted }}
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
          className="max-w-7xl mx-auto px-4 pb-16 space-y-16"
          variants={containerVariants}
          initial="hidden"
          animate="visible"
        >

          {/* How It Works - 4 Steps */}
          <section>
            <motion.div variants={itemVariants}>
              <SectionHeader
                title="How It Works"
                subtitle="Four-step automated trading process"
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
                        style={{ color: luxuryColors.textPrimary }}
                      >
                        {step.title}
                      </h3>
                    </div>
                  </div>
                  <p
                    className="text-xs"
                    style={{ color: luxuryColors.textSecondary }}
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
                      style={{ color: luxuryColors.textSecondary }}
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
                        backgroundColor: luxuryColors.bgSecondary,
                        border: `1px solid ${luxuryColors.borderSubtle}`,
                      }}
                    >
                      <GlowIcon
                        icon={detail.icon}
                        color={steps[activeStep].color}
                        size="sm"
                      />
                      <span
                        className="text-sm"
                        style={{ color: luxuryColors.textPrimary }}
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
                title="Trading Strategies"
                subtitle="Five optimized strategies working in harmony"
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
                    style={{ color: luxuryColors.textPrimary }}
                  >
                    {strategy.name}
                  </h3>
                  <p
                    className="text-[10px] mb-4"
                    style={{ color: luxuryColors.textMuted }}
                  >
                    {strategy.description}
                  </p>

                  <div className="flex items-center justify-between mb-3">
                    <span
                      className="text-[10px] uppercase tracking-wider"
                      style={{ color: luxuryColors.textMuted }}
                    >
                      Win Rate
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
                        style={{ backgroundColor: luxuryColors.emerald }}
                      />
                      <span style={{ color: luxuryColors.textMuted }}>Buy:</span>
                      <span style={{ color: luxuryColors.textSecondary }}>
                        {strategy.signals.buy}
                      </span>
                    </div>
                    <div className="flex items-center gap-2">
                      <div
                        className="w-1.5 h-1.5 rounded-full"
                        style={{ backgroundColor: luxuryColors.rose }}
                      />
                      <span style={{ color: luxuryColors.textMuted }}>Sell:</span>
                      <span style={{ color: luxuryColors.textSecondary }}>
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
                  borderColor: `${luxuryColors.cyan}40`,
                  backgroundColor: `${luxuryColors.cyan}08`,
                }}
              >
                <div className="flex items-start gap-3">
                  <CheckCircle className="h-5 w-5 mt-0.5" style={{ color: luxuryColors.cyan }} />
                  <div>
                    <p className="font-bold text-sm" style={{ color: luxuryColors.textPrimary }}>
                      Multi-Confirmation Required
                    </p>
                    <p className="text-xs mt-1" style={{ color: luxuryColors.textSecondary }}>
                      Trades are only executed when at least 4 out of 5 strategies agree on the signal direction,
                      ensuring high-quality trade entries.
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
                title="Risk Management"
                subtitle="Seven layers of protection for your capital"
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
                      background: luxuryColors.gradientPremium,
                    }}
                  />
                  <div className="pl-3">
                    <div className="flex items-center justify-between mb-3">
                      <Badge size="sm">LAYER {layer.layer}</Badge>
                      <GlowIcon icon={layer.icon} color={luxuryColors.emerald} size="sm" />
                    </div>
                    <h3
                      className="text-sm font-bold mb-2"
                      style={{ color: luxuryColors.textPrimary }}
                    >
                      {layer.name}
                    </h3>
                    <div
                      className="text-2xl font-black mb-1"
                      style={{ color: luxuryColors.emerald }}
                    >
                      {layer.value}
                    </div>
                    <p
                      className="text-[10px]"
                      style={{ color: luxuryColors.textMuted }}
                    >
                      {layer.desc}
                    </p>
                  </div>
                </GlassCard>
              ))}
            </motion.div>

            <motion.div
              variants={containerVariants}
              className="grid grid-cols-1 sm:grid-cols-3 gap-4 mt-4"
            >
              {riskLayers.slice(4).map((layer) => (
                <GlassCard key={layer.layer} hoverable className="relative overflow-hidden">
                  <div
                    className="absolute top-0 left-0 w-1 h-full"
                    style={{
                      background: luxuryColors.gradientPremium,
                    }}
                  />
                  <div className="pl-3 flex items-center gap-4">
                    <GlowIcon icon={layer.icon} color={luxuryColors.emerald} size="md" />
                    <div className="flex-1">
                      <div className="flex items-center justify-between mb-1">
                        <span
                          className="text-sm font-bold"
                          style={{ color: luxuryColors.textPrimary }}
                        >
                          {layer.name}
                        </span>
                        <Badge variant="success" size="sm">
                          {layer.value}
                        </Badge>
                      </div>
                      <p
                        className="text-[10px]"
                        style={{ color: luxuryColors.textMuted }}
                      >
                        {layer.desc}
                      </p>
                    </div>
                  </div>
                </GlassCard>
              ))}
            </motion.div>
          </section>

          {/* Trailing Stop Example */}
          <section>
            <motion.div variants={itemVariants}>
              <GlassCard className="overflow-hidden">
                <div
                  className="p-4 rounded-t-2xl"
                  style={{
                    background: `linear-gradient(135deg, ${luxuryColors.emerald}15, ${luxuryColors.cyan}10)`,
                  }}
                >
                  <div className="flex items-center gap-3 mb-2">
                    <GlowIcon icon={TrendingUp} color={luxuryColors.emerald} size="md" />
                    <h3 className="text-lg font-black" style={{ color: luxuryColors.textPrimary }}>
                      Trailing Stop Protection
                    </h3>
                  </div>
                  <p className="text-xs" style={{ color: luxuryColors.textSecondary }}>
                    Automatically locks in profits as price moves in your favor
                  </p>
                </div>

                <div className="p-6">
                  <div className="flex flex-wrap items-center justify-center gap-2 md:gap-4">
                    {[
                      { label: 'Entry', value: '$45,000', color: luxuryColors.textMuted, icon: Play },
                      { label: '+5% Profit', value: '$47,250', color: luxuryColors.emerald, icon: TrendingUp },
                      { label: 'Trailing Active', value: '$45,832', color: luxuryColors.amber, icon: Zap },
                      { label: 'Peak Price', value: '$48,000', color: luxuryColors.emerald, icon: TrendingUp },
                      { label: 'Exit', value: '$46,560', color: luxuryColors.emerald, icon: CheckCircle, highlight: true },
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
                          <div className="text-[10px]" style={{ color: luxuryColors.textMuted }}>
                            {step.label}
                          </div>
                        </motion.div>
                        {idx < 4 && (
                          <ChevronRight
                            className="h-5 w-5 hidden md:block"
                            style={{ color: luxuryColors.textMuted }}
                          />
                        )}
                      </Fragment>
                    ))}
                  </div>

                  <div
                    className="mt-6 p-4 rounded-xl"
                    style={{
                      backgroundColor: `${luxuryColors.emerald}15`,
                      border: `1px solid ${luxuryColors.emerald}30`,
                    }}
                  >
                    <div className="flex items-center gap-2">
                      <CheckCircle className="h-5 w-5" style={{ color: luxuryColors.emerald }} />
                      <span
                        className="font-bold text-sm"
                        style={{ color: luxuryColors.emerald }}
                      >
                        Result: +3.47% profit ($1,560) protected even when price dropped from peak
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
