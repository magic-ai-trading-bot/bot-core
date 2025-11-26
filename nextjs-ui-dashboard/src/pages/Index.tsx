import { LandingHeader } from "@/components/landing/LandingHeader";
import { HeroSection } from "@/components/landing/HeroSection";
import { PartnersSection } from "@/components/landing/PartnersSection";
import { FeaturesSection } from "@/components/landing/FeaturesSection";
import { PricingSection } from "@/components/landing/PricingSection";
import { TestimonialsSection } from "@/components/landing/TestimonialsSection";
import { FAQSection } from "@/components/landing/FAQSection";
import { CTASection } from "@/components/landing/CTASection";
import { LandingFooter } from "@/components/landing/LandingFooter";
import ChatBot from "@/components/ChatBot";

const Index = () => {
  return (
    <div className="min-h-screen bg-background">
      <LandingHeader />

      <main>
        <HeroSection />

        {/* Partners & Integrations */}
        <PartnersSection />

        <section id="features">
          <FeaturesSection />
        </section>

        <section id="pricing">
          <PricingSection />
        </section>

        <section id="testimonials">
          <TestimonialsSection />
        </section>

        <section id="faq">
          <FAQSection />
        </section>

        <CTASection />
      </main>

      <LandingFooter />

      {/* Chatbot Widget */}
      <ChatBot />
    </div>
  );
};

export default Index;
