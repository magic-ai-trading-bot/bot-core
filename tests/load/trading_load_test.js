import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const apiDuration = new Trend('api_duration');
const successfulRequests = new Counter('successful_requests');
const failedRequests = new Counter('failed_requests');

// Load test configuration
export const options = {
  stages: [
    { duration: '30s', target: 10 },   // Ramp up to 10 users
    { duration: '1m', target: 50 },    // Ramp up to 50 users
    { duration: '2m', target: 100 },   // Stay at 100 users
    { duration: '1m', target: 50 },    // Ramp down to 50
    { duration: '30s', target: 0 },    // Ramp down to 0
  ],
  thresholds: {
    'http_req_duration': ['p(95)<2000'], // 95% of requests under 2s
    'http_req_failed': ['rate<0.05'],     // Less than 5% failure rate
    'errors': ['rate<0.05'],              // Less than 5% errors
  },
};

const BASE_URL_RUST = __ENV.RUST_URL || 'http://localhost:8080';
const BASE_URL_PYTHON = __ENV.PYTHON_URL || 'http://localhost:8000';

export default function () {
  group('Health Checks', () => {
    // Rust health check
    const rustHealth = http.get(`${BASE_URL_RUST}/health`);
    check(rustHealth, {
      'Rust health status 200': (r) => r.status === 200,
      'Rust health response time < 500ms': (r) => r.timings.duration < 500,
    }) || errorRate.add(1);

    apiDuration.add(rustHealth.timings.duration);

    // Python health check
    const pythonHealth = http.get(`${BASE_URL_PYTHON}/health`);
    check(pythonHealth, {
      'Python health status 200': (r) => r.status === 200,
      'Python health response time < 500ms': (r) => r.timings.duration < 500,
    }) || errorRate.add(1);

    apiDuration.add(pythonHealth.timings.duration);
  });

  sleep(1);

  group('Market Data Endpoints', () => {
    const marketData = http.get(`${BASE_URL_RUST}/api/market/data?symbol=BTCUSDT`);

    const marketDataOk = check(marketData, {
      'Market data status 200 or 401': (r) => r.status === 200 || r.status === 401,
      'Market data response time < 1000ms': (r) => r.timings.duration < 1000,
    });

    if (marketDataOk) {
      successfulRequests.add(1);
    } else {
      failedRequests.add(1);
      errorRate.add(1);
    }

    apiDuration.add(marketData.timings.duration);
  });

  sleep(1);

  group('AI Analysis Endpoint', () => {
    const analysisPayload = JSON.stringify({
      symbol: 'BTCUSDT',
      timeframe: '1h',
      candles: [
        {
          open: 50000.0,
          high: 50500.0,
          low: 49800.0,
          close: 50200.0,
          volume: 1000.0,
          timestamp: Date.now(),
        },
      ],
    });

    const params = {
      headers: {
        'Content-Type': 'application/json',
      },
      timeout: '30s', // AI calls can take longer
    };

    const aiAnalysis = http.post(
      `${BASE_URL_PYTHON}/ai/analyze`,
      analysisPayload,
      params
    );

    const aiOk = check(aiAnalysis, {
      'AI analysis status ok': (r) => r.status === 200 || r.status === 429 || r.status === 503,
      'AI analysis response time < 30s': (r) => r.timings.duration < 30000,
    });

    if (aiOk && aiAnalysis.status === 200) {
      successfulRequests.add(1);

      // Validate response structure
      try {
        const body = JSON.parse(aiAnalysis.body);
        check(body, {
          'AI response has signal': (b) => b.signal !== undefined,
          'AI response has confidence': (b) => b.confidence !== undefined,
        });
      } catch (e) {
        errorRate.add(1);
      }
    } else {
      failedRequests.add(1);
      if (aiAnalysis.status !== 429) { // Don't count rate limits as errors
        errorRate.add(1);
      }
    }

    apiDuration.add(aiAnalysis.timings.duration);
  });

  sleep(2);

  group('Trading Endpoints', () => {
    // Test positions endpoint
    const positions = http.get(`${BASE_URL_RUST}/api/positions`);

    check(positions, {
      'Positions endpoint accessible': (r) => r.status === 200 || r.status === 401,
      'Positions response time < 1000ms': (r) => r.timings.duration < 1000,
    }) || errorRate.add(1);

    apiDuration.add(positions.timings.duration);
  });

  sleep(1);
}

export function handleSummary(data) {
  return {
    'stdout': textSummary(data, { indent: ' ', enableColors: true }),
    'load-test-results.json': JSON.stringify(data),
  };
}

function textSummary(data, options) {
  const indent = options.indent || '';
  const enableColors = options.enableColors || false;

  let summary = '\n';
  summary += `${indent}Summary:\n`;
  summary += `${indent}  Total Requests: ${data.metrics.http_reqs.values.count}\n`;
  summary += `${indent}  Failed Requests: ${data.metrics.http_req_failed.values.passes || 0}\n`;
  summary += `${indent}  Request Duration (avg): ${data.metrics.http_req_duration.values.avg.toFixed(2)}ms\n`;
  summary += `${indent}  Request Duration (p95): ${data.metrics.http_req_duration.values['p(95)'].toFixed(2)}ms\n`;
  summary += `${indent}  Request Duration (max): ${data.metrics.http_req_duration.values.max.toFixed(2)}ms\n`;

  return summary;
}
