import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

const resources = {
  en: {
    translation: {
      nav: {
        features: "Features",
        pricing: "Pricing", 
        reviews: "Reviews",
        faq: "FAQ",
        signIn: "Sign In",
        startTrial: "Start Free Trial"
      },
      hero: {
        badge: "AI-Powered Trading Revolution",
        title: "Crypto Trading",
        subtitle: "Redefined by AI",
        description: "Harness the power of artificial intelligence to analyze crypto markets, predict trends, and execute profitable trades automatically on Binance Futures.",
        highlight: "24/7 trading, maximum profit.",
        startTrading: "Start Trading Now",
        watchDemo: "Watch Demo",
        aiChart: "AI Chart Analysis",
        autoTrading: "Automated Trading", 
        riskManagement: "Risk Management"
      },
      features: {
        badge: "Advanced Features",
        title: "Built for",
        subtitle: "Professional Traders",
        description: "Experience the most sophisticated crypto trading platform powered by cutting-edge AI technology"
      },
      pricing: {
        badge: "Pricing Plans",
        title: "Choose Your",
        subtitle: "Trading Edge",
        description: "Flexible pricing plans designed to scale with your trading ambitions",
        mostPopular: "Most Popular",
        getStarted: "Get Started",
        features: "Included Features:",
        limits: "Limits:",
        trial: "All plans include 7-day free trial • Cancel anytime • 30-day money-back guarantee"
      },
      cta: {
        badge: "Ready to Start?",
        title: "Start Your",
        subtitle: "AI Trading Journey",
        todayText: "Today",
        description: "Join thousands of successful traders who trust our AI to grow their crypto portfolios. Start with a free trial and see the difference intelligent automation makes.",
        startTrial: "Start 7-Day Free Trial",
        scheduleDemo: "Schedule Demo Call",
        bankSecurity: "Bank-level security",
        quickSetup: "Setup in 10 minutes",
        noCommitment: "No commitment required",
        footer: "💳 No credit card required for free trial • ✅ Cancel anytime • 🔒 Your funds stay in your exchange"
      }
    }
  },
  vi: {
    translation: {
      nav: {
        features: "Tính năng",
        pricing: "Bảng giá",
        reviews: "Đánh giá", 
        faq: "Câu hỏi",
        signIn: "Đăng nhập",
        startTrial: "Dùng thử miễn phí"
      },
      hero: {
        badge: "Cuộc cách mạng giao dịch bằng AI",
        title: "Giao dịch Crypto",
        subtitle: "Được định nghĩa lại bởi AI",
        description: "Khai thác sức mạnh của trí tuệ nhân tạo để phân tích thị trường crypto, dự đoán xu hướng và thực hiện các giao dịch có lợi nhuận tự động trên Binance Futures.",
        highlight: "Giao dịch 24/7, lợi nhuận tối đa.",
        startTrading: "Bắt đầu giao dịch",
        watchDemo: "Xem demo",
        aiChart: "Phân tích biểu đồ AI",
        autoTrading: "Giao dịch tự động",
        riskManagement: "Quản lý rủi ro"
      },
      features: {
        badge: "Tính năng nâng cao",
        title: "Được xây dựng cho",
        subtitle: "Nhà giao dịch chuyên nghiệp",
        description: "Trải nghiệm nền tảng giao dịch crypto tinh vi nhất được hỗ trợ bởi công nghệ AI tiên tiến"
      },
      pricing: {
        badge: "Bảng giá",
        title: "Chọn",
        subtitle: "Lợi thế giao dịch",
        description: "Gói giá linh hoạt được thiết kế để mở rộng theo tham vọng giao dịch của bạn",
        mostPopular: "Phổ biến nhất",
        getStarted: "Bắt đầu",
        features: "Tính năng bao gồm:",
        limits: "Giới hạn:",
        trial: "Tất cả gói bao gồm dùng thử 7 ngày miễn phí • Hủy bất cứ lúc nào • Đảm bảo hoàn tiền 30 ngày"
      },
      cta: {
        badge: "Sẵn sàng bắt đầu?",
        title: "Bắt đầu hành trình",
        subtitle: "Giao dịch AI",
        todayText: "Hôm nay",
        description: "Tham gia cùng hàng nghìn nhà giao dịch thành công tin tưởng AI của chúng tôi để phát triển danh mục crypto. Bắt đầu với bản dùng thử miễn phí và thấy sự khác biệt mà tự động hóa thông minh mang lại.",
        startTrial: "Dùng thử 7 ngày miễn phí",
        scheduleDemo: "Đặt lịch demo",
        bankSecurity: "Bảo mật cấp ngân hàng",
        quickSetup: "Thiết lập trong 10 phút",
        noCommitment: "Không yêu cầu cam kết",
        footer: "💳 Không cần thẻ tín dụng để dùng thử • ✅ Hủy bất cứ lúc nào • 🔒 Tiền của bạn ở lại sàn giao dịch"
      }
    }
  },
  fr: {
    translation: {
      nav: {
        features: "Fonctionnalités",
        pricing: "Tarification",
        reviews: "Avis",
        faq: "FAQ", 
        signIn: "Se connecter",
        startTrial: "Essai gratuit"
      },
      hero: {
        badge: "Révolution du trading alimentée par l'IA",
        title: "Trading Crypto",
        subtitle: "Redéfini par l'IA",
        description: "Exploitez la puissance de l'intelligence artificielle pour analyser les marchés crypto, prédire les tendances et exécuter automatiquement des trades rentables sur Binance Futures.",
        highlight: "Trading 24/7, profit maximum.",
        startTrading: "Commencer à trader",
        watchDemo: "Voir la démo",
        aiChart: "Analyse graphique IA",
        autoTrading: "Trading automatique",
        riskManagement: "Gestion des risques"
      },
      features: {
        badge: "Fonctionnalités avancées",
        title: "Conçu pour les",
        subtitle: "Traders professionnels",
        description: "Découvrez la plateforme de trading crypto la plus sophistiquée alimentée par une technologie IA de pointe"
      },
      pricing: {
        badge: "Plans tarifaires",
        title: "Choisissez votre",
        subtitle: "Avantage trading",
        description: "Plans tarifaires flexibles conçus pour évoluer avec vos ambitions de trading",
        mostPopular: "Le plus populaire",
        getStarted: "Commencer",
        features: "Fonctionnalités incluses:",
        limits: "Limites:",
        trial: "Tous les plans incluent un essai gratuit de 7 jours • Annulation à tout moment • Garantie de remboursement de 30 jours"
      },
      cta: {
        badge: "Prêt à commencer?",
        title: "Commencez votre",
        subtitle: "Voyage de trading IA",
        todayText: "Aujourd'hui",
        description: "Rejoignez des milliers de traders prospères qui font confiance à notre IA pour développer leurs portefeuilles crypto. Commencez avec un essai gratuit et voyez la différence que fait l'automatisation intelligente.",
        startTrial: "Essai gratuit de 7 jours",
        scheduleDemo: "Planifier une démo",
        bankSecurity: "Sécurité bancaire",
        quickSetup: "Configuration en 10 minutes",
        noCommitment: "Aucun engagement requis",
        footer: "💳 Aucune carte de crédit requise pour l'essai • ✅ Annulez à tout moment • 🔒 Vos fonds restent sur votre exchange"
      }
    }
  },
  zh: {
    translation: {
      nav: {
        features: "功能",
        pricing: "价格",
        reviews: "评价",
        faq: "常见问题",
        signIn: "登录",
        startTrial: "免费试用"
      },
      hero: {
        badge: "AI驱动的交易革命",
        title: "加密货币交易",
        subtitle: "由AI重新定义",
        description: "利用人工智能的力量分析加密市场，预测趋势，并在币安期货上自动执行盈利交易。",
        highlight: "24/7交易，最大利润。",
        startTrading: "开始交易",
        watchDemo: "观看演示",
        aiChart: "AI图表分析",
        autoTrading: "自动交易",
        riskManagement: "风险管理"
      },
      features: {
        badge: "高级功能",
        title: "专为",
        subtitle: "专业交易者",
        description: "体验由尖端AI技术驱动的最先进的加密交易平台"
      },
      pricing: {
        badge: "价格方案",
        title: "选择您的",
        subtitle: "交易优势",
        description: "灵活的价格方案，旨在与您的交易野心一起扩展",
        mostPopular: "最受欢迎",
        getStarted: "开始使用",
        features: "包含功能：",
        limits: "限制：",
        trial: "所有方案包括7天免费试用 • 随时取消 • 30天退款保证"
      },
      cta: {
        badge: "准备开始了吗？",
        title: "开始您的",
        subtitle: "AI交易之旅",
        todayText: "今天",
        description: "加入数千名成功的交易者，他们信任我们的AI来增长他们的加密投资组合。从免费试用开始，看看智能自动化带来的差异。",
        startTrial: "7天免费试用",
        scheduleDemo: "安排演示",
        bankSecurity: "银行级安全",
        quickSetup: "10分钟设置",
        noCommitment: "无需承诺",
        footer: "💳 免费试用无需信用卡 • ✅ 随时取消 • 🔒 您的资金留在您的交易所"
      }
    }
  }
};

i18n
  .use(initReactI18next)
  .init({
    resources,
    lng: 'en',
    fallbackLng: 'en',
    interpolation: {
      escapeValue: false
    }
  });

export default i18n;