/**
 * Logger Utility
 *
 * Provides conditional logging based on NODE_ENV.
 * In production, all logs are suppressed to reduce bundle size and improve performance.
 * In development, logs are output to the console with appropriate formatting.
 */

type LogLevel = 'debug' | 'info' | 'warn' | 'error';

interface LoggerConfig {
  isDevelopment: boolean;
  minLevel: LogLevel;
}

class Logger {
  private config: LoggerConfig;
  private levels: Record<LogLevel, number> = {
    debug: 0,
    info: 1,
    warn: 2,
    error: 3,
  };

  constructor() {
    this.config = {
      isDevelopment: import.meta.env.MODE === 'development',
      minLevel: 'debug',
    };
  }

  private shouldLog(level: LogLevel): boolean {
    if (!this.config.isDevelopment) {
      return false;
    }
    return this.levels[level] >= this.levels[this.config.minLevel];
  }

  private formatMessage(level: LogLevel, message: string, ...args: unknown[]): void {
    if (!this.shouldLog(level)) {
      return;
    }

    const timestamp = new Date().toISOString();
    const prefix = `[${timestamp}] [${level.toUpperCase()}]`;

    switch (level) {
      case 'debug':
        console.debug(prefix, message, ...args);
        break;
      case 'info':
        console.info(prefix, message, ...args);
        break;
      case 'warn':
        console.warn(prefix, message, ...args);
        break;
      case 'error':
        console.error(prefix, message, ...args);
        break;
    }
  }

  /**
   * Log debug information (lowest priority)
   * Only visible in development mode
   */
  debug(message: string, ...args: unknown[]): void {
    this.formatMessage('debug', message, ...args);
  }

  /**
   * Log informational messages
   * Only visible in development mode
   */
  info(message: string, ...args: unknown[]): void {
    this.formatMessage('info', message, ...args);
  }

  /**
   * Log warning messages
   * Only visible in development mode
   */
  warn(message: string, ...args: unknown[]): void {
    this.formatMessage('warn', message, ...args);
  }

  /**
   * Log error messages
   * Only visible in development mode
   */
  error(message: string, ...args: unknown[]): void {
    this.formatMessage('error', message, ...args);
  }

  /**
   * Log errors with stack traces
   * Only visible in development mode
   */
  exception(error: Error | unknown, context?: string): void {
    if (!this.config.isDevelopment) {
      return;
    }

    const prefix = context ? `[${context}]` : '';
    if (error instanceof Error) {
      console.error(prefix, 'Exception:', error.message);
      console.error('Stack:', error.stack);
    } else {
      console.error(prefix, 'Exception:', error);
    }
  }

  /**
   * Log network/API requests
   * Only visible in development mode
   */
  api(method: string, url: string, data?: unknown): void {
    if (!this.config.isDevelopment) {
      return;
    }
    console.log(`[API] ${method.toUpperCase()} ${url}`, data || '');
  }

  /**
   * Log WebSocket events
   * Only visible in development mode
   */
  ws(event: string, data?: unknown): void {
    if (!this.config.isDevelopment) {
      return;
    }
    console.log(`[WebSocket] ${event}`, data || '');
  }
}

// Export singleton instance
export const logger = new Logger();

// Export default for convenience
export default logger;
