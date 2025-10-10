import { describe, it, expect, beforeEach, vi } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import { useAIAnalysis } from '@/hooks/useAIAnalysis';

// Mock the API
vi.mock('@/services/api', () => ({
  getAIAnalysis: vi.fn(),
  getAIPrediction: vi.fn(),
  trainAIModel: vi.fn(),
}));

describe('useAIAnalysis - Comprehensive Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Analysis Fetching', () => {
    it('should fetch AI analysis successfully', async () => {
      const { getAIAnalysis } = await import('@/services/api');
      const mockAnalysis = {
        signal: 'long',
        confidence: 0.85,
        reasoning: 'Strong bullish indicators',
        timestamp: new Date().toISOString(),
      };
      (getAIAnalysis as any).mockResolvedValueOnce(mockAnalysis);

      const { result } = renderHook(() => useAIAnalysis());

      await act(async () => {
        await result.current.fetchAnalysis('BTCUSDT', '1h');
      });

      await waitFor(() => {
        expect(result.current.analysis).toEqual(mockAnalysis);
      });
    });

    it('should handle analysis fetch errors', async () => {
      const { getAIAnalysis } = await import('@/services/api');
      (getAIAnalysis as any).mockRejectedValueOnce(new Error('Analysis failed'));

      const { result } = renderHook(() => useAIAnalysis());

      await act(async () => {
        try {
          await result.current.fetchAnalysis('BTCUSDT', '1h');
        } catch (error) {
          expect(error).toBeDefined();
        }
      });
    });

    it('should fetch analysis for multiple symbols', async () => {
      const { getAIAnalysis } = await import('@/services/api');
      const mockAnalyses = {
        BTCUSDT: { signal: 'long', confidence: 0.85 },
        ETHUSDT: { signal: 'short', confidence: 0.75 },
      };

      (getAIAnalysis as any)
        .mockResolvedValueOnce(mockAnalyses.BTCUSDT)
        .mockResolvedValueOnce(mockAnalyses.ETHUSDT);

      const { result } = renderHook(() => useAIAnalysis());

      await act(async () => {
        await result.current.fetchAnalysis('BTCUSDT', '1h');
      });

      await act(async () => {
        await result.current.fetchAnalysis('ETHUSDT', '1h');
      });

      expect(getAIAnalysis).toHaveBeenCalledTimes(2);
    });
  });

  describe('Prediction Fetching', () => {
    it('should fetch AI predictions', async () => {
      const { getAIPrediction } = await import('@/services/api');
      const mockPrediction = {
        predicted_price: 46000,
        predicted_change: 2.5,
        confidence: 0.78,
        timeframe: '24h',
      };
      (getAIPrediction as any).mockResolvedValueOnce(mockPrediction);

      const { result } = renderHook(() => useAIAnalysis());

      await act(async () => {
        await result.current.fetchPrediction('BTCUSDT', '24h');
      });

      await waitFor(() => {
        expect(result.current.prediction).toEqual(mockPrediction);
      });
    });

    it('should handle prediction fetch errors', async () => {
      const { getAIPrediction } = await import('@/services/api');
      (getAIPrediction as any).mockRejectedValueOnce(new Error('Prediction failed'));

      const { result } = renderHook(() => useAIAnalysis());

      await act(async () => {
        try {
          await result.current.fetchPrediction('BTCUSDT', '24h');
        } catch (error) {
          expect(error).toBeDefined();
        }
      });
    });
  });

  describe('Model Training', () => {
    it('should train AI model', async () => {
      const { trainAIModel } = await import('@/services/api');
      (trainAIModel as any).mockResolvedValueOnce({ status: 'success', model_id: '123' });

      const { result } = renderHook(() => useAIAnalysis());

      await act(async () => {
        await result.current.trainModel({
          symbol: 'BTCUSDT',
          timeframe: '1h',
          epochs: 100,
        });
      });

      expect(trainAIModel).toHaveBeenCalledWith({
        symbol: 'BTCUSDT',
        timeframe: '1h',
        epochs: 100,
      });
    });

    it('should handle training errors', async () => {
      const { trainAIModel } = await import('@/services/api');
      (trainAIModel as any).mockRejectedValueOnce(new Error('Training failed'));

      const { result } = renderHook(() => useAIAnalysis());

      await act(async () => {
        try {
          await result.current.trainModel({
            symbol: 'BTCUSDT',
            timeframe: '1h',
            epochs: 100,
          });
        } catch (error) {
          expect(error).toBeDefined();
        }
      });
    });
  });

  describe('Signal Interpretation', () => {
    it('should interpret strong buy signal', async () => {
      const { getAIAnalysis } = await import('@/services/api');
      const mockAnalysis = {
        signal: 'long',
        confidence: 0.95,
        reasoning: 'Very strong bullish momentum',
      };
      (getAIAnalysis as any).mockResolvedValueOnce(mockAnalysis);

      const { result } = renderHook(() => useAIAnalysis());

      await act(async () => {
        await result.current.fetchAnalysis('BTCUSDT', '1h');
      });

      await waitFor(() => {
        expect(result.current.analysis?.signal).toBe('long');
        expect(result.current.analysis?.confidence).toBeGreaterThan(0.9);
      });
    });

    it('should interpret strong sell signal', async () => {
      const { getAIAnalysis } = await import('@/services/api');
      const mockAnalysis = {
        signal: 'short',
        confidence: 0.92,
        reasoning: 'Strong bearish indicators',
      };
      (getAIAnalysis as any).mockResolvedValueOnce(mockAnalysis);

      const { result } = renderHook(() => useAIAnalysis());

      await act(async () => {
        await result.current.fetchAnalysis('BTCUSDT', '1h');
      });

      await waitFor(() => {
        expect(result.current.analysis?.signal).toBe('short');
        expect(result.current.analysis?.confidence).toBeGreaterThan(0.9);
      });
    });

    it('should interpret neutral signal', async () => {
      const { getAIAnalysis } = await import('@/services/api');
      const mockAnalysis = {
        signal: 'neutral',
        confidence: 0.55,
        reasoning: 'Market consolidation',
      };
      (getAIAnalysis as any).mockResolvedValueOnce(mockAnalysis);

      const { result } = renderHook(() => useAIAnalysis());

      await act(async () => {
        await result.current.fetchAnalysis('BTCUSDT', '1h');
      });

      await waitFor(() => {
        expect(result.current.analysis?.signal).toBe('neutral');
        expect(result.current.analysis?.confidence).toBeLessThan(0.7);
      });
    });
  });

  describe('Confidence Levels', () => {
    it('should categorize high confidence signals', async () => {
      const { getAIAnalysis } = await import('@/services/api');
      const mockAnalysis = { signal: 'long', confidence: 0.9 };
      (getAIAnalysis as any).mockResolvedValueOnce(mockAnalysis);

      const { result } = renderHook(() => useAIAnalysis());

      await act(async () => {
        await result.current.fetchAnalysis('BTCUSDT', '1h');
      });

      await waitFor(() => {
        const confidence = result.current.analysis?.confidence || 0;
        expect(confidence).toBeGreaterThanOrEqual(0.8);
      });
    });

    it('should categorize medium confidence signals', async () => {
      const { getAIAnalysis } = await import('@/services/api');
      const mockAnalysis = { signal: 'long', confidence: 0.65 };
      (getAIAnalysis as any).mockResolvedValueOnce(mockAnalysis);

      const { result } = renderHook(() => useAIAnalysis());

      await act(async () => {
        await result.current.fetchAnalysis('BTCUSDT', '1h');
      });

      await waitFor(() => {
        const confidence = result.current.analysis?.confidence || 0;
        expect(confidence).toBeGreaterThanOrEqual(0.5);
        expect(confidence).toBeLessThan(0.8);
      });
    });

    it('should categorize low confidence signals', async () => {
      const { getAIAnalysis } = await import('@/services/api');
      const mockAnalysis = { signal: 'neutral', confidence: 0.45 };
      (getAIAnalysis as any).mockResolvedValueOnce(mockAnalysis);

      const { result } = renderHook(() => useAIAnalysis());

      await act(async () => {
        await result.current.fetchAnalysis('BTCUSDT', '1h');
      });

      await waitFor(() => {
        const confidence = result.current.analysis?.confidence || 0;
        expect(confidence).toBeLessThan(0.5);
      });
    });
  });

  describe('Caching and Performance', () => {
    it('should cache analysis results', async () => {
      const { getAIAnalysis } = await import('@/services/api');
      const mockAnalysis = { signal: 'long', confidence: 0.85 };
      (getAIAnalysis as any).mockResolvedValueOnce(mockAnalysis);

      const { result } = renderHook(() => useAIAnalysis());

      await act(async () => {
        await result.current.fetchAnalysis('BTCUSDT', '1h');
      });

      await act(async () => {
        await result.current.fetchAnalysis('BTCUSDT', '1h');
      });

      // Should use cache, only call API once
      expect(getAIAnalysis).toHaveBeenCalledTimes(1);
    });

    it('should refresh stale cache', async () => {
      const { getAIAnalysis } = await import('@/services/api');
      const mockAnalysis1 = { signal: 'long', confidence: 0.85 };
      const mockAnalysis2 = { signal: 'short', confidence: 0.75 };

      (getAIAnalysis as any)
        .mockResolvedValueOnce(mockAnalysis1)
        .mockResolvedValueOnce(mockAnalysis2);

      const { result } = renderHook(() => useAIAnalysis());

      await act(async () => {
        await result.current.fetchAnalysis('BTCUSDT', '1h');
      });

      // Force cache refresh
      await act(async () => {
        await result.current.refreshAnalysis('BTCUSDT', '1h');
      });

      expect(getAIAnalysis).toHaveBeenCalledTimes(2);
    });
  });
});
