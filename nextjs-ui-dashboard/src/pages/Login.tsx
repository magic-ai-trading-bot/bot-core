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
import { Logo } from "@/components/ui/Logo";

const Login = () => {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const navigate = useNavigate();
  const { login, isAuthenticated, loading, error } = useAuth();

  // Redirect if already authenticated
  useEffect(() => {
    if (isAuthenticated) {
      navigate("/dashboard", { replace: true });
    }
  }, [isAuthenticated, navigate]);

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!email || !password) {
      toast.error("Lỗi đăng nhập", {
        description: "Vui lòng nhập email và mật khẩu",
      });
      return;
    }

    try {
      toast.loading("Đang đăng nhập...", { id: "login-loading" });

      const success = await login(email, password);

      if (success) {
        toast.success("Đăng nhập thành công", {
          description: "Chào mừng trở lại với Trading Bot Dashboard!",
          id: "login-loading",
        });
        navigate("/dashboard", { replace: true });
      } else {
        toast.error("Lỗi đăng nhập", {
          description: error || "Thông tin đăng nhập không chính xác",
          id: "login-loading",
        });
      }
    } catch (err) {
      logger.error("Login error:", err);
      toast.error("Lỗi đăng nhập", {
        description: "Có lỗi xảy ra khi đăng nhập. Vui lòng thử lại.",
        id: "login-loading",
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
          <div className="flex justify-center mb-4">
            <Logo size="xl" showText={false} />
          </div>
          <h1 className="text-2xl md:text-3xl font-extrabold">
            <span className="text-foreground">Bot</span>
            <span className="bg-gradient-to-r from-emerald-400 to-teal-400 bg-clip-text text-transparent">Core</span>
          </h1>
          <p className="text-muted-foreground mt-2 text-sm md:text-base">
            Đăng nhập để quản lý bot trading của bạn
          </p>
        </div>

        <Card className="shadow-2xl border-border/50 bg-card/80 backdrop-blur">
          <CardHeader className="space-y-1">
            <CardTitle className="text-2xl text-center">Đăng nhập</CardTitle>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleLogin} className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="email">Email</Label>
                <Input
                  id="email"
                  type="email"
                  placeholder="trader@botcore.com"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  className="bg-background/50"
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
                />
              </div>

              <Button
                type="submit"
                className="w-full bg-profit hover:bg-profit/90 text-profit-foreground"
                disabled={loading}
              >
                {loading ? "Đang đăng nhập..." : "Đăng nhập"}
              </Button>
            </form>

            {/* Register Link */}
            <div className="mt-6 text-center">
              <p className="text-sm text-muted-foreground">
                Chưa có tài khoản?{" "}
                <Link
                  to="/register"
                  className="text-primary hover:underline font-medium"
                >
                  Đăng ký ngay
                </Link>
              </p>
            </div>

            {/* Demo Credentials */}
            <div className="mt-4 p-4 bg-muted/50 rounded-lg border border-border/50">
              <p className="text-sm text-muted-foreground mb-2">
                Demo credentials:
              </p>
              <div className="text-xs space-y-1">
                <p>
                  <strong>Email:</strong> trader@botcore.com
                </p>
                <p>
                  <strong>Password:</strong> password123
                </p>
              </div>
              <div className="mt-3 flex gap-2">
                <button
                  type="button"
                  onClick={() => {
                    setEmail('trader@botcore.com');
                    setPassword('password123');
                  }}
                  className="flex-1 px-3 py-1.5 text-xs bg-primary/10 hover:bg-primary/20 rounded border border-primary/30 transition-colors"
                >
                  Use Trader
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setEmail('admin@botcore.com');
                    setPassword('password123');
                  }}
                  className="flex-1 px-3 py-1.5 text-xs bg-accent/10 hover:bg-accent/20 rounded border border-accent/30 transition-colors"
                >
                  Use Admin
                </button>
              </div>
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

export default Login;
