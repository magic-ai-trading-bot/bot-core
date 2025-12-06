import { motion } from "framer-motion";
import { Link } from "react-router-dom";
import { useState } from "react";
import {
  Mail,
  Phone,
  MapPin,
  MessageCircle,
  Send,
  ArrowLeft,
  Clock,
  HelpCircle,
  Headphones,
  Building,
} from "lucide-react";
import {
  luxuryColors,
  GlassCard,
  GradientText,
  PremiumButton,
  Badge,
  GlowIcon,
  PageWrapper,
} from "@/styles/luxury-design-system";
import { toast } from "sonner";

const contactInfo = [
  {
    icon: Mail,
    title: "Email",
    value: "support@botcore.io",
    description: "We'll respond within 24 hours",
  },
  {
    icon: MessageCircle,
    title: "Live Chat",
    value: "Available 24/7",
    description: "Chat with our support team",
  },
  {
    icon: Phone,
    title: "Phone",
    value: "+1 (555) 123-4567",
    description: "Mon-Fri 9am-6pm PST",
  },
  {
    icon: MapPin,
    title: "Office",
    value: "San Francisco, CA",
    description: "123 Market Street, Suite 456",
  },
];

const supportCategories = [
  {
    icon: HelpCircle,
    title: "General Inquiry",
    description: "Questions about our platform",
  },
  {
    icon: Headphones,
    title: "Technical Support",
    description: "Help with technical issues",
  },
  {
    icon: Building,
    title: "Enterprise Sales",
    description: "Custom solutions for businesses",
  },
  {
    icon: MessageCircle,
    title: "Partnership",
    description: "Collaboration opportunities",
  },
];

