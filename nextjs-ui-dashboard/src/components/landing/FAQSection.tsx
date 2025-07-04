import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "@/components/ui/accordion";
import { Badge } from "@/components/ui/badge";
import { HelpCircle } from "lucide-react";

const faqs = [
  {
    question: "How does the AI trading bot work?",
    answer: "Our AI analyzes thousands of market indicators, news sentiment, technical patterns, and historical data in real-time. It uses advanced machine learning algorithms to identify profitable trading opportunities and automatically executes trades on your behalf through secure API connections to Binance Futures."
  },
  {
    question: "Is my money safe with automated trading?",
    answer: "Yes, your funds remain in your own Binance account at all times. Our bot only has trading permissions through API keys - it cannot withdraw funds. We also implement multiple safety mechanisms including stop-loss limits, maximum daily loss caps, and emergency shutdown features."
  },
  {
    question: "What's the minimum capital required to start?",
    answer: "You can start with as little as $100 on the Basic plan (though we recommend $500+ for better risk management). Premium plans support up to $10,000, and Enterprise has no capital limits. The bot automatically adjusts position sizes based on your account balance and risk settings."
  },
  {
    question: "Can I customize the trading strategy?",
    answer: "Absolutely! Premium and Enterprise plans offer extensive customization options including risk tolerance levels, preferred trading pairs, maximum leverage settings, and specific market conditions to trade in. You maintain full control over your trading parameters."
  },
  {
    question: "How accurate are the AI predictions?",
    answer: "Our AI maintains an average accuracy rate of 73-78% across different market conditions. While no trading system is 100% accurate, our advanced risk management ensures profitable trades outweigh losses. Past performance results are available in your dashboard."
  },
  {
    question: "Do I need trading experience to use this?",
    answer: "No prior experience required! The Basic plan is designed for beginners with pre-configured safe settings. However, we recommend learning basic crypto trading concepts to better understand and optimize your bot's performance. We provide educational resources and support."
  },
  {
    question: "What trading pairs are supported?",
    answer: "We support 50+ major cryptocurrency futures pairs including BTC/USDT, ETH/USDT, BNB/USDT, ADA/USDT, and many more. The number of available pairs depends on your plan - Basic (3 pairs), Premium (10+ pairs), Enterprise (unlimited)."
  },
  {
    question: "How do I get started after purchasing?",
    answer: "After subscription purchase, you'll receive an email with registration instructions. You'll then connect your Binance account via secure API keys, configure your risk settings, and the bot will start analyzing markets immediately. Full setup typically takes 10-15 minutes."
  },
  {
    question: "What payment methods do you accept?",
    answer: "We accept all major credit cards, PayPal, and cryptocurrency payments through our secure Stripe integration. All transactions are encrypted and PCI-compliant. Subscriptions are billed monthly with automatic renewal (can be cancelled anytime)."
  },
  {
    question: "Is there a money-back guarantee?",
    answer: "Yes! We offer a 7-day free trial and 30-day money-back guarantee. If you're not satisfied with the bot's performance within 30 days, we'll provide a full refund. We're confident in our AI's capabilities and want you to trade with complete peace of mind."
  }
];

export function FAQSection() {
  return (
    <section className="py-24 bg-gradient-to-b from-card/20 to-background">
      <div className="container mx-auto px-4 max-w-4xl">
        <div className="text-center mb-16">
          <Badge variant="outline" className="mb-4 bg-info/10 text-info border-info/20">
            <HelpCircle className="w-3 h-3 mr-1" />
            Frequently Asked Questions
          </Badge>
          <h2 className="text-3xl md:text-5xl font-bold mb-6">
            Got <span className="text-info">Questions?</span>
          </h2>
          <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
            Everything you need to know about our AI trading platform
          </p>
        </div>
        
        <Accordion type="single" collapsible className="space-y-4">
          {faqs.map((faq, index) => (
            <AccordionItem 
              key={index} 
              value={`item-${index}`}
              className="border border-border/50 rounded-lg px-6 bg-card/30 backdrop-blur hover:bg-card/50 transition-colors"
            >
              <AccordionTrigger className="text-left hover:no-underline py-6">
                <span className="font-semibold">{faq.question}</span>
              </AccordionTrigger>
              <AccordionContent className="pb-6 text-muted-foreground leading-relaxed">
                {faq.answer}
              </AccordionContent>
            </AccordionItem>
          ))}
        </Accordion>
        
        <div className="text-center mt-12">
          <p className="text-muted-foreground mb-4">
            Still have questions? Our support team is here to help.
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <a 
              href="mailto:support@cryptotradingbot.com" 
              className="text-primary hover:text-primary/80 font-medium"
            >
              📧 support@cryptotradingbot.com
            </a>
            <span className="hidden sm:block text-muted-foreground">•</span>
            <span className="text-muted-foreground">
              💬 Live chat available 24/7
            </span>
          </div>
        </div>
      </div>
    </section>
  );
}