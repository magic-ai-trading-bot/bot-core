import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Star, Quote } from "lucide-react";

const testimonials = [
  {
    name: "Alex Chen",
    role: "Professional Trader",
    avatar: "/placeholder.svg",
    initials: "AC",
    rating: 5,
    profit: "+248%",
    content: "This AI trading bot has completely transformed my crypto portfolio. The accuracy of the signals is incredible - I've seen consistent profits for 8 months straight.",
    highlight: "8 months profitable"
  },
  {
    name: "Sarah Rodriguez",
    role: "Investment Manager",
    avatar: "/placeholder.svg", 
    initials: "SR",
    rating: 5,
    profit: "+156%",
    content: "As someone managing institutional funds, I needed something reliable and transparent. The risk management features give me complete confidence in every trade.",
    highlight: "Institutional grade"
  },
  {
    name: "Michael Kim",
    role: "Crypto Enthusiast", 
    avatar: "/placeholder.svg",
    initials: "MK",
    rating: 5,
    profit: "+89%",
    content: "Started with the Basic plan and upgraded to Premium after seeing results. The AI's market analysis is far superior to any manual trading I've done.",
    highlight: "From basic to premium"
  },
  {
    name: "Emma Thompson",
    role: "Day Trader",
    avatar: "/placeholder.svg",
    initials: "ET", 
    rating: 5,
    profit: "+312%",
    content: "The 24/7 automation means I never miss opportunities. Woke up to profitable trades that happened while I was sleeping - it's like having a personal trading assistant.",
    highlight: "24/7 profits"
  },
  {
    name: "David Park",
    role: "Hedge Fund Analyst",
    avatar: "/placeholder.svg",
    initials: "DP",
    rating: 5,
    profit: "+198%",
    content: "We've integrated this into our fund's strategy. The AI's ability to process market sentiment and technical indicators simultaneously is remarkable.",
    highlight: "Hedge fund approved"
  },
  {
    name: "Lisa Wang",
    role: "Financial Advisor",
    avatar: "/placeholder.svg",
    initials: "LW", 
    rating: 5,
    profit: "+134%",
    content: "Recommended this to my clients and they're thrilled with the results. The transparent reporting and risk controls make it perfect for wealth management.",
    highlight: "Client recommended"
  }
];

export function TestimonialsSection() {
  return (
    <section className="py-24 bg-gradient-to-b from-background to-card/20">
      <div className="container mx-auto px-4">
        <div className="text-center mb-16">
          <Badge variant="outline" className="mb-4 bg-accent/10 text-accent border-accent/20">
            <Star className="w-3 h-3 mr-1" />
            Success Stories
          </Badge>
          <h2 className="text-3xl md:text-5xl font-bold mb-6">
            Trusted by <span className="text-profit">Thousands</span> of Traders
          </h2>
          <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
            Join successful traders who have transformed their portfolios with our AI-powered platform
          </p>
        </div>
        
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {testimonials.map((testimonial, index) => (
            <Card key={index} className="group hover:shadow-lg transition-all duration-300 border-border/50 bg-card/50 backdrop-blur hover:bg-card/80">
              <CardContent className="p-6">
                <div className="flex items-start gap-4 mb-4">
                  <Avatar className="w-12 h-12">
                    <AvatarImage src={testimonial.avatar} alt={testimonial.name} />
                    <AvatarFallback className="bg-primary/10 text-primary font-semibold">
                      {testimonial.initials}
                    </AvatarFallback>
                  </Avatar>
                  <div className="flex-1">
                    <div className="flex items-center gap-2 mb-1">
                      <h4 className="font-semibold">{testimonial.name}</h4>
                      <Badge variant="outline" className="text-xs bg-profit/10 text-profit border-profit/20">
                        {testimonial.profit}
                      </Badge>
                    </div>
                    <p className="text-sm text-muted-foreground">{testimonial.role}</p>
                    <div className="flex items-center gap-1 mt-1">
                      {[...Array(testimonial.rating)].map((_, i) => (
                        <Star key={i} className="w-3 h-3 fill-yellow-500 text-yellow-500" />
                      ))}
                    </div>
                  </div>
                </div>
                
                <div className="relative">
                  <Quote className="w-6 h-6 text-muted-foreground/30 mb-2" />
                  <p className="text-sm leading-relaxed mb-4 italic">
                    "{testimonial.content}"
                  </p>
                  <Badge variant="secondary" className="text-xs">
                    {testimonial.highlight}
                  </Badge>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
        
        <div className="text-center mt-12">
          <div className="flex items-center justify-center gap-8 text-sm text-muted-foreground">
            <div className="flex items-center gap-2">
              <div className="flex">
                {[...Array(5)].map((_, i) => (
                  <Star key={i} className="w-4 h-4 fill-yellow-500 text-yellow-500" />
                ))}
              </div>
              <span>4.9/5 average rating</span>
            </div>
            <div>2,500+ active traders</div>
            <div>$50M+ in managed capital</div>
          </div>
        </div>
      </div>
    </section>
  );
}