const Contact = () => {
  const [formData, setFormData] = useState({
    name: "",
    email: "",
    subject: "",
    message: "",
    category: "General Inquiry",
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    toast.success("Message sent successfully! We'll get back to you soon.");
    setFormData({ name: "", email: "", subject: "", message: "", category: "General Inquiry" });
  };

  return (
    <PageWrapper>
      {/* Back Button */}
      <motion.div
        initial={{ opacity: 0, x: -20 }}
        animate={{ opacity: 1, x: 0 }}
        className="mb-8"
      >
        <Link to="/">
          <PremiumButton variant="secondary" size="sm">
            <ArrowLeft className="w-4 h-4" />
            Back to Home
          </PremiumButton>
        </Link>
      </motion.div>

      {/* Hero Section */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        className="text-center mb-12"
      >
        <Badge variant="info" className="mb-4">
          Contact Us
        </Badge>
        <h1 className="text-4xl md:text-5xl font-black mb-4">
          <GradientText>Get in Touch</GradientText>
        </h1>
        <p className="text-lg max-w-2xl mx-auto" style={{ color: luxuryColors.textMuted }}>
          Have questions? We're here to help. Reach out to our team and we'll get back to you as soon as possible.
        </p>
      </motion.div>

      {/* Contact Info Cards */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.1 }}
        className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-12"
      >
        {contactInfo.map((info, index) => (
          <GlassCard key={info.title} noPadding className="p-4 text-center">
            <GlowIcon icon={info.icon} size="md" color={luxuryColors.cyan} className="mx-auto mb-3" />
            <h3 className="font-semibold mb-1" style={{ color: luxuryColors.textPrimary }}>
              {info.title}
            </h3>
            <p className="text-sm font-medium mb-1" style={{ color: luxuryColors.cyan }}>
              {info.value}
            </p>
            <p className="text-xs" style={{ color: luxuryColors.textMuted }}>
              {info.description}
            </p>
          </GlassCard>
        ))}
      </motion.div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        {/* Contact Form */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.2 }}
          className="lg:col-span-2"
        >
          <GlassCard>
            <h2 className="text-xl font-bold mb-6" style={{ color: luxuryColors.textPrimary }}>
              Send Us a Message
            </h2>
            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm mb-2" style={{ color: luxuryColors.textSecondary }}>
                    Your Name
                  </label>
                  <input
                    type="text"
                    required
                    value={formData.name}
                    onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                    className="w-full px-4 py-2 rounded-lg border bg-transparent outline-none text-sm focus:border-cyan-500"
                    style={{
                      borderColor: luxuryColors.borderSubtle,
                      color: luxuryColors.textPrimary,
                    }}
                    placeholder="John Doe"
                  />
                </div>
                <div>
                  <label className="block text-sm mb-2" style={{ color: luxuryColors.textSecondary }}>
                    Email Address
                  </label>
                  <input
                    type="email"
                    required
                    value={formData.email}
                    onChange={(e) => setFormData({ ...formData, email: e.target.value })}
                    className="w-full px-4 py-2 rounded-lg border bg-transparent outline-none text-sm focus:border-cyan-500"
                    style={{
                      borderColor: luxuryColors.borderSubtle,
                      color: luxuryColors.textPrimary,
                    }}
                    placeholder="john@example.com"
                  />
                </div>
              </div>
              <div>
                <label className="block text-sm mb-2" style={{ color: luxuryColors.textSecondary }}>
                  Category
                </label>
                <select
                  value={formData.category}
                  onChange={(e) => setFormData({ ...formData, category: e.target.value })}
                  className="w-full px-4 py-2 rounded-lg border bg-transparent outline-none text-sm focus:border-cyan-500"
                  style={{
                    borderColor: luxuryColors.borderSubtle,
                    color: luxuryColors.textPrimary,
                    backgroundColor: luxuryColors.bgSecondary,
                  }}
                >
                  {supportCategories.map((cat) => (
                    <option key={cat.title} value={cat.title}>
                      {cat.title}
                    </option>
                  ))}
                </select>
              </div>
              <div>
                <label className="block text-sm mb-2" style={{ color: luxuryColors.textSecondary }}>
                  Subject
                </label>
                <input
                  type="text"
                  required
                  value={formData.subject}
                  onChange={(e) => setFormData({ ...formData, subject: e.target.value })}
                  className="w-full px-4 py-2 rounded-lg border bg-transparent outline-none text-sm focus:border-cyan-500"
                  style={{
                    borderColor: luxuryColors.borderSubtle,
                    color: luxuryColors.textPrimary,
                  }}
                  placeholder="How can we help?"
                />
              </div>
              <div>
                <label className="block text-sm mb-2" style={{ color: luxuryColors.textSecondary }}>
                  Message
                </label>
                <textarea
                  required
                  rows={5}
                  value={formData.message}
                  onChange={(e) => setFormData({ ...formData, message: e.target.value })}
                  className="w-full px-4 py-2 rounded-lg border bg-transparent outline-none text-sm focus:border-cyan-500 resize-none"
                  style={{
                    borderColor: luxuryColors.borderSubtle,
                    color: luxuryColors.textPrimary,
                  }}
                  placeholder="Tell us more about your inquiry..."
                />
              </div>
              <PremiumButton variant="primary" type="submit" fullWidth>
                <Send className="w-4 h-4" />
                Send Message
              </PremiumButton>
            </form>
          </GlassCard>
        </motion.div>

        {/* Support Categories */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.3 }}
          className="space-y-4"
        >
          <h2 className="text-xl font-bold mb-4" style={{ color: luxuryColors.textPrimary }}>
            How Can We Help?
          </h2>
          {supportCategories.map((category) => (
            <GlassCard
              key={category.title}
              noPadding
              className="p-4 hover:border-cyan-500/30 transition-all cursor-pointer"
            >
              <div className="flex items-start gap-3">
                <GlowIcon icon={category.icon} size="sm" color={luxuryColors.cyan} />
                <div>
                  <h3 className="font-semibold" style={{ color: luxuryColors.textPrimary }}>
                    {category.title}
                  </h3>
                  <p className="text-sm" style={{ color: luxuryColors.textMuted }}>
                    {category.description}
                  </p>
                </div>
              </div>
            </GlassCard>
          ))}

          {/* Response Time */}
          <GlassCard noPadding className="p-4 mt-6">
            <div className="flex items-center gap-3 mb-3">
              <GlowIcon icon={Clock} size="sm" color={luxuryColors.profit} />
              <h3 className="font-semibold" style={{ color: luxuryColors.textPrimary }}>
                Response Time
              </h3>
            </div>
            <ul className="space-y-2 text-sm" style={{ color: luxuryColors.textMuted }}>
              <li>• Email: Within 24 hours</li>
              <li>• Live Chat: Under 5 minutes</li>
              <li>• Phone: Immediate during business hours</li>
              <li>• Enterprise: Dedicated support</li>
            </ul>
          </GlassCard>
        </motion.div>
      </div>
    </PageWrapper>
  );
};

export default Contact;
