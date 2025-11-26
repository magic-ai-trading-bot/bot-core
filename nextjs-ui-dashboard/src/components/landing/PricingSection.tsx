import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Check, Crown, Zap, Rocket } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useState } from "react";
import { toast } from "sonner";

const plans = [
  {
    name: "Basic",
    price: 20,
    icon: Zap,
    description: "Perfect for beginners starting their crypto trading journey",
    badge: "Popular",
    features: [
      "AI-powered trading signals",
      "Basic risk management",
      "3 trading pairs",
      "Email support",
      "Mobile dashboard",
      "Weekly performance reports"
    ],
    limitations: [
      "Limited to $1,000 trading capital",
      "Standard execution speed",
      "Basic analytics"
    ]
  },
  {
    name: "Premium",
    price: 50,
    icon: Crown,
    description: "Advanced features for serious crypto traders",
    badge: "Best Value",
    featured: true,
    features: [
      "Advanced AI analysis",
      "Smart position sizing",
      "10+ trading pairs", 
      "Priority support",
      "Advanced analytics",
      "Custom risk parameters",
      "Real-time notifications",
      "Daily market insights"
    ],
    limitations: [
      "Up to $10,000 trading capital",
      "Enhanced execution speed"
    ]
  },
  {
    name: "Enterprise",
    price: 100,
    icon: Rocket,
    description: "Professional-grade solution for institutional traders",
    badge: "Pro",
    features: [
      "Full AI trading suite",
      "Unlimited trading pairs",
      "Custom strategies",
      "24/7 dedicated support",
      "Advanced portfolio management", 
      "API access",
      "White-label options",
      "Custom integrations",
      "Risk analytics dashboard",
      "Institutional reporting"
    ],
    limitations: [
      "Unlimited trading capital",
      "Ultra-fast execution",
      "Custom SLA available"
    ]
  }
];

export function PricingSection() {
  const { t } = useTranslation();
  const [isProcessing, setIsProcessing] = useState<string | null>(null);

  const handleSubscribe = async (planName: string, price: number) => {
    setIsProcessing(planName);

    try {
      // Simulate payment processing
      await new Promise(resolve => setTimeout(resolve, 2000));

      // Show success toast
      toast.success(`${planName} Plan Selected!`, {
        description: `$${price}/month - You will receive an email with registration instructions.`,
        duration: 5000,
      });
    } catch {
      toast.error("Payment Failed", {
        description: "Please try again or contact support.",
        duration: 4000,
      });
    } finally {
      setIsProcessing(null);
    }
  };

  return (
    <section className="py-24 bg-gradient-to-b from-card/20 to-background">
      <div className="container mx-auto px-4">
        <div className="text-center mb-16">
          <Badge variant="outline" className="mb-4 bg-profit/10 text-profit border-profit/20">
            <Crown className="w-3 h-3 mr-1" />
            {t('pricing.badge')}
          </Badge>
          <h2 className="text-3xl md:text-5xl font-bold mb-6">
            {t('pricing.title')} <span className="text-profit">{t('pricing.subtitle')}</span>
          </h2>
          <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
            {t('pricing.description')}
          </p>
        </div>
        
        <div className="grid grid-cols-1 md:grid-cols-3 gap-8 max-w-6xl mx-auto">
          {plans.map((plan, index) => (
            <Card 
              key={index} 
              className={`relative group transition-all duration-300 ${
                plan.featured 
                  ? 'border-profit shadow-lg scale-105 bg-gradient-to-b from-card to-profit/5' 
                  : 'border-border/50 hover:border-primary/50 bg-card/50 backdrop-blur'
              }`}
            >
              {plan.featured && (
                <div className="absolute -top-3 left-1/2 transform -translate-x-1/2">
                  <Badge className="bg-profit text-profit-foreground">
                    {t('pricing.mostPopular')}
                  </Badge>
                </div>
              )}
              
              <CardHeader className="text-center pb-4">
                <div className="flex items-center justify-center gap-2 mb-2">
                  <div className={`p-2 rounded-lg ${plan.featured ? 'bg-profit/20' : 'bg-primary/10'}`}>
                    <plan.icon className={`w-6 h-6 ${plan.featured ? 'text-profit' : 'text-primary'}`} />
                  </div>
                  <Badge variant="secondary" className="text-xs">
                    {plan.badge}
                  </Badge>
                </div>
                <CardTitle className="text-2xl mb-2">{plan.name}</CardTitle>
                <div className="mb-4">
                  <span className="text-4xl font-bold">${plan.price}</span>
                  <span className="text-muted-foreground">/month</span>
                </div>
                <p className="text-sm text-muted-foreground">
                  {plan.description}
                </p>
              </CardHeader>
              
              <CardContent className="space-y-6">
                <Button 
                  className={`w-full ${
                    plan.featured 
                      ? 'bg-profit hover:bg-profit/90 text-profit-foreground' 
                      : 'bg-primary hover:bg-primary/90'
                  }`}
                  size="lg"
                  onClick={() => handleSubscribe(plan.name, plan.price)}
                  disabled={isProcessing === plan.name}
                >
                  {isProcessing === plan.name ? 'Processing...' : t('pricing.getStarted')}
                </Button>
                
                <div className="space-y-3">
                  <h4 className="font-semibold text-sm">{t('pricing.features')}</h4>
                  <ul className="space-y-2">
                    {plan.features.map((feature, featureIndex) => (
                      <li key={featureIndex} className="flex items-start gap-2 text-sm">
                        <Check className="w-4 h-4 text-profit mt-0.5 flex-shrink-0" />
                        <span>{feature}</span>
                      </li>
                    ))}
                  </ul>
                </div>
                
                <div className="space-y-2 pt-4 border-t border-border/50">
                  <h4 className="font-semibold text-sm text-muted-foreground">{t('pricing.limits')}</h4>
                  <ul className="space-y-1">
                    {plan.limitations.map((limitation, limitIndex) => (
                      <li key={limitIndex} className="text-xs text-muted-foreground">
                        • {limitation}
                      </li>
                    ))}
                  </ul>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
        
        <div className="text-center mt-12">
          <p className="text-sm text-muted-foreground mb-4">
            {t('pricing.trial')}
          </p>
          <div className="flex flex-wrap justify-center gap-4 text-xs text-muted-foreground">
            <span>🔒 Secure payments via Stripe & PayPal</span>
            <span>📊 Real-time performance tracking</span>
            <span>🌍 24/7 global market access</span>
          </div>
        </div>
      </div>
    </section>
  );
}