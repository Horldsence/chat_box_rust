/**
 * 错误处理服务
 * 统一管理应用中的错误处理逻辑，包括后端错误、网络错误、业务逻辑错误等
 */

import { ElMessage, ElNotification, ElMessageBox } from 'element-plus'
import { chatAPI } from '@/api'

export interface ErrorInfo {
  id: string
  code: string
  title: string
  message: string
  details?: string
  timestamp: number
  type: 'error' | 'warning' | 'info' | 'success'
  category: 'network' | 'business' | 'system' | 'validation' | 'unknown'
  severity: 'low' | 'medium' | 'high' | 'critical'
  source?: string
  retryable?: boolean
  userAction?: string
}

export interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: {
    code: string
    message: string
    details?: string
    timestamp: number
  }
}

export interface ErrorHandlerOptions {
  showNotification?: boolean
  showDialog?: boolean
  logToConsole?: boolean
  logToBackend?: boolean
  autoRetry?: boolean
  retryCount?: number
  retryDelay?: number
}

class ErrorService {
  private static instance: ErrorService
  private errorHistory: ErrorInfo[] = []
  private maxHistorySize = 100
  private retryCounters = new Map<string, number>()
  private errorCallbacks = new Map<string, (error: ErrorInfo) => void>()

  private constructor() {
    this.setupGlobalErrorHandlers()
  }

  public static getInstance(): ErrorService {
    if (!ErrorService.instance) {
      ErrorService.instance = new ErrorService()
    }
    return ErrorService.instance
  }

  /**
   * 处理错误
   */
  public async handleError(
    error: any,
    context?: string,
    options: ErrorHandlerOptions = {}
  ): Promise<void> {
    const errorInfo = this.parseError(error, context)

    // 默认选项
    const defaultOptions: ErrorHandlerOptions = {
      showNotification: true,
      showDialog: errorInfo.severity === 'critical',
      logToConsole: true,
      logToBackend: true,
      autoRetry: errorInfo.retryable && errorInfo.severity !== 'critical',
      retryCount: 3,
      retryDelay: 1000
    }

    const finalOptions = { ...defaultOptions, ...options }

    // 添加到历史记录
    this.addToHistory(errorInfo)

    // 控制台日志
    if (finalOptions.logToConsole) {
      this.logToConsole(errorInfo)
    }

    // 后端日志
    if (finalOptions.logToBackend) {
      await this.logToBackend(errorInfo)
    }

    // 显示通知
    if (finalOptions.showNotification) {
      this.showNotification(errorInfo)
    }

    // 显示对话框
    if (finalOptions.showDialog) {
      await this.showErrorDialog(errorInfo)
    }

    // 自动重试
    if (finalOptions.autoRetry && this.shouldRetry(errorInfo, finalOptions.retryCount!)) {
      setTimeout(() => {
        this.handleRetry(errorInfo, finalOptions)
      }, finalOptions.retryDelay)
    }

    // 触发回调
    this.triggerCallbacks(errorInfo)
  }

  /**
   * 处理API响应
   */
  public handleApiResponse<T>(response: ApiResponse<T>, context?: string): T {
    if (!response.success && response.error) {
      const error = new Error(response.error.message)
      Object.assign(error, {
        code: response.error.code,
        details: response.error.details,
        timestamp: response.error.timestamp,
        apiResponse: true
      })

      this.handleError(error, context)
      throw error
    }

    return response.data as T
  }

  /**
   * 解析错误信息
   */
  private parseError(error: any, context?: string): ErrorInfo {
    const timestamp = Date.now()
    const id = this.generateErrorId()

    // API响应错误
    if (error.apiResponse) {
      return {
        id,
        code: error.code || 'API_ERROR',
        title: '接口调用失败',
        message: error.message,
        details: error.details,
        timestamp: error.timestamp || timestamp,
        type: 'error',
        category: 'business',
        severity: this.getSeverityByCode(error.code),
        source: context,
        retryable: this.isRetryableError(error.code)
      }
    }

    // 网络错误
    if (error.name === 'TypeError' && error.message.includes('fetch')) {
      return {
        id,
        code: 'NETWORK_ERROR',
        title: '网络连接失败',
        message: '无法连接到服务器，请检查网络连接',
        details: error.message,
        timestamp,
        type: 'error',
        category: 'network',
        severity: 'high',
        source: context,
        retryable: true,
        userAction: '请检查网络连接后重试'
      }
    }

    // Tauri命令错误
    if (error.message && error.message.includes('tauri://')) {
      return {
        id,
        code: 'TAURI_COMMAND_ERROR',
        title: 'Tauri命令执行失败',
        message: error.message,
        timestamp,
        type: 'error',
        category: 'system',
        severity: 'medium',
        source: context,
        retryable: false
      }
    }

    // 验证错误
    if (error.name === 'ValidationError') {
      return {
        id,
        code: 'VALIDATION_ERROR',
        title: '数据验证失败',
        message: error.message,
        timestamp,
        type: 'warning',
        category: 'validation',
        severity: 'low',
        source: context,
        retryable: false,
        userAction: '请检查输入数据是否正确'
      }
    }

    // 默认错误处理
    return {
      id,
      code: error.code || 'UNKNOWN_ERROR',
      title: error.name || '未知错误',
      message: error.message || '发生了未知错误',
      details: error.stack,
      timestamp,
      type: 'error',
      category: 'unknown',
      severity: 'medium',
      source: context,
      retryable: false
    }
  }

