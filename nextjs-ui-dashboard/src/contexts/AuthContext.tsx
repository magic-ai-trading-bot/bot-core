import React, { createContext, useContext, useState, useEffect } from "react";
import logger from "@/utils/logger";
import {


// @spec:FR-AUTH-005 (Frontend) - `nextjs-ui-dashboard/src/contexts/AuthContext.tsx:78-234`
// @ref:specs/02-design/2.5-components/COMP-RUST-AUTH.md
// @test:

  BotCoreApiClient,
  LoginRequest,
  RegisterRequest,
  UserProfile,
} from "@/services/api";

interface AuthContextType {
  isAuthenticated: boolean;
  user: UserProfile | null;
  login: (email: string, password: string) => Promise<boolean>;
  register: (
    email: string,
    password: string,
    fullName?: string
  ) => Promise<boolean>;
  logout: () => void;
  loading: boolean;
  error: string | null;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

const apiClient = new BotCoreApiClient();

export const AuthProvider: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => {
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [user, setUser] = useState<UserProfile | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // Check if user is already logged in
    const initializeAuth = async () => {
      const token = apiClient.auth.getAuthToken();

      if (token && !apiClient.auth.isTokenExpired()) {
        try {
          const userProfile = await apiClient.auth.getProfile();
          setUser(userProfile);
          setIsAuthenticated(true);
        } catch (error) {
          logger.error("Failed to verify token:", error);
          apiClient.auth.removeAuthToken();
        }
      }
      setLoading(false);
    };

    initializeAuth();
  }, []);

  const login = async (email: string, password: string): Promise<boolean> => {
    setLoading(true);
    setError(null);

    try {
      const loginRequest: LoginRequest = { email, password };
      const response = await apiClient.auth.login(loginRequest);

      // Store token and user data
      apiClient.auth.setAuthToken(response.token);
      setUser(response.user);
      setIsAuthenticated(true);

      return true;
    } catch (error: unknown) {
      logger.error("Login failed:", error);
      setError(error instanceof Error ? error.message : "Login failed");
      return false;
    } finally {
      setLoading(false);
    }
  };

  const register = async (
    email: string,
    password: string,
    fullName?: string
  ): Promise<boolean> => {
    setLoading(true);
    setError(null);

    try {
      const registerRequest: RegisterRequest = {
        email,
        password,
        full_name: fullName,
      };
      const response = await apiClient.auth.register(registerRequest);

      // Store token and user data
      apiClient.auth.setAuthToken(response.token);
      setUser(response.user);
      setIsAuthenticated(true);

      return true;
    } catch (error: unknown) {
      logger.error("Registration failed:", error);
      setError(error instanceof Error ? error.message : "Registration failed");
      return false;
    } finally {
      setLoading(false);
    }
  };

  const logout = () => {
    apiClient.auth.removeAuthToken();
    setIsAuthenticated(false);
    setUser(null);
    setError(null);
  };

  return (
    <AuthContext.Provider
      value={{
        isAuthenticated,
        user,
        login,
        register,
        logout,
        loading,
        error,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
};

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return context;
};
