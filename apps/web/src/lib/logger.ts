/**
 * Centralized logging utility
 *
 * Provides a consistent logging interface across the application with:
 * - Environment-aware logging (console in dev, silent/external in prod)
 * - Structured logging with context objects
 * - Type-safe log levels
 * - Easy integration with external logging services (Sentry, LogRocket, etc.)
 *
 * @example
 * ```typescript
 * import { logger } from './lib/logger';
 *
 * // Simple log
 * logger.info('User logged in');
 *
 * // Log with context
 * logger.error('API request failed', {
 *   endpoint: '/api/issues',
 *   statusCode: 500,
 *   error: err
 * });
 *
 * // Debug logging (only in development)
 * logger.debug('Graph nodes updated', { nodeCount: 42 });
 * ```
 */

type LogLevel = 'debug' | 'info' | 'warn' | 'error';

interface LogContext {
  [key: string]: unknown;
}

class Logger {
  private isDevelopment: boolean;

  constructor() {
    this.isDevelopment = import.meta.env.DEV;
  }

  /**
   * Debug-level logging (only in development)
   * Use for detailed debugging information
   */
  debug(message: string, context?: LogContext): void {
    if (this.isDevelopment) {
      this.log('debug', message, context);
    }
  }

  /**
   * Info-level logging
   * Use for general informational messages
   */
  info(message: string, context?: LogContext): void {
    this.log('info', message, context);
  }

  /**
   * Warning-level logging
   * Use for recoverable errors or unexpected situations
   */
  warn(message: string, context?: LogContext): void {
    this.log('warn', message, context);
  }

  /**
   * Error-level logging
   * Use for errors and exceptions
   */
  error(message: string, context?: LogContext): void {
    this.log('error', message, context);
  }

  /**
   * Internal logging method
   * Handles actual console output and can be extended for external services
   */
  private log(level: LogLevel, message: string, context?: LogContext): void {
    const timestamp = new Date().toISOString();
    const logData = {
      timestamp,
      level,
      message,
      ...context,
    };

    // In development, use console with colors
    if (this.isDevelopment) {
      const styles = this.getConsoleStyles(level);
      console.log(
        `%c[${level.toUpperCase()}]%c ${message}`,
        styles.label,
        styles.message,
        context || ''
      );
      return;
    }

    // In production, use structured logging
    // This can be extended to send to external services:
    // - Sentry.captureMessage(message, { level, extra: context })
    // - LogRocket.log(message, context)
    // - Custom analytics endpoint
    switch (level) {
      case 'error':
        console.error(logData);
        // TODO: Send to error tracking service (Sentry, etc.)
        break;
      case 'warn':
        console.warn(logData);
        break;
      case 'info':
        console.info(logData);
        break;
      case 'debug':
        // Silent in production
        break;
    }
  }

  /**
   * Get console styles for different log levels
   */
  private getConsoleStyles(level: LogLevel): { label: string; message: string } {
    const styles = {
      debug: {
        label: 'color: #6B7280; font-weight: bold;',
        message: 'color: #6B7280;',
      },
      info: {
        label: 'color: #3B82F6; font-weight: bold;',
        message: 'color: #1F2937;',
      },
      warn: {
        label: 'color: #F59E0B; font-weight: bold;',
        message: 'color: #92400E;',
      },
      error: {
        label: 'color: #EF4444; font-weight: bold;',
        message: 'color: #7F1D1D;',
      },
    };

    return styles[level];
  }
}

// Export singleton instance
export const logger = new Logger();

// Export type for external use
export type { LogLevel, LogContext };