  /**
   * 根据错误代码获取严重程度
   */
  private getSeverityByCode(code: string): 'low' | 'medium' | 'high' | 'critical' {
    const criticalCodes = ['SYSTEM_CRASH', 'DATABASE_CORRUPTION', 'SECURITY_BREACH']
    const highCodes = ['DATABASE_CONNECTION_FAILED', 'LLM_SERVICE_UNAVAILABLE']
    const mediumCodes = ['API_TIMEOUT', 'INVALID_RESPONSE']
    const lowCodes = ['VALIDATION_ERROR', 'USER_INPUT_ERROR']

    if (criticalCodes.some(c => code.includes(c))) return 'critical'
    if (highCodes.some(c => code.includes(c))) return 'high'
    if (mediumCodes.some(c => code.includes(c))) return 'medium'
    if (lowCodes.some(c => code.includes(c))) return 'low'

    return 'medium'
  }

  /**
   * 判断错误是否可重试
   */
  private isRetryableError(code: string): boolean {
    const retryableCodes = [
      'NETWORK_ERROR',
      'TIMEOUT_ERROR',
      'SERVER_BUSY',
      'RATE_LIMITED',
      'CONNECTION_REFUSED'
    ]
    return retryableCodes.some(c => code.includes(c))
  }

  /**
   * 显示通知
   */
  private showNotification(error: ErrorInfo): void {
    const options = {
      title: error.title,
      message: error.message,
      type: error.type,
      duration: this.getNotificationDuration(error.severity),
      showClose: true
    }

    if (error.severity === 'critical' || error.severity === 'high') {
      ElNotification.error(options)
    } else if (error.severity === 'medium') {
      ElNotification.warning(options)
    } else {
      ElNotification.info(options)
    }
  }

  /**
   * 显示错误对话框
   */
  private async showErrorDialog(error: ErrorInfo): Promise<void> {
    const actions = ['确定']
    if (error.retryable) {
      actions.unshift('重试')
    }

    try {
      const action = await ElMessageBox({
        title: error.title,
        message: this.formatErrorForDialog(error),
        type: 'error',
        showCancelButton: error.retryable,
        confirmButtonText: '确定',
        cancelButtonText: error.retryable ? '重试' : '',
        dangerouslyUseHTMLString: true,
        customClass: 'error-dialog'
      })

      if (action === 'cancel' && error.retryable) {
        this.handleRetry(error, { retryCount: 1 })
      }
    } catch (e) {
      // 用户取消对话框
    }
  }

  /**
   * 格式化错误信息用于对话框显示
   */
  private formatErrorForDialog(error: ErrorInfo): string {
    let html = `<p><strong>错误信息:</strong> ${error.message}</p>`

    if (error.details) {
      html += `<details style="margin-top: 10px;">
        <summary>错误详情</summary>
        <pre style="background: #f5f5f5; padding: 10px; margin-top: 5px; font-size: 12px; overflow-x: auto;">${error.details}</pre>
      </details>`
    }

    if (error.userAction) {
      html += `<p style="margin-top: 10px; color: #409eff;"><strong>建议操作:</strong> ${error.userAction}</p>`
    }

    html += `<p style="margin-top: 10px; font-size: 12px; color: #909399;">
      错误代码: ${error.code} | 发生时间: ${new Date(error.timestamp).toLocaleString()}
    </p>`

    return html
  }

  /**
   * 控制台日志
   */
  private logToConsole(error: ErrorInfo): void {
    const prefix = `[${error.category.toUpperCase()}] ${error.code}`

    switch (error.severity) {
      case 'critical':
      case 'high':
        console.error(prefix, error)
        break
      case 'medium':
        console.warn(prefix, error)
        break
      case 'low':
        console.info(prefix, error)
        break
    }
  }

