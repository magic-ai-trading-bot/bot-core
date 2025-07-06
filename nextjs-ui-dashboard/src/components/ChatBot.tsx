import React, { useState, useEffect, useRef } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  MessageCircle,
  Send,
  X,
  Minimize2,
  Maximize2,
  Bot,
  User,
  Loader2,
  Sparkles,
  HelpCircle,
  Trash2,
  Volume2,
  VolumeX,
} from "lucide-react";
import { chatbotService, ChatMessage } from "@/services/chatbot";
import { toast } from "sonner";

interface ChatBotProps {
  isOpen?: boolean;
  onToggle?: () => void;
  className?: string;
}

interface TypingIndicatorProps {
  isTyping: boolean;
}

const TypingIndicator: React.FC<TypingIndicatorProps> = ({ isTyping }) => {
  if (!isTyping) return null;

  return (
    <div className="flex items-center space-x-2 p-3 bg-secondary/30 rounded-lg max-w-[80px]">
      <div className="flex items-center space-x-1">
        <div className="w-2 h-2 bg-primary rounded-full animate-bounce"></div>
        <div
          className="w-2 h-2 bg-primary rounded-full animate-bounce"
          style={{ animationDelay: "0.1s" }}
        ></div>
        <div
          className="w-2 h-2 bg-primary rounded-full animate-bounce"
          style={{ animationDelay: "0.2s" }}
        ></div>
      </div>
    </div>
  );
};

