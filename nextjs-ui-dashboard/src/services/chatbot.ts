import logger from "@/utils/logger";

export interface ChatMessage {
  id: string;
  type: "user" | "bot";
  content: string;
  timestamp: Date;
  isTyping?: boolean;
  sources?: Array<{ title: string; path: string }>;
}

export interface ChatResponse {
  success: boolean;
  message: string;
  confidence: number;
  type: "faq" | "ai" | "rag" | "fallback" | "error";
  sources?: Array<{ title: string; path: string }>;
}

// All chat responses use local FAQ database.

// FAQ Database cho trading bot (Vietnamese)
const FAQ_DATABASE = {
  "bot hoạt động": {
    keywords: ["bot", "hoạt động", "làm việc", "chạy", "trading"],
    response: `Bot trading AI của chúng tôi hoạt động 24/7 để:
• Phân tích thị trường crypto liên tục
• Sử dụng AI để dự đoán xu hướng giá
• Tự động thực hiện lệnh mua/bán theo chiến lược
• Quản lý rủi ro với stop-loss và take-profit
• Theo dõi nhiều cặp tiền tệ đồng thời (BTC/USDT, ETH/USDT, BNB/USDT, SOL/USDT)`,
    confidence: 0.95,
  },
  "bắt đầu": {
    keywords: ["bắt đầu", "start", "khởi động", "sử dụng", "setup"],
    response: `Để bắt đầu sử dụng bot trading:
1. **Tạo tài khoản Binance** và lấy API keys
2. **Thiết lập cấu hình** trong phần Settings
3. **Chọn chiến lược AI** phù hợp với style trading của bạn
4. **Đặt vốn giao dịch** (khuyến nghị từ $500+)
5. **Bật Paper Trading** để test trước khi trade thật
6. **Nhấn Start Trading** để bot bắt đầu

💡 Tip: Nên bắt đầu với Paper Trading để hiểu cách bot hoạt động!`,
    confidence: 0.95,
  },
  "an toàn": {
    keywords: ["an toàn", "bảo mật", "rủi ro", "mất tiền", "trust"],
    response: `Bot trading của chúng tôi rất an toàn:
🔒 **Bảo mật tuyệt đối**:
• Tiền của bạn luôn trong tài khoản Binance của bạn
• Bot chỉ có quyền trading, không thể rút tiền
• Mã hóa API keys với AES-256

⚡ **Quản lý rủi ro thông minh**:
• Stop-loss tự động (có thể set 1-15%)
• Take-profit để chốt lời kịp thời
• Giới hạn số lệnh mở cùng lúc
• Emergency stop để dừng ngay lập tức

📊 **Minh bạch**: Tất cả giao dịch đều có thể theo dõi realtime`,
    confidence: 0.98,
  },
  "chiến lược": {
    keywords: ["chiến lược", "strategy", "RSI", "MACD", "Bollinger"],
    response: `Bot hỗ trợ nhiều chiến lược AI:
📈 **RSI Strategy**: Dựa trên chỉ báo RSI (14,30,70)
📊 **MACD Strategy**: Phân tích đường MACD và Signal
📉 **Bollinger Bands**: Giao dịch theo dải Bollinger
📊 **Volume Strategy**: Phân tích volume để xác định xu hướng
🤖 **AI Hybrid**: Kết hợp tất cả chỉ báo + Machine Learning

💡 **Tùy chỉnh được**:
• Điều chỉnh tham số từng chiến lược
• Chọn timeframe (1m, 5m, 15m, 1h, 4h)
• Set risk level phù hợp
• Bật/tắt từng cặp coin`,
    confidence: 0.92,
  },
  phí: {
    keywords: ["phí", "fee", "chi phí", "giá", "cost"],
    response: `Chi phí sử dụng bot trading:
💰 **Phí bot**: MIỄN PHÍ trong giai đoạn beta
💱 **Phí Binance**: 0.1% mỗi lệnh (standard)
📊 **Không phí ẩn**: Hoàn toàn minh bạch

🎯 **Lợi nhuận ước tính**:
• Accuracy rate: 73-78%
• Monthly return: 8-15% (tùy market)
• Drawdown: < 5% với risk management tốt

💡 **Tip**: Profit từ bot thường cao hơn phí rất nhiều lần!`,
    confidence: 0.89,
  },
  "vốn tối thiểu": {
    keywords: ["vốn", "capital", "tiền", "minimum", "tối thiểu"],
    response: `Vốn tối thiểu để sử dụng bot:
💵 **Tối thiểu kỹ thuật**: $100 USD
💰 **Khuyến nghị**: $500 - $1000 USD
🚀 **Tối ưu**: $2000+ USD

📊 **Tại sao cần vốn hợp lý?**:
• Quản lý rủi ro tốt hơn với position sizing
• Đa dạng hóa nhiều cặp coin
• Chịu được volatility của crypto
• Compound effect tốt hơn

⚠️ **Lưu ý**: Chỉ invest số tiền bạn có thể mất được!`,
    confidence: 0.94,
  },
  "kết quả": {
    keywords: ["kết quả", "performance", "profit", "lợi nhuận", "thống kê"],
    response: `Kết quả trading bot (backtesting):
📊 **Thống kê chung**:
• Win rate: 73-78%
• Monthly return: 8-15%
• Max drawdown: < 5%
• Sharpe ratio: 2.1-2.8

📈 **Performance theo thời gian**:
• Daily: 0.3-0.8% average
• Weekly: 2-4% average  
• Monthly: 8-15% average

⚡ **Realtime tracking**:
• Xem P&L realtime trong dashboard
• Lịch sử giao dịch chi tiết
• Biểu đồ performance
• AI confidence scores

⚠️ **Disclaimer**: Kết quả quá khứ không đảm bảo tương lai`,
    confidence: 0.91,
  },
  "hỗ trợ": {
    keywords: ["hỗ trợ", "support", "help", "liên hệ", "problem"],
    response: `Cần hỗ trợ? Chúng tôi sẵn sàng giúp bạn!
📞 **Liên hệ hỗ trợ**:
• Telegram: @TradingBotSupport
• Discord: Trading Bot Community
• Email: support@tradingbot.com

🔧 **Tự khắc phục**:
• Kiểm tra API keys còn hiệu lực
• Đảm bảo đủ balance trong account
• Refresh page nếu UI lag
• Check network connection

📚 **Tài liệu**: Có guide chi tiết trong phần Settings → Documentation`,
    confidence: 0.96,
  },
};

