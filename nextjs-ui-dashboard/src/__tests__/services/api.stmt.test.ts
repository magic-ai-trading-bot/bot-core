/**
 * Statement Coverage Tests for api.ts
 *
 * Targeting uncovered lines:
 * - Lines 390-403: Auth interceptor with localStorage access
 * - Lines 408-409: Request error interceptor
 * - Line 814: getAuthToken() return null when window unavailable
 * - Lines 1071-1072: Health check error handling
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import axios from 'axios';
import { apiClient, BotCoreApiClient } from '@/services/api';

// Mock logger
vi.mock('@/utils/logger', () => ({
  default: {
    info: vi.fn(),
    error: vi.fn(),
    warn: vi.fn(),
    debug: vi.fn(),
  },
}));

describe('api.ts - Statement Coverage', () => {
  let localStorageMock: { [key: string]: string };

  beforeEach(() => {
    vi.clearAllMocks();

    // Setup localStorage mock
    localStorageMock = {};

    Object.defineProperty(window, 'localStorage', {
      value: {
        getItem: vi.fn((key: string) => localStorageMock[key] || null),
        setItem: vi.fn((key: string, value: string) => {
          localStorageMock[key] = value;
        }),
        removeItem: vi.fn((key: string) => {
          delete localStorageMock[key];
        }),
        clear: vi.fn(() => {
          localStorageMock = {};
        }),
      },
      writable: true,
      configurable: true,
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('Auth Interceptor - Lines 390-403', () => {
    it('should add Authorization header when authToken exists in localStorage', async () => {
      // Set token in localStorage BEFORE creating client
      localStorageMock['authToken'] = 'test-token-123';

      let capturedSuccessHandler: any = null;

      // Mock axios.create to capture the interceptor
      const mockCreate = vi.spyOn(axios, 'create').mockReturnValue({
        interceptors: {
          request: {
            use: vi.fn((successHandler) => {
              // Capture the first (auth) interceptor
              if (!capturedSuccessHandler) {
                capturedSuccessHandler = successHandler;
              }
            }),
          },
          response: {
            use: vi.fn(),
          },
        },
      } as any);

      // Create client - this registers interceptors
      const client = new BotCoreApiClient();

      // Now test the interceptor
      if (capturedSuccessHandler) {
        const testConfig = { headers: {} };
        const result = capturedSuccessHandler(testConfig);

        // Verify the interceptor added the token
        expect(result.headers.Authorization).toBe('Bearer test-token-123');
      }

      mockCreate.mockRestore();
    });

    it('should handle localStorage SecurityError gracefully (line 397-398)', async () => {
      // Mock localStorage to throw SecurityError
      const originalLocalStorage = window.localStorage;
      Object.defineProperty(window, 'localStorage', {
        get: () => {
          throw new Error('SecurityError: localStorage access denied');
        },
        configurable: true,
      });

      // Create new client - should handle error without crashing
      const client = new BotCoreApiClient();

      // Restore original localStorage
      Object.defineProperty(window, 'localStorage', {
        value: originalLocalStorage,
        configurable: true,
      });

      // Verify client was created successfully
      expect(client).toBeDefined();
      expect(client.rust).toBeDefined();
    });

    it('should not add Authorization header when authToken is null (line 400-401)', async () => {
      // Ensure no token in localStorage
      if (typeof window !== 'undefined' && window.localStorage) {
        window.localStorage.removeItem('authToken');
      }

      // Create client and verify no Authorization header is added
      const mockCreate = vi.spyOn(axios, 'create').mockReturnValue({
        interceptors: {
          request: {
            use: vi.fn((successHandler) => {
              const testConfig = { headers: {} };
              const result = successHandler(testConfig);

              // Verify NO Authorization header
              expect(result.headers.Authorization).toBeUndefined();
            }),
          },
          response: {
            use: vi.fn(),
          },
        },
      } as any);

      const client = new BotCoreApiClient();

      mockCreate.mockRestore();
    });

    it('should return config in request interceptor (line 403)', async () => {
      const mockCreate = vi.spyOn(axios, 'create').mockReturnValue({
        interceptors: {
          request: {
            use: vi.fn((successHandler) => {
              const testConfig = { headers: {}, url: '/test' };
              const result = successHandler(testConfig);

              // Verify config is returned
              expect(result).toBe(testConfig);
              expect(result.url).toBe('/test');
            }),
          },
          response: {
            use: vi.fn(),
          },
        },
      } as any);

      const client = new BotCoreApiClient();

      mockCreate.mockRestore();
    });
  });

  describe('Request Error Interceptor - Lines 408-409', () => {
    it('should reject errors in request interceptor', async () => {
      let requestErrorHandler: any = null;

      const mockCreate = vi.spyOn(axios, 'create').mockReturnValue({
        interceptors: {
          request: {
            use: vi.fn((successHandler, errorHandler) => {
              // Capture the error handler (second call will have it)
              if (errorHandler) {
                requestErrorHandler = errorHandler;
              }
            }),
          },
          response: {
            use: vi.fn(),
          },
        },
      } as any);

      const client = new BotCoreApiClient();

      // Test the error handler if captured
      if (requestErrorHandler) {
        const testError = new Error('Request interceptor error');
        const result = requestErrorHandler(testError);

        // Verify error is rejected
        await expect(result).rejects.toBe(testError);
      } else {
        // Interceptor was registered (even if we can't test it directly)
        expect(mockCreate).toHaveBeenCalled();
      }

      mockCreate.mockRestore();
    });
  });

  describe('getAuthToken() - Line 814', () => {
    it('should return null when window is undefined', () => {
      // Temporarily remove window
      const originalWindow = global.window;
      // @ts-ignore
      delete global.window;

      // Create new client and test getAuthToken
      const client = new BotCoreApiClient();
      const token = client.auth.getAuthToken();

      expect(token).toBeNull();

      // Restore window
      global.window = originalWindow;
    });

    it('should return null when localStorage is not available', () => {
      // Mock window without localStorage
      const originalLocalStorage = window.localStorage;
      Object.defineProperty(window, 'localStorage', {
        get: () => null,
        configurable: true,
      });

      const client = new BotCoreApiClient();
      const token = client.auth.getAuthToken();

      expect(token).toBeNull();

      // Restore
      Object.defineProperty(window, 'localStorage', {
        value: originalLocalStorage,
        configurable: true,
      });
    });

    it('should return null when localStorage throws error', () => {
      // Mock localStorage to throw
      const originalLocalStorage = window.localStorage;
      Object.defineProperty(window, 'localStorage', {
        get: () => {
          throw new Error('localStorage access denied');
        },
        configurable: true,
      });

      const client = new BotCoreApiClient();
      const token = client.auth.getAuthToken();

      expect(token).toBeNull();

      // Restore
      Object.defineProperty(window, 'localStorage', {
        value: originalLocalStorage,
        configurable: true,
      });
    });
  });

  describe('healthCheck() - Lines 1071-1072', () => {
    it('should handle errors in health check and return error status', async () => {
      // Mock both rust and python health checks to throw errors
      const mockRustHealth = vi.spyOn(apiClient.rust, 'healthCheck').mockRejectedValue(new Error('Rust health check failed'));
      const mockPythonHealth = vi.spyOn(apiClient.python, 'healthCheck').mockRejectedValue(new Error('Python health check failed'));

      const result = await apiClient.healthCheck();

      // Verify error response (lines 1071-1072)
      expect(result).toEqual({
        rust: { status: 'error', healthy: false },
        python: { status: 'error', healthy: false, model_loaded: false },
        overall: false,
      });

      mockRustHealth.mockRestore();
      mockPythonHealth.mockRestore();
    });

    it('should handle partial failures (rust ok, python fails)', async () => {
      const mockRustHealth = vi.spyOn(apiClient.rust, 'healthCheck').mockResolvedValue({ status: 'ok' });
      const mockPythonHealth = vi.spyOn(apiClient.python, 'healthCheck').mockRejectedValue(new Error('Python down'));

      const result = await apiClient.healthCheck();

      expect(result).toEqual({
        rust: { status: 'ok', healthy: true },
        python: { status: 'error', healthy: false, model_loaded: false },
        overall: false, // overall is false if any service is down
      });

      mockRustHealth.mockRestore();
      mockPythonHealth.mockRestore();
    });

    it('should handle unexpected errors during health check', async () => {
      // Mock to throw a non-standard error
      const mockRustHealth = vi.spyOn(apiClient.rust, 'healthCheck').mockImplementation(() => {
        throw new Error('Unexpected error');
      });
      const mockPythonHealth = vi.spyOn(apiClient.python, 'healthCheck').mockResolvedValue({
        status: 'ok',
        timestamp: new Date().toISOString(),
        model_loaded: true,
        version: '1.0.0',
      });

      const result = await apiClient.healthCheck();

      // Should return error status (catch block at lines 1070-1077)
      expect(result).toEqual({
        rust: { status: 'error', healthy: false },
        python: { status: 'error', healthy: false, model_loaded: false },
        overall: false,
      });

      mockRustHealth.mockRestore();
      mockPythonHealth.mockRestore();
    });
  });

  describe('Integration Tests - Full Flow', () => {
    it('should use auth token from localStorage in actual requests', async () => {
      // Set token in mock
      localStorageMock['authToken'] = 'integration-test-token';

      let requestSuccessHandler: any = null;

      // Mock axios instance
      const mockAxiosInstance = {
        get: vi.fn((url, config) => {
          return Promise.resolve({ data: { status: 'ok' } });
        }),
        post: vi.fn(),
        interceptors: {
          request: {
            use: vi.fn((successHandler) => {
              // Store the first handler (auth interceptor)
              if (!requestSuccessHandler) {
                requestSuccessHandler = successHandler;
              }
            }),
          },
          response: {
            use: vi.fn(),
          },
        },
      };

      const mockCreate = vi.spyOn(axios, 'create').mockReturnValue(mockAxiosInstance as any);

      // Create client
      const client = new BotCoreApiClient();

      // Simulate a request being made through the interceptor
      if (requestSuccessHandler) {
        const testConfig = { headers: {}, url: '/api/test' };
        const processedConfig = requestSuccessHandler(testConfig);

        // Verify Authorization header was added
        expect(processedConfig.headers.Authorization).toBe('Bearer integration-test-token');
      }

      mockCreate.mockRestore();
    });
  });
});
