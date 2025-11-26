/**
 * How It Works Page - Professional Redesign
 *
 * Trang giải thích cách bot hoạt động với UI chuyên nghiệp
 * Verified data against rust-core-engine/src/paper_trading/settings.rs
 */

import React, { useState } from 'react';
import { Link } from 'react-router-dom';
import ErrorBoundary from '@/components/ErrorBoundary';
import { DashboardHeader } from '@/components/dashboard/DashboardHeader';
import { useAuth } from '@/contexts/AuthContext';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Separator } from '@/components/ui/separator';
import {
  Database,
  TrendingUp,
  Brain,
  Shield,
  CheckCircle,
  ArrowRight,
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
  const { user } = useAuth();
  const [activeStep, setActiveStep] = useState(0);

  // Verified against rust-core-engine settings
  const stats = [
    { value: '5', label: 'Strategies', icon: <BarChart3 className="h-5 w-5" /> },
    { value: '7', label: 'Risk Layers', icon: <Shield className="h-5 w-5" /> },
    { value: '24/7', label: 'Uptime', icon: <Clock className="h-5 w-5" /> },
    { value: '72%', label: 'AI Accuracy', icon: <Brain className="h-5 w-5" /> },
  ];

  const steps = [
    {
      number: 1,
      title: 'Data Collection',
      subtitle: 'Real-time Market Data',
      icon: <Database className="h-6 w-6" />,
      color: 'from-blue-500 to-cyan-500',
      bgColor: 'bg-blue-500/10',
      borderColor: 'border-blue-500/20',
      description: 'Continuous streaming from Binance exchange',
      details: [
        { icon: <Activity className="h-4 w-4" />, text: 'OHLC price data every second' },
        { icon: <BarChart3 className="h-4 w-4" />, text: 'Volume & market depth analysis' },
        { icon: <Clock className="h-4 w-4" />, text: '1h & 4h timeframe monitoring' },
        { icon: <Zap className="h-4 w-4" />, text: 'WebSocket real-time updates' },
      ]
    },
    {
      number: 2,
      title: 'Technical Analysis',
      subtitle: '5 Optimized Strategies',
      icon: <LineChart className="h-6 w-6" />,
      color: 'from-emerald-500 to-green-500',
      bgColor: 'bg-emerald-500/10',
      borderColor: 'border-emerald-500/20',
      description: 'Multi-strategy analysis with AI enhancement',
      details: [
        { icon: <TrendingUp className="h-4 w-4" />, text: 'RSI: Overbought/Oversold (65%)' },
        { icon: <BarChart3 className="h-4 w-4" />, text: 'MACD: Trend & Momentum (61%)' },
        { icon: <Activity className="h-4 w-4" />, text: 'Bollinger: Volatility (63%)' },
        { icon: <Layers className="h-4 w-4" />, text: 'Volume & Stochastic (58-64%)' },
      ]
    },
    {
      number: 3,
      title: 'Signal Generation',
      subtitle: 'AI-Powered Decisions',
      icon: <Brain className="h-6 w-6" />,
      color: 'from-violet-500 to-purple-500',
      bgColor: 'bg-violet-500/10',
      borderColor: 'border-violet-500/20',
      description: 'Smart signal generation with multi-confirmation',
      details: [
        { icon: <Target className="h-4 w-4" />, text: 'Requires 4/5 strategy agreement' },
        { icon: <Percent className="h-4 w-4" />, text: '65-100% confidence threshold' },
        { icon: <Clock className="h-4 w-4" />, text: '60-minute signal interval' },
        { icon: <Sparkles className="h-4 w-4" />, text: 'Multi-timeframe validation' },
      ]
    },
    {
      number: 4,
      title: 'Risk Management',
      subtitle: '7 Protection Layers',
      icon: <Shield className="h-6 w-6" />,
      color: 'from-rose-500 to-red-500',
      bgColor: 'bg-rose-500/10',
      borderColor: 'border-rose-500/20',
      description: 'Comprehensive risk control before execution',
      details: [
        { icon: <Lock className="h-4 w-4" />, text: 'Max 1% risk per trade' },
        { icon: <AlertTriangle className="h-4 w-4" />, text: '5% stop loss mandatory' },
        { icon: <Timer className="h-4 w-4" />, text: '60min cool-down after losses' },
        { icon: <TrendingDown className="h-4 w-4" />, text: '3% daily loss limit' },
      ]
    },
  ];

  // Verified against settings.rs
  const strategies = [
    {
      name: 'RSI Strategy',
      winRate: 65,
      description: 'Relative Strength Index',
      icon: <TrendingUp className="h-5 w-5" />,
      color: 'text-blue-500',
      bgColor: 'bg-blue-500/10',
      signals: { buy: 'RSI < 25', sell: 'RSI > 75' }
    },
    {
      name: 'MACD Strategy',
      winRate: 61,
      description: 'Moving Average Convergence',
      icon: <BarChart3 className="h-5 w-5" />,
      color: 'text-emerald-500',
      bgColor: 'bg-emerald-500/10',
      signals: { buy: 'MACD crosses up', sell: 'MACD crosses down' }
    },
    {
      name: 'Bollinger Bands',
      winRate: 63,
      description: 'Volatility & Breakouts',
      icon: <Activity className="h-5 w-5" />,
      color: 'text-orange-500',
      bgColor: 'bg-orange-500/10',
      signals: { buy: 'Touch lower band', sell: 'Touch upper band' }
    },
    {
      name: 'Volume Strategy',
      winRate: 58,
      description: 'Trend Strength Confirmation',
      icon: <Layers className="h-5 w-5" />,
      color: 'text-purple-500',
      bgColor: 'bg-purple-500/10',
      signals: { buy: 'Volume spike + up', sell: 'Volume spike + down' }
    },
    {
      name: 'Stochastic',
      winRate: 64,
      description: 'Momentum Oscillator',
      icon: <Target className="h-5 w-5" />,
      color: 'text-pink-500',
      bgColor: 'bg-pink-500/10',
      signals: { buy: '%K crosses %D < 15', sell: '%K crosses %D > 85' }
    },
  ];

  // Verified against settings.rs defaults
  const riskLayers = [
    { layer: 1, name: 'Position Risk', value: '≤1%', desc: 'Max risk per trade', icon: <Percent className="h-4 w-4" /> },
    { layer: 2, name: 'Stop Loss', value: '5%', desc: 'Mandatory stop loss', icon: <AlertTriangle className="h-4 w-4" /> },
    { layer: 3, name: 'Portfolio Risk', value: '≤10%', desc: 'Total exposure limit', icon: <Layers className="h-4 w-4" /> },
    { layer: 4, name: 'Daily Loss', value: '3%', desc: 'Daily loss limit', icon: <TrendingDown className="h-4 w-4" /> },
    { layer: 5, name: 'Consecutive Losses', value: '3 max', desc: 'Before cool-down', icon: <Timer className="h-4 w-4" /> },
    { layer: 6, name: 'Cool-Down', value: '60 min', desc: 'Rest period', icon: <Clock className="h-4 w-4" /> },
    { layer: 7, name: 'Correlation', value: '70%', desc: 'Position diversity', icon: <Activity className="h-4 w-4" /> },
  ];

  return (
    <ErrorBoundary>
      <div className="min-h-screen bg-background">
        {user && <DashboardHeader />}

        {/* Hero Section */}
        <section className="relative overflow-hidden">
          <div className="absolute inset-0 bg-gradient-to-br from-primary/5 via-background to-accent/5" />
          <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-primary/10 via-transparent to-transparent" />

          <div className="relative max-w-7xl mx-auto px-4 py-16 lg:py-24">
            <div className="text-center space-y-6">
              <Badge variant="outline" className="px-4 py-1.5 text-sm font-medium">
                <Sparkles className="h-3.5 w-3.5 mr-2" />
                AI-Powered Trading Bot
              </Badge>

              <h1 className="text-4xl lg:text-6xl font-bold tracking-tight">
                Smart Trading,{' '}
                <span className="bg-gradient-to-r from-primary to-accent bg-clip-text text-transparent">
                  Zero Emotion
                </span>
              </h1>

              <p className="text-xl text-muted-foreground max-w-2xl mx-auto">
                Automated cryptocurrency trading with advanced AI analysis,
                multi-strategy confirmation, and comprehensive risk management.
              </p>

              {/* Stats */}
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4 max-w-3xl mx-auto pt-8">
                {stats.map((stat, index) => (
                  <Card key={index} className="bg-card/50 backdrop-blur border-border/50">
                    <CardContent className="p-4 text-center">
                      <div className="flex justify-center mb-2 text-primary">{stat.icon}</div>
                      <div className="text-2xl font-bold">{stat.value}</div>
                      <div className="text-xs text-muted-foreground">{stat.label}</div>
                    </CardContent>
                  </Card>
                ))}
              </div>

              {!user && (
                <div className="pt-6">
                  <Link to="/register">
                    <Button size="lg" className="gap-2">
                      Start Paper Trading
                      <ArrowRight className="h-4 w-4" />
                    </Button>
                  </Link>
                  <p className="text-sm text-muted-foreground mt-3">
                    Free $10,000 virtual balance • No credit card required
                  </p>
                </div>
              )}
            </div>
          </div>
        </section>

        <div className="max-w-7xl mx-auto px-4 pb-16 space-y-16">

          {/* How It Works - 4 Steps */}
          <section>
            <div className="text-center mb-12">
              <h2 className="text-3xl font-bold mb-3">How It Works</h2>
              <p className="text-muted-foreground">Four-step automated trading process</p>
            </div>

            {/* Step Cards */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
              {steps.map((step, index) => (
                <Card
                  key={step.number}
                  className={`cursor-pointer transition-all duration-300 hover:shadow-lg ${
                    activeStep === index
                      ? `ring-2 ring-primary shadow-lg ${step.bgColor}`
                      : 'hover:border-primary/50'
                  }`}
                  onClick={() => setActiveStep(index)}
                >
                  <CardHeader className="pb-3">
                    <div className="flex items-center gap-3">
                      <div className={`p-2.5 rounded-xl bg-gradient-to-br ${step.color} text-white`}>
                        {step.icon}
                      </div>
                      <div>
                        <Badge variant="secondary" className="text-xs mb-1">Step {step.number}</Badge>
                        <CardTitle className="text-base">{step.title}</CardTitle>
                      </div>
                    </div>
                  </CardHeader>
                  <CardContent>
                    <p className="text-sm text-muted-foreground">{step.subtitle}</p>
                  </CardContent>
                </Card>
              ))}
            </div>

            {/* Step Details */}
            <Card className={`${steps[activeStep].bgColor} ${steps[activeStep].borderColor} border-2`}>
              <CardHeader>
                <div className="flex items-center gap-4">
                  <div className={`p-3 rounded-xl bg-gradient-to-br ${steps[activeStep].color} text-white`}>
                    {steps[activeStep].icon}
                  </div>
                  <div>
                    <CardTitle className="text-xl">
                      Step {steps[activeStep].number}: {steps[activeStep].title}
                    </CardTitle>
                    <CardDescription className="text-base mt-1">
                      {steps[activeStep].description}
                    </CardDescription>
                  </div>
                </div>
              </CardHeader>
              <CardContent>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                  {steps[activeStep].details.map((detail, idx) => (
                    <div key={idx} className="flex items-center gap-3 p-3 rounded-lg bg-background/50">
                      <div className="text-primary">{detail.icon}</div>
                      <span className="text-sm">{detail.text}</span>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          </section>

          {/* Trading Strategies */}
          <section>
            <div className="text-center mb-12">
              <h2 className="text-3xl font-bold mb-3">Trading Strategies</h2>
              <p className="text-muted-foreground">Five optimized strategies working in harmony</p>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4">
              {strategies.map((strategy) => (
                <Card key={strategy.name} className="group hover:shadow-lg transition-all duration-300">
                  <CardHeader className="pb-3">
                    <div className={`w-12 h-12 rounded-xl ${strategy.bgColor} flex items-center justify-center mb-3 group-hover:scale-110 transition-transform`}>
                      <div className={strategy.color}>{strategy.icon}</div>
                    </div>
                    <CardTitle className="text-base">{strategy.name}</CardTitle>
                    <CardDescription className="text-xs">{strategy.description}</CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-3">
                    <div className="flex items-center justify-between">
                      <span className="text-xs text-muted-foreground">Win Rate</span>
                      <Badge variant="secondary" className="font-mono">{strategy.winRate}%</Badge>
                    </div>
                    <Separator />
                    <div className="space-y-2 text-xs">
                      <div className="flex items-center gap-2">
                        <div className="w-1.5 h-1.5 rounded-full bg-emerald-500" />
                        <span className="text-muted-foreground">Buy:</span>
                        <span>{strategy.signals.buy}</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <div className="w-1.5 h-1.5 rounded-full bg-rose-500" />
                        <span className="text-muted-foreground">Sell:</span>
                        <span>{strategy.signals.sell}</span>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>

            {/* Strategy Note */}
            <Card className="mt-6 bg-primary/5 border-primary/20">
              <CardContent className="p-4">
                <div className="flex items-start gap-3">
                  <CheckCircle className="h-5 w-5 text-primary mt-0.5" />
                  <div>
                    <p className="font-medium">Multi-Confirmation Required</p>
                    <p className="text-sm text-muted-foreground">
                      Trades are only executed when at least 4 out of 5 strategies agree on the signal direction,
                      ensuring high-quality trade entries.
                    </p>
                  </div>
                </div>
              </CardContent>
            </Card>
          </section>

          {/* Risk Management */}
          <section>
            <div className="text-center mb-12">
              <h2 className="text-3xl font-bold mb-3">Risk Management</h2>
              <p className="text-muted-foreground">Seven layers of protection for your capital</p>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
              {riskLayers.slice(0, 4).map((layer) => (
                <Card key={layer.layer} className="relative overflow-hidden group">
                  <div className="absolute top-0 left-0 w-1 h-full bg-gradient-to-b from-primary to-accent" />
                  <CardHeader className="pb-2">
                    <div className="flex items-center justify-between">
                      <Badge variant="outline" className="text-xs">Layer {layer.layer}</Badge>
                      <div className="text-primary">{layer.icon}</div>
                    </div>
                    <CardTitle className="text-base">{layer.name}</CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="text-2xl font-bold text-primary mb-1">{layer.value}</div>
                    <p className="text-xs text-muted-foreground">{layer.desc}</p>
                  </CardContent>
                </Card>
              ))}
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-3 gap-4 mt-4">
              {riskLayers.slice(4).map((layer) => (
                <Card key={layer.layer} className="relative overflow-hidden">
                  <div className="absolute top-0 left-0 w-1 h-full bg-gradient-to-b from-primary to-accent" />
                  <CardContent className="p-4">
                    <div className="flex items-center gap-4">
                      <div className="text-primary">{layer.icon}</div>
                      <div className="flex-1">
                        <div className="flex items-center justify-between">
                          <span className="text-sm font-medium">{layer.name}</span>
                          <Badge variant="secondary" className="font-mono">{layer.value}</Badge>
                        </div>
                        <p className="text-xs text-muted-foreground">{layer.desc}</p>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          </section>

          {/* Trailing Stop Example */}
          <section>
            <Card className="overflow-hidden">
              <CardHeader className="bg-gradient-to-r from-emerald-500/10 to-green-500/10">
                <CardTitle className="flex items-center gap-2">
                  <TrendingUp className="h-5 w-5 text-emerald-500" />
                  Trailing Stop Protection
                </CardTitle>
                <CardDescription>
                  Automatically locks in profits as price moves in your favor
                </CardDescription>
              </CardHeader>
              <CardContent className="p-6">
                <div className="flex flex-wrap items-center justify-center gap-2 md:gap-4">
                  {[
                    { label: 'Entry', value: '$45,000', color: 'bg-muted', icon: <Play className="h-4 w-4" /> },
                    { label: '+5% Profit', value: '$47,250', color: 'bg-emerald-500/10 text-emerald-600', icon: <TrendingUp className="h-4 w-4" /> },
                    { label: 'Trailing Active', value: '$45,832', color: 'bg-amber-500/10 text-amber-600', icon: <Zap className="h-4 w-4" /> },
                    { label: 'Peak Price', value: '$48,000', color: 'bg-emerald-500/10 text-emerald-600', icon: <TrendingUp className="h-4 w-4" /> },
                    { label: 'Exit', value: '$46,560', color: 'bg-emerald-500/20 text-emerald-600 ring-2 ring-emerald-500', icon: <CheckCircle className="h-4 w-4" /> },
                  ].map((step, idx) => (
                    <React.Fragment key={idx}>
                      <div className={`p-4 rounded-xl text-center min-w-[100px] ${step.color}`}>
                        <div className="flex justify-center mb-2">{step.icon}</div>
                        <div className="text-lg font-bold">{step.value}</div>
                        <div className="text-xs opacity-80">{step.label}</div>
                      </div>
                      {idx < 4 && <ChevronRight className="h-5 w-5 text-muted-foreground hidden md:block" />}
                    </React.Fragment>
                  ))}
                </div>
                <div className="mt-6 p-4 rounded-lg bg-emerald-500/10 border border-emerald-500/20">
                  <div className="flex items-center gap-2 text-emerald-600">
                    <CheckCircle className="h-5 w-5" />
                    <span className="font-medium">
                      Result: +3.47% profit ($1,560) protected even when price dropped from peak
                    </span>
                  </div>
                </div>
              </CardContent>
            </Card>
          </section>

          {/* CTA Section */}
          {!user && (
            <section>
              <Card className="bg-gradient-to-br from-primary to-accent text-primary-foreground overflow-hidden relative">
                <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_bottom_right,_var(--tw-gradient-stops))] from-white/10 via-transparent to-transparent" />
                <CardContent className="relative p-8 md:p-12 text-center">
                  <h2 className="text-3xl font-bold mb-4">Ready to Start?</h2>
                  <p className="text-lg opacity-90 max-w-xl mx-auto mb-8">
                    Try paper trading with $10,000 virtual balance.
                    Real market data, zero risk.
                  </p>
                  <div className="flex flex-wrap justify-center gap-3 mb-6">
                    <Badge variant="secondary" className="bg-white/20 text-white border-white/30">
                      <CheckCircle className="h-3.5 w-3.5 mr-1.5" />
                      100% Free
                    </Badge>
                    <Badge variant="secondary" className="bg-white/20 text-white border-white/30">
                      <CheckCircle className="h-3.5 w-3.5 mr-1.5" />
                      No KYC Required
                    </Badge>
                    <Badge variant="secondary" className="bg-white/20 text-white border-white/30">
                      <CheckCircle className="h-3.5 w-3.5 mr-1.5" />
                      Real Binance Data
                    </Badge>
                  </div>
                  <Link to="/register">
                    <Button size="lg" variant="secondary" className="gap-2">
                      Create Free Account
                      <ArrowRight className="h-4 w-4" />
                    </Button>
                  </Link>
                </CardContent>
              </Card>
            </section>
          )}
        </div>
      </div>
    </ErrorBoundary>
  );
};

export default HowItWorks;
