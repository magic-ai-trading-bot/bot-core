import { useState, useEffect } from "react";
import { PremiumButton } from "@/styles/luxury-design-system";
import { Card, CardContent } from "@/components/ui/card";
import { X, ChevronLeft, ChevronRight } from "lucide-react";

interface TourStep {
  title: string;
  description: string;
  target?: string;
  position?: "top" | "bottom" | "left" | "right";
}

const tourSteps: TourStep[] = [
  {
    title: "Welcome to Bot Core Trading Dashboard! 🎉",
    description:
      "Let's take a quick tour to help you get started with AI-powered cryptocurrency trading. This tour will show you the main features.",
  },
  {
    title: "Trading Charts 📊",
    description:
      "Monitor real-time price movements with beautiful candlestick charts. Add multiple symbols to track different cryptocurrencies simultaneously.",
    target: "[data-tour='trading-charts']",
  },
  {
    title: "AI Signals 🤖",
    description:
      "Get intelligent trading signals powered by advanced AI models (LSTM, GRU, Transformer, GPT-4). Each signal includes confidence scores and explanations.",
    target: "[data-tour='ai-signals']",
  },
  {
    title: "Bot Status & Positions 💰",
    description:
      "View your account balance, open positions, and real-time PnL. Monitor leverage, entry prices, and position sizes at a glance.",
    target: "[data-tour='bot-status']",
  },
  {
    title: "Strategy Education 📚",
    description:
      "Learn about trading strategies like RSI, MACD, Bollinger Bands, and Volume analysis. Click any strategy to see interactive visualizations!",
    target: "[data-tour='strategy-education']",
  },
  {
    title: "Paper Trading 📝",
    description:
      "Practice trading with virtual funds before risking real money. Test your strategies, track performance, and learn risk-free!",
  },
  {
    title: "You're All Set! 🚀",
    description:
      "You're ready to start trading! Remember to enable testnet mode in Settings before connecting to Binance. Happy trading!",
  },
];

export function ProductTour() {
  // Lazy initialization - check localStorage only once on mount
  const [isOpen, setIsOpen] = useState(() => {
    const hasSeenTour = localStorage.getItem("hasSeenProductTour");
    return !hasSeenTour;
  });
  const [currentStep, setCurrentStep] = useState(0);

  const handleComplete = () => {
    localStorage.setItem("hasSeenProductTour", "true");
    setIsOpen(false);
    setCurrentStep(0);
  };

  const handleNext = () => {
    if (currentStep < tourSteps.length - 1) {
      setCurrentStep(currentStep + 1);
    } else {
      handleComplete();
    }
  };

  const handlePrev = () => {
    if (currentStep > 0) {
      setCurrentStep(currentStep - 1);
    }
  };

  const handleSkip = () => {
    handleComplete();
  };

  if (!isOpen) return null;

  const step = tourSteps[currentStep];
  const progress = ((currentStep + 1) / tourSteps.length) * 100;

  return (
    <>
      {/* Backdrop */}
      <div
        className="fixed inset-0 bg-background/80 backdrop-blur-sm z-40 animate-in fade-in"
        onClick={handleSkip}
        aria-hidden="true"
      />

      {/* Tour Card */}
      <Card
        className="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 z-50 w-[90%] max-w-lg shadow-2xl animate-in zoom-in-95"
        role="dialog"
        aria-labelledby="tour-title"
        aria-describedby="tour-description"
      >
        <CardContent className="p-6">
          {/* Close Button */}
          <PremiumButton
            variant="ghost"
            size="sm"
            className="absolute top-2 right-2 h-8 w-8 p-0 focus-custom"
            onClick={handleSkip}
            aria-label="Close tour"
          >
            <X className="h-4 w-4" />
          </PremiumButton>

          {/* Progress Bar */}
          <div className="mb-6">
            <div className="flex justify-between items-center mb-2">
              <span className="text-xs text-muted-foreground">
                Step {currentStep + 1} of {tourSteps.length}
              </span>
              <span className="text-xs text-muted-foreground">
                {Math.round(progress)}% complete
              </span>
            </div>
            <div className="h-2 bg-secondary rounded-full overflow-hidden">
              <div
                className="h-full bg-profit transition-all duration-300"
                style={{ width: `${progress}%` }}
                role="progressbar"
                aria-valuenow={progress}
                aria-valuemin={0}
                aria-valuemax={100}
                aria-label="Tour progress"
              />
            </div>
          </div>

          {/* Content */}
          <div className="space-y-4">
            <h2 id="tour-title" className="text-2xl font-bold">
              {step.title}
            </h2>
            <p id="tour-description" className="text-muted-foreground leading-relaxed">
              {step.description}
            </p>
          </div>

          {/* Navigation */}
          <div className="flex items-center justify-between mt-8">
            <PremiumButton
              variant="secondary"
              onClick={handlePrev}
              disabled={currentStep === 0}
              className="focus-custom"
            >
              <ChevronLeft className="h-4 w-4 mr-1" aria-hidden="true" />
              Previous
            </PremiumButton>

            <div className="flex gap-2">
              <PremiumButton
                variant="ghost"
                onClick={handleSkip}
                className="focus-custom"
              >
                Skip Tour
              </PremiumButton>
              <PremiumButton
                onClick={handleNext}
                className="focus-custom"
              >
                {currentStep === tourSteps.length - 1 ? (
                  "Get Started"
                ) : (
                  <>
                    Next
                    <ChevronRight className="h-4 w-4 ml-1" aria-hidden="true" />
                  </>
                )}
              </PremiumButton>
            </div>
          </div>
        </CardContent>
      </Card>
    </>
  );
}