const ChatBot: React.FC<ChatBotProps> = ({
  isOpen: controlledOpen,
  onToggle,
  className = "",
}) => {
  const [internalOpen, setInternalOpen] = useState(false);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [inputMessage, setInputMessage] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [isTyping, setIsTyping] = useState(false);
  const [isMinimized, setIsMinimized] = useState(false);
  const [soundEnabled, setSoundEnabled] = useState(true);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const isOpen = controlledOpen !== undefined ? controlledOpen : internalOpen;

  // Scroll to bottom when new messages arrive
  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages, isTyping]);

  // Initial greeting message
  useEffect(() => {
    if (isOpen && messages.length === 0) {
      const welcomeMessage: ChatMessage = {
        id: "welcome",
        type: "bot",
        content: `👋 Chào bạn! Tôi là AI Assistant của Trading Bot.

Tôi có thể giúp bạn:
• Hiểu cách bot hoạt động
• Hướng dẫn sử dụng các tính năng
• Giải thích về chiến lược trading
• Hỗ trợ kỹ thuật và troubleshooting

Hãy hỏi tôi bất cứ điều gì nhé! 🤖`,
        timestamp: new Date(),
      };
      setMessages([welcomeMessage]);
    }
  }, [isOpen]);

  // Handle toggle
  const handleToggle = () => {
    if (onToggle) {
      onToggle();
    } else {
      setInternalOpen(!internalOpen);
    }
  };

  // Handle minimize
  const handleMinimize = () => {
    setIsMinimized(!isMinimized);
  };

  // Play notification sound
  const playNotificationSound = () => {
    if (soundEnabled) {
      // Create a simple notification sound
      const AudioContextClass =
        window.AudioContext ||
        (window as typeof window & { webkitAudioContext: typeof AudioContext })
          .webkitAudioContext;
      const context = new AudioContextClass();
      const oscillator = context.createOscillator();
      const gainNode = context.createGain();

      oscillator.connect(gainNode);
      gainNode.connect(context.destination);

      oscillator.frequency.value = 800;
      oscillator.type = "sine";
      gainNode.gain.setValueAtTime(0.1, context.currentTime);
      gainNode.gain.exponentialRampToValueAtTime(
        0.01,
        context.currentTime + 0.1
      );

      oscillator.start(context.currentTime);
      oscillator.stop(context.currentTime + 0.1);
    }
  };

  // Send message
  const handleSendMessage = async () => {
    if (!inputMessage.trim() || isLoading) return;

    const userMessage: ChatMessage = {
      id: Date.now().toString(),
      type: "user",
      content: inputMessage.trim(),
      timestamp: new Date(),
    };

    setMessages((prev) => [...prev, userMessage]);
    setInputMessage("");
    setIsLoading(true);
    setIsTyping(true);

    try {
      // Add to chatbot service history
      chatbotService.addMessageToHistory(userMessage);

      // Get response from chatbot service
      const response = await chatbotService.processMessage(userMessage.content);

      // Simulate typing delay
      setTimeout(() => {
        setIsTyping(false);

        const botMessage: ChatMessage = {
          id: (Date.now() + 1).toString(),
          type: "bot",
          content: response.message,
          timestamp: new Date(),
        };

        setMessages((prev) => [...prev, botMessage]);
        chatbotService.addMessageToHistory(botMessage);
        playNotificationSound();

        // Show confidence indicator for AI responses
        if (response.type === "ai" && response.confidence < 0.7) {
          toast.info(
            `💡 Confidence: ${Math.round(response.confidence * 100)}%`,
            {
              description:
                "Câu trả lời có thể chưa chính xác. Hãy hỏi cụ thể hơn.",
              duration: 3000,
            }
          );
        }
      }, 1000 + Math.random() * 1000); // 1-2 second delay
    } catch (error) {
      setIsTyping(false);
      console.error("Chat error:", error);
      toast.error("Có lỗi xảy ra khi gửi tin nhắn");
    } finally {
      setIsLoading(false);
    }
  };

  // Handle suggested question click
  const handleSuggestedQuestion = (question: string) => {
    setInputMessage(question);
    inputRef.current?.focus();
  };

  // Clear chat history
  const handleClearChat = () => {
    setMessages([]);
    chatbotService.clearHistory();
    // Silent clear - no toast notification needed
  };

  // Get suggested questions
  const suggestedQuestions = chatbotService.getSuggestedQuestions();

  // Handle Enter key
  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSendMessage();
    }
  };

  if (!isOpen) {
    return (
      <div className={`fixed bottom-4 right-4 z-50 ${className}`}>
        <Button
          onClick={handleToggle}
          size="lg"
          className="rounded-full w-14 h-14 shadow-lg hover:shadow-xl transition-all duration-300 bg-gradient-to-br from-blue-500 via-cyan-500 to-blue-600 hover:from-blue-600 hover:via-cyan-600 hover:to-emerald-500 hover:scale-110 active:scale-95 shadow-blue-500/25 hover:shadow-cyan-500/40"
        >
          <MessageCircle className="h-6 w-6" />
        </Button>
      </div>
    );
  }

  return (
    <div className={`fixed bottom-4 right-4 z-50 ${className}`}>
      <Card
        className={`w-96 shadow-2xl border-2 transition-all duration-300 ${
          isMinimized ? "h-16" : "h-[600px]"
        }`}
      >
        {/* Header */}
        <CardHeader className="pb-3 bg-gradient-to-br from-blue-500 via-cyan-500 to-blue-600 text-white rounded-t-lg shadow-inner">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-2">
              <div className="relative">
                <Bot className="h-6 w-6" />
                <div className="absolute -top-1 -right-1 w-3 h-3 bg-green-400 rounded-full border-2 border-white"></div>
              </div>
              <div>
                <CardTitle className="text-lg">AI Assistant</CardTitle>
                <p className="text-sm opacity-90">Trading Bot Helper</p>
              </div>
            </div>
            <div className="flex items-center space-x-1">
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setSoundEnabled(!soundEnabled)}
                className="text-white hover:bg-white/20"
              >
                {soundEnabled ? (
                  <Volume2 className="h-4 w-4" />
                ) : (
                  <VolumeX className="h-4 w-4" />
                )}
              </Button>
              <Button
                variant="ghost"
                size="sm"
                onClick={handleMinimize}
                className="text-white hover:bg-white/20"
              >
                {isMinimized ? (
                  <Maximize2 className="h-4 w-4" />
                ) : (
                  <Minimize2 className="h-4 w-4" />
                )}
              </Button>
              <Button
                variant="ghost"
                size="sm"
                onClick={handleToggle}
                className="text-white hover:bg-white/20"
              >
                <X className="h-4 w-4" />
              </Button>
            </div>
          </div>
        </CardHeader>

        {/* Content */}
        {!isMinimized && (
          <CardContent className="p-0 flex flex-col h-[calc(600px-80px)]">
            {/* Messages Area */}
            <ScrollArea className="flex-1 p-4">
              <div className="space-y-4">
                {messages.map((message) => (
                  <div
                    key={message.id}
                    className={`flex ${
                      message.type === "user" ? "justify-end" : "justify-start"
                    }`}
                  >
                    <div
                      className={`flex items-start space-x-2 max-w-[85%] ${
                        message.type === "user"
                          ? "flex-row-reverse space-x-reverse"
                          : "flex-row"
                      }`}
                    >
                      {/* Avatar */}
                      <div
                        className={`w-8 h-8 rounded-full flex items-center justify-center flex-shrink-0 ${
                          message.type === "user"
                            ? "bg-blue-500 text-white"
                            : "bg-gradient-to-br from-blue-500 via-cyan-500 to-blue-600 text-white shadow-lg shadow-blue-500/20"
                        }`}
                      >
                        {message.type === "user" ? (
                          <User className="h-4 w-4" />
                        ) : (
                          <Bot className="h-4 w-4" />
                        )}
                      </div>

                      {/* Message Bubble */}
                      <div
                        className={`rounded-lg p-3 ${
                          message.type === "user"
                            ? "bg-blue-500 text-white"
                            : "bg-secondary/50 text-foreground border"
                        }`}
                      >
                        <div className="text-sm whitespace-pre-wrap">
                          {message.content}
                        </div>
                        <div className="text-xs opacity-70 mt-1">
                          {message.timestamp.toLocaleTimeString("vi-VN", {
                            hour: "2-digit",
                            minute: "2-digit",
                          })}
                        </div>
                      </div>
                    </div>
                  </div>
                ))}

                {/* Typing Indicator */}
                <div className="flex justify-start">
                  <TypingIndicator isTyping={isTyping} />
                </div>

                {/* Suggested Questions */}
                {messages.length === 1 && (
                  <div className="border-t pt-4">
                    <div className="flex items-center space-x-2 mb-3">
                      <Sparkles className="h-4 w-4 text-cyan-500" />
                      <span className="text-sm font-medium text-muted-foreground">
                        Câu hỏi gợi ý:
                      </span>
                    </div>
                    <div className="space-y-2">
                      {suggestedQuestions.slice(0, 4).map((question, index) => (
                        <Button
                          key={index}
                          variant="outline"
                          size="sm"
                          className="w-full justify-start text-left h-auto p-2 text-xs"
                          onClick={() => handleSuggestedQuestion(question)}
                        >
                          <HelpCircle className="h-3 w-3 mr-2 flex-shrink-0" />
                          {question}
                        </Button>
                      ))}
                    </div>
                  </div>
                )}
              </div>
              <div ref={messagesEndRef} />
            </ScrollArea>

            {/* Input Area */}
            <div className="border-t p-4">
              <div className="flex items-center space-x-2">
                <div className="flex-1 relative">
                  <Input
                    ref={inputRef}
                    value={inputMessage}
                    onChange={(e) => setInputMessage(e.target.value)}
                    onKeyPress={handleKeyPress}
                    placeholder="Hỏi về trading bot..."
                    disabled={isLoading}
                    className="pr-10"
                  />
                  {messages.length > 1 && (
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={handleClearChat}
                      className="absolute right-2 top-1/2 -translate-y-1/2 h-6 w-6 p-0 hover:bg-destructive/10"
                    >
                      <Trash2 className="h-3 w-3" />
                    </Button>
                  )}
                </div>
                <Button
                  onClick={handleSendMessage}
                  disabled={!inputMessage.trim() || isLoading}
                  size="sm"
                  className="px-3"
                >
                  {isLoading ? (
                    <Loader2 className="h-4 w-4 animate-spin" />
                  ) : (
                    <Send className="h-4 w-4" />
                  )}
                </Button>
              </div>

              {/* Status */}
              <div className="flex items-center justify-between mt-2">
                <Badge variant="secondary" className="text-xs">
                  AI • Vietnamese
                </Badge>
                <div className="flex items-center space-x-2 text-xs text-muted-foreground">
                  <div className="w-2 h-2 bg-green-500 rounded-full"></div>
                  <span>Online</span>
                </div>
              </div>
            </div>
          </CardContent>
        )}
      </Card>
    </div>
  );
};

export default ChatBot;