  /**
   * 后端日志
   */
  private async logToBackend(error: ErrorInfo): Promise<void> {
    try {
      switch (error.type) {
        case 'error':
          await chatAPI.log_error(error.code, error.message, error.details)
          break
        case 'warning':
          await chatAPI.log_warning(error.message, error.details)
          break
        default:
          await chatAPI.log_info(error.message, error.details)
      }
    } catch (e) {
      console.warn('Failed to log error to backend:', e)
    }
  }

  /**
   * 获取通知持续时间
   */
  private getNotificationDuration(severity: string): number {
    switch (severity) {
      case 'critical': return 0 // 不自动关闭
      case 'high': return 8000
      case 'medium': return 4000
      case 'low': return 2000
      default: return 4000
    }
  }

  /**
   * 判断是否应该重试
   */
  private shouldRetry(error: ErrorInfo, maxRetries: number): boolean {
    if (!error.retryable) return false

    const currentCount = this.retryCounters.get(error.code) || 0
    return currentCount < maxRetries
  }

  /**
   * 处理重试
   */
  private handleRetry(error: ErrorInfo, options: ErrorHandlerOptions): void {
    const currentCount = this.retryCounters.get(error.code) || 0
    this.retryCounters.set(error.code, currentCount + 1)

    ElMessage.info(`正在重试... (${currentCount + 1}/${options.retryCount})`)
  }

  /**
   * 添加到历史记录
   */
  private addToHistory(error: ErrorInfo): void {
    this.errorHistory.unshift(error)
    if (this.errorHistory.length > this.maxHistorySize) {
      this.errorHistory = this.errorHistory.slice(0, this.maxHistorySize)
    }
  }

  /**
   * 触发错误回调
   */
  private triggerCallbacks(error: ErrorInfo): void {
    this.errorCallbacks.forEach((callback, key) => {
      try {
        callback(error)
      } catch (e) {
        console.warn(`Error in callback ${key}:`, e)
      }
    })
  }

  /**
   * 生成错误ID
   */
  private generateErrorId(): string {
    return `error_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
  }

  /**
   * 设置全局错误处理器
   */
  private setupGlobalErrorHandlers(): void {
    // 捕获未处理的Promise拒绝
    window.addEventListener('unhandledrejection', (event) => {
      this.handleError(event.reason, 'unhandledrejection', {
        showDialog: false,
        logToBackend: true
      })
    })

    // 捕获JavaScript错误
    window.addEventListener('error', (event) => {
      const error = new Error(event.message)
      Object.assign(error, {
        filename: event.filename,
        lineno: event.lineno,
        colno: event.colno
      })

      this.handleError(error, 'javascript', {
        showDialog: false,
        logToBackend: true
      })
    })
  }

  /**
   * 公共方法
   */
  public getErrorHistory(): ErrorInfo[] {
    return [...this.errorHistory]
  }

  public clearErrorHistory(): void {
    this.errorHistory = []
    this.retryCounters.clear()
  }

  public addErrorCallback(key: string, callback: (error: ErrorInfo) => void): void {
    this.errorCallbacks.set(key, callback)
  }

  public removeErrorCallback(key: string): void {
    this.errorCallbacks.delete(key)
  }

  public getErrorStats(): { total: number; byCategory: Record<string, number>; bySeverity: Record<string, number> } {
    const stats = {
      total: this.errorHistory.length,
      byCategory: {} as Record<string, number>,
      bySeverity: {} as Record<string, number>
    }

    this.errorHistory.forEach(error => {
      stats.byCategory[error.category] = (stats.byCategory[error.category] || 0) + 1
      stats.bySeverity[error.severity] = (stats.bySeverity[error.severity] || 0) + 1
    })

    return stats
  }

  /**
   * 便利方法
   */
  public showSuccess(message: string, title = '操作成功'): void {
    ElNotification.success({
      title,
      message,
      duration: 3000
    })
  }

  public showWarning(message: string, title = '警告'): void {
    ElNotification.warning({
      title,
      message,
      duration: 4000
    })
  }

  public showInfo(message: string, title = '提示'): void {
    ElNotification.info({
      title,
      message,
      duration: 3000
    })
  }
}

// 导出单例
export const errorService = ErrorService.getInstance()

// 导出便利函数
export const {
  handleError,
  handleApiResponse,
  showSuccess,
  showWarning,
  showInfo,
  getErrorHistory,
  clearErrorHistory,
  addErrorCallback,
  removeErrorCallback,
  getErrorStats
} = errorService