// Hugging Face API configuration
const HF_API_URL =
  "https://api-inference.huggingface.co/models/microsoft/DialoGPT-large";
const HF_API_KEY = import.meta.env.VITE_HF_API_KEY || ""; // Load from environment variable

class ChatbotService {
  private conversationHistory: ChatMessage[] = [];
  private requestCount = 0;
  private lastRequestTime = 0;
  private readonly REQUEST_LIMIT = 10; // Giới hạn 10 requests mỗi 5 phút
  private readonly TIME_WINDOW = 5 * 60 * 1000; // 5 phút
  private useRAG = true; // Default to RAG mode

  // Kiểm tra rate limiting
  private checkRateLimit(): boolean {
    const now = Date.now();
    if (now - this.lastRequestTime > this.TIME_WINDOW) {
      this.requestCount = 0;
      this.lastRequestTime = now;
    }

    if (this.requestCount >= this.REQUEST_LIMIT) {
      return false;
    }

    this.requestCount++;
    return true;
  }

  // Tìm kiếm trong FAQ database
  private findFAQMatch(
    message: string
  ): { response: string; confidence: number } | null {
    const normalizedMessage = message.toLowerCase().trim();

    for (const [key, faq] of Object.entries(FAQ_DATABASE)) {
      for (const keyword of faq.keywords) {
        if (normalizedMessage.includes(keyword.toLowerCase())) {
          return {
            response: faq.response,
            confidence: faq.confidence,
          };
        }
      }
    }

    return null;
  }

  // Local FAQ chatbot
  private async callRAGChatbot(message: string): Promise<ChatResponse> {
    if (!this.checkRateLimit()) {
      return {
        success: false,
        message: "⏰ Xin lỗi, bạn đã hỏi quá nhiều câu hỏi. Vui lòng chờ 5 phút rồi thử lại.",
        confidence: 0,
        type: "error",
      };
    }
    return this.fallbackToLocalFAQ(message);
  }

  // Search local FAQ
  private fallbackToLocalFAQ(message: string): ChatResponse {
    const faqMatch = this.findFAQMatch(message);

    if (faqMatch) {
      return {
        success: true,
        message: faqMatch.response,
        confidence: faqMatch.confidence,
        type: "faq",
      };
    }

    return {
      success: true,
      message: `🤔 Tôi chưa tìm thấy thông tin về câu hỏi này.

Bạn có thể hỏi về:
• Bot hoạt động như thế nào?
• Cách bắt đầu sử dụng?
• Các chiến lược trading?
• Bảo mật và quản lý rủi ro?
• Paper trading là gì?`,
      confidence: 0.5,
      type: "fallback",
    };
  }

