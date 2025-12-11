import { motion } from "framer-motion";
import { Link } from "react-router-dom";
import { useState } from "react";
import { useTranslation } from "react-i18next";
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
  GlassCard,
  GradientText,
  PremiumButton,
  Badge,
  GlowIcon,
  PageWrapper,
} from "@/styles/luxury-design-system";
import { useThemeColors } from "@/hooks/useThemeColors";
import { toast } from "sonner";

const Contact = () => {
  const colors = useThemeColors();
  const { t } = useTranslation('pages');

  const contactInfoKeys = [
    { key: "email", icon: Mail },
    { key: "liveChat", icon: MessageCircle },
    { key: "phone", icon: Phone },
    { key: "office", icon: MapPin },
  ];

  const categoryKeys = [
    { key: "general", icon: HelpCircle },
    { key: "technical", icon: Headphones },
    { key: "enterprise", icon: Building },
    { key: "partnership", icon: MessageCircle },
  ];

  const [formData, setFormData] = useState({
    name: "",
    email: "",
    subject: "",
    message: "",
    category: "general",
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    toast.success(t('contact.form.success'));
    setFormData({ name: "", email: "", subject: "", message: "", category: "general" });
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
            {t('common.backToHome')}
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
          {t('contact.badge')}
        </Badge>
        <h1 className="text-4xl md:text-5xl font-black mb-4">
          <GradientText>{t('contact.title')}</GradientText>
        </h1>
        <p className="text-lg max-w-2xl mx-auto" style={{ color: colors.textMuted }}>
          {t('contact.description')}
        </p>
      </motion.div>

      {/* Contact Info Cards */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.1 }}
        className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-12"
      >
        {contactInfoKeys.map((info) => (
          <GlassCard key={info.key} noPadding className="p-4 text-center">
            <GlowIcon icon={info.icon} size="md" color={colors.cyan} className="mx-auto mb-3" />
            <h3 className="font-semibold mb-1" style={{ color: colors.textPrimary }}>
              {t(`contact.info.${info.key}.title`)}
            </h3>
            <p className="text-sm font-medium mb-1" style={{ color: colors.cyan }}>
              {t(`contact.info.${info.key}.value`)}
            </p>
            <p className="text-xs" style={{ color: colors.textMuted }}>
              {t(`contact.info.${info.key}.description`)}
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
            <h2 className="text-xl font-bold mb-6" style={{ color: colors.textPrimary }}>
              {t('contact.form.title')}
            </h2>
            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm mb-2" style={{ color: colors.textSecondary }}>
                    {t('contact.form.name')}
                  </label>
                  <input
                    type="text"
                    required
                    value={formData.name}
                    onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                    className="w-full px-4 py-2 rounded-lg border bg-transparent outline-none text-sm focus:border-cyan-500"
                    style={{
                      borderColor: colors.borderSubtle,
                      color: colors.textPrimary,
                    }}
                  />
                </div>
                <div>
                  <label className="block text-sm mb-2" style={{ color: colors.textSecondary }}>
                    {t('contact.form.email')}
                  </label>
                  <input
                    type="email"
                    required
                    value={formData.email}
                    onChange={(e) => setFormData({ ...formData, email: e.target.value })}
                    className="w-full px-4 py-2 rounded-lg border bg-transparent outline-none text-sm focus:border-cyan-500"
                    style={{
                      borderColor: colors.borderSubtle,
                      color: colors.textPrimary,
                    }}
                  />
                </div>
              </div>
              <div>
                <label className="block text-sm mb-2" style={{ color: colors.textSecondary }}>
                  {t('contact.form.category')}
                </label>
                <select
                  value={formData.category}
                  onChange={(e) => setFormData({ ...formData, category: e.target.value })}
                  className="w-full px-4 py-2 rounded-lg border bg-transparent outline-none text-sm focus:border-cyan-500"
                  style={{
                    borderColor: colors.borderSubtle,
                    color: colors.textPrimary,
                    backgroundColor: colors.bgSecondary,
                  }}
                >
                  {categoryKeys.map((cat) => (
                    <option key={cat.key} value={cat.key}>
                      {t(`contact.categories.${cat.key}.title`)}
                    </option>
                  ))}
                </select>
              </div>
              <div>
                <label className="block text-sm mb-2" style={{ color: colors.textSecondary }}>
                  {t('contact.form.subject')}
                </label>
                <input
                  type="text"
                  required
                  value={formData.subject}
                  onChange={(e) => setFormData({ ...formData, subject: e.target.value })}
                  className="w-full px-4 py-2 rounded-lg border bg-transparent outline-none text-sm focus:border-cyan-500"
                  style={{
                    borderColor: colors.borderSubtle,
                    color: colors.textPrimary,
                  }}
                />
              </div>
              <div>
                <label className="block text-sm mb-2" style={{ color: colors.textSecondary }}>
                  {t('contact.form.message')}
                </label>
                <textarea
                  required
                  rows={5}
                  value={formData.message}
                  onChange={(e) => setFormData({ ...formData, message: e.target.value })}
                  className="w-full px-4 py-2 rounded-lg border bg-transparent outline-none text-sm focus:border-cyan-500 resize-none"
                  style={{
                    borderColor: colors.borderSubtle,
                    color: colors.textPrimary,
                  }}
                />
              </div>
              <PremiumButton variant="primary" type="submit" fullWidth>
                <Send className="w-4 h-4" />
                {t('contact.form.submit')}
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
          <h2 className="text-xl font-bold mb-4" style={{ color: colors.textPrimary }}>
            {t('contact.categories.title')}
          </h2>
          {categoryKeys.map((category) => (
            <GlassCard
              key={category.key}
              noPadding
              className="p-4 hover:border-cyan-500/30 transition-all cursor-pointer"
            >
              <div className="flex items-start gap-3">
                <GlowIcon icon={category.icon} size="sm" color={colors.cyan} />
                <div>
                  <h3 className="font-semibold" style={{ color: colors.textPrimary }}>
                    {t(`contact.categories.${category.key}.title`)}
                  </h3>
                  <p className="text-sm" style={{ color: colors.textMuted }}>
                    {t(`contact.categories.${category.key}.description`)}
                  </p>
                </div>
              </div>
            </GlassCard>
          ))}

          {/* Response Time */}
          <GlassCard noPadding className="p-4 mt-6">
            <div className="flex items-center gap-3 mb-3">
              <GlowIcon icon={Clock} size="sm" color={colors.profit} />
              <h3 className="font-semibold" style={{ color: colors.textPrimary }}>
                {t('contact.responseTime.title')}
              </h3>
            </div>
            <ul className="space-y-2 text-sm" style={{ color: colors.textMuted }}>
              {(t('contact.responseTime.items', { returnObjects: true }) as string[]).map((item, index) => (
                <li key={index}>• {item}</li>
              ))}
            </ul>
          </GlassCard>
        </motion.div>
      </div>
    </PageWrapper>
  );
};

export default Contact;
