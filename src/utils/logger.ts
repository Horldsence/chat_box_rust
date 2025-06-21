import { info, warn, error, debug, trace, attachConsole } from '@tauri-apps/plugin-log';

export type LogLevel = 'trace' | 'debug' | 'info' | 'warn' | 'error';

export interface LogEntry {
  level: LogLevel;
  message: string;
  context?: string;
  details?: any;
  timestamp: Date;
}

class Logger {
  private attached = false;

  async init() {
    if (!this.attached) {
      try {
        await attachConsole();
        this.attached = true;
        await this.info('Tauri logger initialized', 'Logger');
      } catch (err) {
        console.warn('Failed to attach console logger:', err);
      }
    }
  }

  async trace(message: string, context?: string, details?: any) {
    try {
      const logMessage = this.formatMessage(message, context, details);
      await trace(logMessage);
      this.logToConsole('trace', message, context, details);
    } catch (err) {
      console.trace(this.formatMessage(message, context, details));
    }
  }

  async debug(message: string, context?: string, details?: any) {
    try {
      const logMessage = this.formatMessage(message, context, details);
      await debug(logMessage);
      this.logToConsole('debug', message, context, details);
    } catch (err) {
      console.debug(this.formatMessage(message, context, details));
    }
  }

  async info(message: string, context?: string, details?: any) {
    try {
      const logMessage = this.formatMessage(message, context, details);
      await info(logMessage);
      this.logToConsole('info', message, context, details);
    } catch (err) {
      console.info(this.formatMessage(message, context, details));
    }
  }

  async warn(message: string, context?: string, details?: any) {
    try {
      const logMessage = this.formatMessage(message, context, details);
      await warn(logMessage);
      this.logToConsole('warn', message, context, details);
    } catch (err) {
      console.warn(this.formatMessage(message, context, details));
    }
  }

  async error(message: string, context?: string, details?: any) {
    try {
      const logMessage = this.formatMessage(message, context, details);
      await error(logMessage);
      this.logToConsole('error', message, context, details);
    } catch (err) {
      console.error(this.formatMessage(message, context, details));
    }
  }

  async logApiError(error: any, context: string = "API call") {
    const errorMessage = typeof error === "string" ? error : error?.message || "Unknown error";
    const errorCode = error?.code || "UNKNOWN_ERROR";

    const details = {
      code: errorCode,
      message: errorMessage,
      context,
      stack: error?.stack,
      originalError: error
    };

    await this.error(`${context}: ${errorMessage}`, errorCode, details);
  }

  private formatMessage(message: string, context?: string, details?: any): string {
    let formatted = message;

    if (context) {
      formatted = `[${context}] ${formatted}`;
    }

    if (details) {
      const detailsStr = typeof details === 'string'
        ? details
        : JSON.stringify(details, null, 2);
      formatted += ` | Details: ${detailsStr}`;
    }

    return formatted;
  }

  private logToConsole(level: LogLevel, message: string, context?: string, details?: any) {
    const timestamp = new Date().toISOString();
    const formattedMessage = `[${timestamp}] ${this.formatMessage(message, context, details)}`;

    switch (level) {
      case 'trace':
        console.trace(formattedMessage);
        break;
      case 'debug':
        console.debug(formattedMessage);
        break;
      case 'info':
        console.info(formattedMessage);
        break;
      case 'warn':
        console.warn(formattedMessage);
        break;
      case 'error':
        console.error(formattedMessage);
        break;
    }
  }

  // Convenience methods for common use cases
  async logStateChange(stateName: string, oldValue: any, newValue: any, context?: string) {
    await this.debug(
      `State change: ${stateName}`,
      context || 'StateManager',
      { oldValue, newValue }
    );
  }

  async logUserAction(action: string, details?: any, context?: string) {
    await this.info(
      `User action: ${action}`,
      context || 'UserInterface',
      details
    );
  }

  async logApiCall(method: string, endpoint: string, params?: any, context?: string) {
    await this.debug(
      `API call: ${method} ${endpoint}`,
      context || 'ApiClient',
      params
    );
  }

  async logPerformance(operation: string, duration: number, context?: string) {
    await this.info(
      `Performance: ${operation} took ${duration}ms`,
      context || 'Performance',
      { operation, duration }
    );
  }
}

// Create a singleton instance
export const logger = new Logger();

// Initialize logger on module load
logger.init().catch(err => {
  console.warn('Failed to initialize logger:', err);
});

// Export convenience functions
export const log = {
  trace: (message: string, context?: string, details?: any) => logger.trace(message, context, details),
  debug: (message: string, context?: string, details?: any) => logger.debug(message, context, details),
  info: (message: string, context?: string, details?: any) => logger.info(message, context, details),
  warn: (message: string, context?: string, details?: any) => logger.warn(message, context, details),
  error: (message: string, context?: string, details?: any) => logger.error(message, context, details),

  // Specialized logging methods
  apiError: (error: any, context?: string) => logger.logApiError(error, context),
  stateChange: (stateName: string, oldValue: any, newValue: any, context?: string) =>
    logger.logStateChange(stateName, oldValue, newValue, context),
  userAction: (action: string, details?: any, context?: string) =>
    logger.logUserAction(action, details, context),
  apiCall: (method: string, endpoint: string, params?: any, context?: string) =>
    logger.logApiCall(method, endpoint, params, context),
  performance: (operation: string, duration: number, context?: string) =>
    logger.logPerformance(operation, duration, context),
};

export default logger;
