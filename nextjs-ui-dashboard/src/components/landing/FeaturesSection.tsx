import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Brain, TrendingUp, Shield, Zap, BarChart3, Clock, Target, Globe } from "lucide-react";

const features = [
  {
    icon: Brain,
    title: "Advanced AI Analysis",
    description: "Our neural networks analyze thousands of market indicators, news sentiment, and technical patterns in real-time to predict market movements with unprecedented accuracy.",
    badge: "AI Core"
  },
  {
    icon: TrendingUp,
    title: "Smart Position Management",
    description: "Intelligent entry and exit strategies with dynamic stop-loss and take-profit levels that adapt to market volatility and maximize your profit potential.",
    badge: "Trading"
  },
  {
    icon: Shield,
    title: "Advanced Risk Control",
    description: "Multi-layered risk management system with position sizing, drawdown protection, and emergency stop mechanisms to safeguard your capital.",
    badge: "Safety"
  },
  {
    icon: Zap,
    title: "Lightning Fast Execution",
    description: "Sub-millisecond order execution directly connected to Binance APIs ensures you never miss profitable opportunities in volatile markets.",
    badge: "Speed"
  },
  {
    icon: BarChart3,
    title: "Real-time Analytics",
    description: "Comprehensive dashboard with live P&L tracking, performance metrics, and detailed trade analysis to optimize your trading strategy.",
    badge: "Analytics"
  },
  {
    icon: Clock,
    title: "24/7 Market Monitoring",
    description: "Never sleep on opportunities. Our AI monitors global crypto markets around the clock, executing trades even when you're away.",
    badge: "Automation"
  },
  {
    icon: Target,
    title: "Precision Trading Signals",
    description: "High-confidence trading signals with detailed entry points, leverage recommendations, and risk assessments for each trade.",
    badge: "Signals"
  },
  {
    icon: Globe,
    title: "Multi-Asset Support",
    description: "Trade across major cryptocurrency pairs including BTC, ETH, BNB, and 50+ altcoins with customizable strategies for each asset.",
    badge: "Assets"
  }
];

export function FeaturesSection() {
  return (
    <section className="py-24 bg-gradient-to-b from-background to-card/20">
      <div className="container mx-auto px-4">
        <div className="text-center mb-16">
          <Badge variant="outline" className="mb-4 bg-primary/10 text-primary border-primary/20">
            <Brain className="w-3 h-3 mr-1" />
            Advanced Features
          </Badge>
          <h2 className="text-3xl md:text-5xl font-bold mb-6">
            Built for <span className="text-profit">Professional Traders</span>
          </h2>
          <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
            Experience the most sophisticated crypto trading platform powered by cutting-edge AI technology
          </p>
        </div>
        
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          {features.map((feature, index) => (
            <Card key={index} className="group hover:shadow-lg transition-all duration-300 border-border/50 bg-card/50 backdrop-blur hover:bg-card/80">
              <CardContent className="p-6">
                <div className="flex items-start gap-4 mb-4">
                  <div className="p-2 rounded-lg bg-primary/10 group-hover:bg-primary/20 transition-colors">
                    <feature.icon className="w-5 h-5 text-primary" />
                  </div>
                  <Badge variant="secondary" className="text-xs">
                    {feature.badge}
                  </Badge>
                </div>
                <h3 className="text-lg font-semibold mb-3 group-hover:text-primary transition-colors">
                  {feature.title}
                </h3>
                <p className="text-sm text-muted-foreground leading-relaxed">
                  {feature.description}
                </p>
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    </section>
  );
}