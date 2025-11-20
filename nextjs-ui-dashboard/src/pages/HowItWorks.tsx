/**
 * How It Works Page
 *
 * Trang giải thích cách bot hoạt động cho user
 * Hiển thị thông tin dễ hiểu với visual elements
 */

import React, { useState } from 'react';
import ErrorBoundary from '@/components/ErrorBoundary';
import { DashboardHeader } from '@/components/dashboard/DashboardHeader';
import { useAuth } from '@/contexts/AuthContext';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Progress } from '@/components/ui/progress';
import {
  Database,
  TrendingUp,
  Brain,
  Shield,
  AlertTriangle,
  CheckCircle,
  Info,
  Play,
  Pause,
  DollarSign,
  BarChart3,
  Zap
} from 'lucide-react';

const HowItWorks = () => {
  const { user } = useAuth();
  const [activeStep, setActiveStep] = useState(0);

  const steps = [
    {
      number: 1,
      title: 'Thu Thập Dữ Liệu',
      icon: <Database className="h-8 w-8" />,
      color: 'text-info',
      description: 'Bot thu thập dữ liệu thị trường từ Binance mỗi giây',
      details: [
        'Giá OHLC (Open, High, Low, Close)',
        'Khối lượng giao dịch',
        'Biến động theo thời gian thực',
        'Theo dõi khung 1h và 4h'
      ]
    },
    {
      number: 2,
      title: 'Phân Tích Kỹ Thuật',
      icon: <BarChart3 className="h-8 w-8" />,
      color: 'text-profit',
      description: 'Bot phân tích dữ liệu bằng 5 chiến lược + AI',
      details: [
        'RSI: Phát hiện quá mua/quá bán (65% win rate - optimized)',
        'MACD: Xu hướng và đảo chiều (61% win rate - optimized)',
        'Bollinger Bands: Biến động và breakout (63% win rate - optimized)',
        'Volume: Độ mạnh xu hướng (58% win rate - optimized)',
        'Stochastic: Crossover %K/%D (64% win rate - NEW!)',
        'AI/ML: Dự đoán giá (72% accuracy)'
      ]
    },
    {
      number: 3,
      title: 'Tạo Tín Hiệu',
      icon: <Brain className="h-8 w-8" />,
      color: 'text-primary',
      description: 'Bot tạo tín hiệu MUA/BÁN dựa trên phân tích',
      details: [
        'Tần suất: Mỗi 60 phút (có thể điều chỉnh)',
        'Yêu cầu: ≥3/5 chiến lược đồng ý',
        'Độ tin cậy: 60-100%',
        'Xác nhận đa chiều (multi-confirmation)'
      ]
    },
    {
      number: 4,
      title: 'Giao Dịch An Toàn',
      icon: <Shield className="h-8 w-8" />,
      color: 'text-destructive',
      description: 'Bot kiểm tra 7 lớp rủi ro trước khi vào lệnh',
      details: [
        '✅ Rủi ro mỗi lệnh ≤2%',
        '✅ Rủi ro danh mục ≤10%',
        '✅ Stop loss bắt buộc',
        '✅ Daily loss limit 5%',
        '✅ Cool-down sau thua lỗ',
        '✅ Position correlation 70%',
        '✅ Trailing stop tự động'
      ]
    }
  ];

  const strategies = [
    {
      name: 'RSI Strategy',
      winRate: 65,
      description: 'Phát hiện quá mua/quá bán',
      icon: '📊',
      signals: {
        buy: 'RSI < 25 (quá bán - optimized)',
        sell: 'RSI > 75 (quá mua - optimized)'
      }
    },
    {
      name: 'MACD Strategy',
      winRate: 61,
      description: 'Phát hiện xu hướng và đảo chiều',
      icon: '📈',
      signals: {
        buy: 'MACD cắt lên Signal (fast: 10/22 - optimized)',
        sell: 'MACD cắt xuống Signal (fast: 10/22 - optimized)'
      }
    },
    {
      name: 'Bollinger Bands',
      winRate: 63,
      description: 'Phát hiện biến động và breakout',
      icon: '📉',
      signals: {
        buy: 'Giá chạm dải dưới (period: 15, std: 2.5 - optimized)',
        sell: 'Giá chạm dải trên (period: 15, std: 2.5 - optimized)'
      }
    },
    {
      name: 'Volume Strategy',
      winRate: 58,
      description: 'Xác nhận độ mạnh xu hướng',
      icon: '📊',
      signals: {
        buy: 'Volume spike 1.8x + giá tăng (optimized)',
        sell: 'Volume spike 1.8x + giá giảm (optimized)'
      }
    },
    {
      name: 'Stochastic Strategy',
      winRate: 64,
      description: 'Phát hiện crossover %K/%D',
      icon: '🎯',
      signals: {
        buy: '%K cắt lên %D trong vùng oversold (<15) - NEW!',
        sell: '%K cắt xuống %D trong vùng overbought (>85) - NEW!'
      }
    }
  ];

  const riskLayers = [
    {
      layer: 1,
      name: 'Position Size',
      description: 'Rủi ro mỗi lệnh ≤2%',
      example: 'Tài khoản $10,000 → Rủi ro tối đa $200/lệnh'
    },
    {
      layer: 2,
      name: 'Stop Loss',
      description: 'Stop loss bắt buộc (2%)',
      example: 'Giá vào $45,000 → SL tại $44,100'
    },
    {
      layer: 3,
      name: 'Portfolio Risk',
      description: 'Rủi ro tổng ≤10%',
      example: 'Tối đa 5 lệnh mở cùng lúc (2% × 5 = 10%)'
    },
    {
      layer: 4,
      name: 'Daily Loss Limit',
      description: 'Thua lỗ trong ngày ≤5%',
      example: 'Thua $500 → Bot nghỉ đến ngày mai'
    },
    {
      layer: 5,
      name: 'Consecutive Losses',
      description: 'Tối đa 5 lệnh thua liên tiếp',
      example: 'Sau 5 lệnh thua → Cool-down 60 phút'
    },
    {
      layer: 6,
      name: 'Cool-Down Period',
      description: 'Nghỉ 60 phút sau thua lỗ',
      example: 'Tránh giao dịch cảm tính'
    },
    {
      layer: 7,
      name: 'Position Correlation',
      description: 'Giới hạn tương quan 70%',
      example: 'Phân tán rủi ro, không all-in 1 chiều'
    }
  ];

  const signalQuality = [
    {
      level: 'Mạnh',
      confidence: '80-100%',
      criteria: '5/5 chiến lược đồng ý',
      color: 'bg-profit/10 text-profit border-profit/20'
    },
    {
      level: 'Trung Bình',
      confidence: '60-79%',
      criteria: '3-4/5 chiến lược đồng ý',
      color: 'bg-warning/10 text-warning border-warning/20'
    },
    {
      level: 'Yếu',
      confidence: '<60%',
      criteria: '<3/5 chiến lược đồng ý',
      color: 'bg-muted text-muted-foreground border-border'
    }
  ];

  return (
    <ErrorBoundary>
      <div className="min-h-screen bg-background">
        {/* Show header if user is logged in */}
        {user && <DashboardHeader />}

        <div className="p-4 lg:p-6 space-y-6">
          {/* Hero Section */}
          <Card className="bg-gradient-to-r from-primary to-accent text-primary-foreground">
            <CardHeader>
              <CardTitle className="text-3xl">🤖 Bot Trading Tự Động</CardTitle>
              <CardDescription className="text-primary-foreground/90 text-lg">
                Giao dịch cryptocurrency 24/7 với trí tuệ nhân tạo
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                <div className="text-center">
                  <div className="text-3xl font-bold">72%</div>
                  <div className="text-sm text-primary-foreground/80">Độ chính xác AI</div>
                </div>
                <div className="text-center">
                  <div className="text-3xl font-bold">7</div>
                  <div className="text-sm text-primary-foreground/80">Lớp bảo vệ rủi ro</div>
                </div>
                <div className="text-center">
                  <div className="text-3xl font-bold">24/7</div>
                  <div className="text-sm text-primary-foreground/80">Hoạt động liên tục</div>
                </div>
                <div className="text-center">
                  <div className="text-3xl font-bold">0</div>
                  <div className="text-sm text-primary-foreground/80">Cảm xúc con người</div>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* 4 Steps Process */}
          <Card>
            <CardHeader>
              <CardTitle>🎯 Bot Hoạt Động Như Thế Nào? (4 Bước)</CardTitle>
              <CardDescription>
                Quy trình tự động phân tích và giao dịch của bot
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
                {steps.map((step, index) => (
                  <Card
                    key={step.number}
                    className={`cursor-pointer transition-all ${
                      activeStep === index ? 'ring-2 ring-primary shadow-lg' : ''
                    }`}
                    onClick={() => setActiveStep(index)}
                  >
                    <CardHeader>
                      <div className="flex items-center gap-3">
                        <div className={`${step.color}`}>{step.icon}</div>
                        <div>
                          <Badge>Bước {step.number}</Badge>
                          <CardTitle className="text-lg mt-1">{step.title}</CardTitle>
                        </div>
                      </div>
                    </CardHeader>
                    <CardContent>
                      <p className="text-sm text-muted-foreground">{step.description}</p>
                    </CardContent>
                  </Card>
                ))}
              </div>

              {/* Step Details */}
              <Card className="bg-primary/10 border-primary/20">
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    {steps[activeStep].icon}
                    <span className={steps[activeStep].color}>Bước {steps[activeStep].number}: {steps[activeStep].title}</span>
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <ul className="space-y-2">
                    {steps[activeStep].details.map((detail, index) => (
                      <li key={index} className="flex items-start gap-2">
                        <CheckCircle className="h-5 w-5 text-profit mt-0.5" />
                        <span>{detail}</span>
                      </li>
                    ))}
                  </ul>
                </CardContent>
              </Card>
            </CardContent>
          </Card>

          {/* Trading Strategies */}
          <Card>
            <CardHeader>
              <CardTitle>📊 5 Chiến Lược Giao Dịch</CardTitle>
              <CardDescription>
                Bot sử dụng 5 chiến lược kỹ thuật được tối ưu hóa cho thị trường crypto
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4">
                {strategies.map((strategy) => (
                  <Card key={strategy.name} className="border-2">
                    <CardHeader>
                      <div className="text-center text-4xl mb-2">{strategy.icon}</div>
                      <CardTitle className="text-center text-lg">{strategy.name}</CardTitle>
                      <div className="flex justify-center items-center gap-2">
                        <Badge className="bg-profit/10 text-profit border-profit/20">
                          Win Rate: {strategy.winRate}%
                        </Badge>
                      </div>
                    </CardHeader>
                    <CardContent className="space-y-3">
                      <p className="text-sm text-muted-foreground text-center">
                        {strategy.description}
                      </p>
                      <div className="space-y-2 text-sm">
                        <div className="flex items-center gap-2">
                          <TrendingUp className="h-4 w-4 text-profit" />
                          <span className="text-profit">Mua:</span>
                          <span className="text-xs">{strategy.signals.buy}</span>
                        </div>
                        <div className="flex items-center gap-2">
                          <TrendingUp className="h-4 w-4 text-loss rotate-180" />
                          <span className="text-loss">Bán:</span>
                          <span className="text-xs">{strategy.signals.sell}</span>
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            </CardContent>
          </Card>

          {/* Risk Management */}
          <Card>
            <CardHeader>
              <CardTitle>🛡️ 7 Lớp Bảo Vệ Rủi Ro</CardTitle>
              <CardDescription>
                Hệ thống quản lý rủi ro toàn diện bảo vệ tài khoản của bạn
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {riskLayers.map((layer) => (
                  <Card key={layer.layer} className="border-l-4 border-l-primary">
                    <CardHeader>
                      <div className="flex items-center justify-between">
                        <div>
                          <Badge variant="outline">Lớp {layer.layer}</Badge>
                          <CardTitle className="text-lg mt-1">{layer.name}</CardTitle>
                        </div>
                        <Shield className="h-8 w-8 text-primary" />
                      </div>
                    </CardHeader>
                    <CardContent>
                      <p className="text-sm mb-2">{layer.description}</p>
                      <Alert>
                        <Info className="h-4 w-4" />
                        <AlertDescription>
                          <strong>Ví dụ:</strong> {layer.example}
                        </AlertDescription>
                      </Alert>
                    </CardContent>
                  </Card>
                ))}
              </div>
            </CardContent>
          </Card>

          {/* Signal Quality Levels */}
          <Card>
            <CardHeader>
              <CardTitle>🎯 3 Mức Độ Tín Hiệu</CardTitle>
              <CardDescription>
                Bot chỉ giao dịch khi có đủ độ tin cậy
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                {signalQuality.map((signal) => (
                  <Card key={signal.level} className={`border-2 ${signal.color}`}>
                    <CardHeader>
                      <CardTitle className="text-center">{signal.level}</CardTitle>
                      <div className="text-center text-2xl font-bold">{signal.confidence}</div>
                    </CardHeader>
                    <CardContent className="text-center">
                      <p className="text-sm">{signal.criteria}</p>
                    </CardContent>
                  </Card>
                ))}
              </div>
              <Alert className="mt-6">
                <AlertTriangle className="h-4 w-4" />
                <AlertDescription>
                  <strong>Lưu ý:</strong> Bot sẽ bỏ qua tín hiệu YẾU để tránh rủi ro. Chỉ giao dịch khi tín hiệu MẠNH hoặc TRUNG BÌNH.
                </AlertDescription>
              </Alert>
            </CardContent>
          </Card>

          {/* Trailing Stop Example */}
          <Card>
            <CardHeader>
              <CardTitle>💰 Ví Dụ: Trailing Stop Tự Động</CardTitle>
              <CardDescription>
                Cách bot bảo vệ lợi nhuận của bạn
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                <div className="grid grid-cols-1 md:grid-cols-5 gap-4">
                  <Card className="bg-muted">
                    <CardContent className="p-4 text-center">
                      <div className="text-sm text-muted-foreground mb-1">Giá vào</div>
                      <div className="text-xl font-bold">$45,000</div>
                      <Play className="h-4 w-4 mx-auto mt-2 text-info" />
                    </CardContent>
                  </Card>
                  <Card className="bg-profit/10">
                    <CardContent className="p-4 text-center">
                      <div className="text-sm text-muted-foreground mb-1">Lên +5%</div>
                      <div className="text-xl font-bold text-profit">$47,250</div>
                      <TrendingUp className="h-4 w-4 mx-auto mt-2 text-profit" />
                    </CardContent>
                  </Card>
                  <Card className="bg-warning/10">
                    <CardContent className="p-4 text-center">
                      <div className="text-sm text-muted-foreground mb-1">Trailing kích hoạt</div>
                      <div className="text-xl font-bold text-warning">$45,832</div>
                      <Zap className="h-4 w-4 mx-auto mt-2 text-warning" />
                    </CardContent>
                  </Card>
                  <Card className="bg-profit/10">
                    <CardContent className="p-4 text-center">
                      <div className="text-sm text-muted-foreground mb-1">Giá cao nhất</div>
                      <div className="text-xl font-bold text-profit">$48,000</div>
                      <TrendingUp className="h-4 w-4 mx-auto mt-2 text-profit" />
                    </CardContent>
                  </Card>
                  <Card className="bg-profit/20 border-2 border-profit">
                    <CardContent className="p-4 text-center">
                      <div className="text-sm text-muted-foreground mb-1">Chốt lời tại</div>
                      <div className="text-xl font-bold text-profit">$46,560</div>
                      <CheckCircle className="h-4 w-4 mx-auto mt-2 text-profit" />
                    </CardContent>
                  </Card>
                </div>
                <Alert className="bg-profit/10 border-profit/20">
                  <CheckCircle className="h-4 w-4 text-profit" />
                  <AlertDescription className="text-profit">
                    <strong>Kết quả:</strong> Lợi nhuận +3.47% ($1,560) được bảo vệ tự động ngay cả khi giá giảm từ đỉnh!
                  </AlertDescription>
                </Alert>
              </div>
            </CardContent>
          </Card>

          {/* CTA Section */}
          <Card className="bg-gradient-to-r from-primary to-accent text-primary-foreground">
            <CardHeader>
              <CardTitle className="text-2xl text-center">🚀 Bắt Đầu Paper Trading Ngay!</CardTitle>
              <CardDescription className="text-primary-foreground/90 text-center text-lg">
                Thử nghiệm bot với $10,000 ảo - Không rủi ro, Không mất tiền
              </CardDescription>
            </CardHeader>
            <CardContent className="text-center space-y-4">
              <div className="flex flex-wrap justify-center gap-4">
                <Badge className="bg-primary-foreground/10 text-primary-foreground border-primary-foreground/20 text-sm py-2 px-4">
                  ✅ Miễn phí 100%
                </Badge>
                <Badge className="bg-primary-foreground/10 text-primary-foreground border-primary-foreground/20 text-sm py-2 px-4">
                  ✅ Không cần KYC
                </Badge>
                <Badge className="bg-primary-foreground/10 text-primary-foreground border-primary-foreground/20 text-sm py-2 px-4">
                  ✅ Dữ liệu thật từ Binance
                </Badge>
              </div>
              {!user && (
                <div className="pt-4">
                  <a href="/register" className="inline-block bg-primary-foreground text-primary px-8 py-3 rounded-lg font-semibold hover:opacity-90 transition-opacity">
                    Đăng Ký Ngay →
                  </a>
                </div>
              )}
            </CardContent>
          </Card>
        </div>
      </div>
    </ErrorBoundary>
  );
};

export default HowItWorks;
