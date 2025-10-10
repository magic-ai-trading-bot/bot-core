import { useState } from "react";
import logger from "@/utils/logger";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { useNavigate, Link } from "react-router-dom";
import { useAuth } from "@/contexts/AuthContext";
import { useEffect } from "react";
import { toast } from "sonner";
import ChatBot from "@/components/ChatBot";

const Register = () => {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [fullName, setFullName] = useState("");
  const navigate = useNavigate();
  const { register, isAuthenticated, loading, error } = useAuth();

  // Redirect if already authenticated
  useEffect(() => {
    if (isAuthenticated) {
      navigate("/dashboard", { replace: true });
    }
  }, [isAuthenticated, navigate]);

  const handleRegister = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!email || !password || !confirmPassword) {
      toast.error("Lỗi đăng ký", {
        description: "Vui lòng điền đầy đủ thông tin",
      });
      return;
    }

    if (password !== confirmPassword) {
      toast.error("Lỗi đăng ký", {
        description: "Mật khẩu xác nhận không khớp",
      });
      return;
    }

    if (password.length < 6) {
      toast.error("Lỗi đăng ký", {
        description: "Mật khẩu phải có ít nhất 6 ký tự",
      });
      return;
    }

    try {
      toast.loading("Đang đăng ký...", { id: "register-loading" });

      const success = await register(email, password, fullName || undefined);

      if (success) {
        toast.success("Đăng ký thành công", {
          description: "Chào mừng bạn đến với Trading Bot Dashboard!",
          id: "register-loading",
        });
        navigate("/dashboard", { replace: true });
      } else {
        toast.error("Lỗi đăng ký", {
          description: error || "Không thể tạo tài khoản. Vui lòng thử lại.",
          id: "register-loading",
        });
      }
    } catch (err) {
      logger.error("Registration error:", err);
      toast.error("Lỗi đăng ký", {
        description: "Có lỗi xảy ra khi đăng ký. Vui lòng thử lại.",
        id: "register-loading",
      });
    }
  };

  return (
    <div className="min-h-screen bg-background flex items-center justify-center p-4">
      {/* Background Pattern */}
      <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_center,_var(--tw-gradient-stops))] from-primary/5 via-background to-background"></div>

      <div className="relative z-10 w-full max-w-sm md:max-w-md">
        {/* Logo/Brand */}
        <div className="text-center mb-6 md:mb-8">
          <div className="w-12 h-12 md:w-16 md:h-16 bg-gradient-to-br from-primary to-accent rounded-2xl flex items-center justify-center mx-auto mb-4">
            <span className="text-primary-foreground font-bold text-lg md:text-2xl">
              BT
            </span>
          </div>
          <h1 className="text-2xl md:text-3xl font-bold">Crypto Trading Bot</h1>
          <p className="text-muted-foreground mt-2 text-sm md:text-base">
            Tạo tài khoản để bắt đầu giao dịch
          </p>
        </div>

        <Card className="shadow-2xl border-border/50 bg-card/80 backdrop-blur">
          <CardHeader className="space-y-1">
            <CardTitle className="text-2xl text-center">Đăng ký</CardTitle>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleRegister} className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="fullName">Họ và tên (tùy chọn)</Label>
                <Input
                  id="fullName"
                  type="text"
                  placeholder="Nguyễn Văn A"
                  value={fullName}
                  onChange={(e) => setFullName(e.target.value)}
                  className="bg-background/50"
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="email">Email</Label>
                <Input
                  id="email"
                  type="email"
                  placeholder="your@email.com"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  className="bg-background/50"
                  required
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="password">Mật khẩu</Label>
                <Input
                  id="password"
                  type="password"
                  placeholder="Nhập mật khẩu của bạn"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  className="bg-background/50"
                  required
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="confirmPassword">Xác nhận mật khẩu</Label>
                <Input
                  id="confirmPassword"
                  type="password"
                  placeholder="Nhập lại mật khẩu"
                  value={confirmPassword}
                  onChange={(e) => setConfirmPassword(e.target.value)}
                  className="bg-background/50"
                  required
                />
              </div>

              <Button
                type="submit"
                className="w-full bg-profit hover:bg-profit/90 text-profit-foreground"
                disabled={loading}
              >
                {loading ? "Đang đăng ký..." : "Đăng ký"}
              </Button>
            </form>

            {/* Login Link */}
            <div className="mt-6 text-center">
              <p className="text-sm text-muted-foreground">
                Đã có tài khoản?{" "}
                <Link
                  to="/login"
                  className="text-primary hover:underline font-medium"
                >
                  Đăng nhập ngay
                </Link>
              </p>
            </div>

            {/* Features Preview */}
            <div className="mt-6 space-y-3">
              <div className="flex items-center gap-3 text-sm">
                <div className="w-2 h-2 bg-profit rounded-full"></div>
                <span>AI-Powered Trading Signals</span>
              </div>
              <div className="flex items-center gap-3 text-sm">
                <div className="w-2 h-2 bg-info rounded-full"></div>
                <span>Real-time Performance Analytics</span>
              </div>
              <div className="flex items-center gap-3 text-sm">
                <div className="w-2 h-2 bg-warning rounded-full"></div>
                <span>Advanced Risk Management</span>
              </div>
            </div>
          </CardContent>
        </Card>

        <div className="text-center mt-6 text-sm text-muted-foreground">
          <p>Bảo mật với mã hóa end-to-end và xác thực 2FA</p>
        </div>
      </div>

      {/* Chatbot Widget */}
      <ChatBot />
    </div>
  );
};

export default Register;