  // Gọi Hugging Face API (legacy fallback)
  private async callHuggingFaceAPI(message: string): Promise<string> {
    if (!this.checkRateLimit()) {
      return "⏰ Xin lỗi, bạn đã hỏi quá nhiều câu hỏi. Vui lòng chờ 5 phút rồi thử lại. Trong lúc này, bạn có thể xem FAQ hoặc documentation.";
    }

    try {
      const response = await fetch(HF_API_URL, {
        method: "POST",
        headers: {
          Authorization: `Bearer ${HF_API_KEY}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          inputs: `Context: You are a helpful trading bot assistant. Answer questions about cryptocurrency trading bot in Vietnamese. Keep responses concise and helpful.

User: ${message}
Assistant:`,
          parameters: {
            max_length: 200,
            temperature: 0.7,
            do_sample: true,
            return_full_text: false,
          },
        }),
      });

      if (!response.ok) {
        throw new Error(`API Error: ${response.status}`);
      }

      const result = await response.json();

      if (result.error) {
        throw new Error(result.error);
      }

      return (
        result[0]?.generated_text?.trim() ||
        "Xin lỗi, tôi không hiểu câu hỏi của bạn."
      );
    } catch (error) {
      logger.error("Hugging Face API Error:", error);
      return "⚠️ Hiện tại AI đang bảo trì. Vui lòng hỏi câu hỏi khác hoặc liên hệ support.";
    }
  }

  // Toggle RAG mode
  setRAGMode(enabled: boolean): void {
    this.useRAG = enabled;
    logger.info(`RAG mode ${enabled ? "enabled" : "disabled"}`);
  }

  // Check if RAG mode is enabled
  isRAGEnabled(): boolean {
    return this.useRAG;
  }

  // Xử lý tin nhắn chính
  async processMessage(message: string): Promise<ChatResponse> {
    try {
      // Use RAG chatbot as primary (calls Python backend with GPT-4)
      if (this.useRAG) {
        return await this.callRAGChatbot(message);
      }

      // Legacy mode: Local FAQ first, then Hugging Face
      const faqMatch = this.findFAQMatch(message);

      if (faqMatch && faqMatch.confidence > 0.8) {
        return {
          success: true,
          message: faqMatch.response,
          confidence: faqMatch.confidence,
          type: "faq",
        };
      }

      // Nếu không tìm thấy trong FAQ, dùng Hugging Face AI
      const aiResponse = await this.callHuggingFaceAPI(message);

      return {
        success: true,
        message: aiResponse,
        confidence: 0.7,
        type: "ai",
      };
    } catch (error) {
      logger.error("Chatbot Error:", error);
      return {
        success: false,
        message:
          "Xin lỗi, có lỗi xảy ra. Vui lòng thử lại sau hoặc liên hệ support.",
        confidence: 0,
        type: "error",
      };
    }
  }

  // Lấy suggested questions
  async getSuggestedQuestionsAsync(): Promise<string[]> {
    return this.getDefaultSuggestedQuestions();
  }

  // Lấy suggested questions (sync version - uses defaults)
  getSuggestedQuestions(): string[] {
    return this.getDefaultSuggestedQuestions();
  }

  // Default suggested questions
  private getDefaultSuggestedQuestions(): string[] {
    return [
      "Bot hoạt động như thế nào?",
      "Làm sao để bắt đầu sử dụng bot?",
      "Bot có an toàn không?",
      "Vốn tối thiểu là bao nhiêu?",
      "Bot có những chiến lược gì?",
      "Kết quả trading như thế nào?",
      "Paper trading là gì?",
      "Cách cấu hình API keys?",
    ];
  }

  // Clear conversation history
  async clearHistoryAsync(): Promise<void> {
    this.conversationHistory = [];
  }

  // Reset rate limit counter (for testing purposes)
  resetRateLimit(): void {
    this.requestCount = 0;
    this.lastRequestTime = 0;
  }

  // Lấy lịch sử conversation
  getConversationHistory(): ChatMessage[] {
    return this.conversationHistory;
  }

  // Thêm tin nhắn vào lịch sử
  addMessageToHistory(message: ChatMessage): void {
    this.conversationHistory.push(message);
    // Giới hạn lịch sử 50 tin nhắn
    if (this.conversationHistory.length > 50) {
      this.conversationHistory = this.conversationHistory.slice(-50);
    }
  }

  // Clear lịch sử
  clearHistory(): void {
    this.conversationHistory = [];
  }
}

export const chatbotService = new ChatbotService();
export default chatbotService;
