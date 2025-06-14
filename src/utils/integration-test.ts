/**
 * Vue + Tauri + Rust 集成测试脚本
 * 用于测试前后端通信和错误处理功能
 */

import { chatAPI } from '@/api'
import { errorService } from '@/services/ErrorService'

interface TestResult {
  name: string
  passed: boolean
  error?: string
  duration: number
}

export class IntegrationTester {
  private results: TestResult[] = []

  /**
   * 运行所有集成测试
   */
  public async runAllTests(): Promise<TestResult[]> {
    console.log('🚀 开始运行 Vue + Tauri + Rust 集成测试...')

    this.results = []

    // 基础连接测试
    await this.runTest('连接测试', this.testConnection.bind(this))

    // 系统信息测试
    await this.runTest('系统信息测试', this.testSystemInfo.bind(this))

    // 健康检查测试
    await this.runTest('健康检查测试', this.testHealthCheck.bind(this))

    // 配置管理测试
    await this.runTest('配置管理测试', this.testConfigManagement.bind(this))

    // 对话管理测试
    await this.runTest('对话管理测试', this.testConversationManagement.bind(this))

    // 错误处理测试
    await this.runTest('错误处理测试', this.testErrorHandling.bind(this))

    // 日志记录测试
    await this.runTest('日志记录测试', this.testLogging.bind(this))

    this.printResults()
    return this.results
  }

  /**
   * 运行单个测试
   */
  private async runTest(name: string, testFn: () => Promise<void>): Promise<void> {
    const startTime = Date.now()

    try {
      await testFn()
      const duration = Date.now() - startTime

      this.results.push({
        name,
        passed: true,
        duration
      })

      console.log(`✅ ${name} - 通过 (${duration}ms)`)
    } catch (error) {
      const duration = Date.now() - startTime
      const errorMessage = error instanceof Error ? error.message : String(error)

      this.results.push({
        name,
        passed: false,
        error: errorMessage,
        duration
      })

      console.error(`❌ ${name} - 失败 (${duration}ms): ${errorMessage}`)
    }
  }

  /**
   * 测试基础连接
   */
  private async testConnection(): Promise<void> {
    const isConnected = await chatAPI.checkConnection()

    if (!isConnected) {
      throw new Error('无法连接到后端服务')
    }
  }

  /**
   * 测试系统信息获取
   */
  private async testSystemInfo(): Promise<void> {
    const systemInfo = await chatAPI.getSystemInfo()

    // 验证必要字段
    const requiredFields = ['os', 'arch', 'app_version']
    for (const field of requiredFields) {
      if (!systemInfo[field]) {
        throw new Error(`系统信息缺少字段: ${field}`)
      }
    }

    console.log('📊 系统信息:', systemInfo)
  }

  /**
   * 测试健康检查
   */
  private async testHealthCheck(): Promise<void> {
    const healthStatus = await chatAPI.getHealthStatus()

    // 验证健康状态字段
    const requiredFields = ['config_loaded', 'database_connected', 'llm_available']
    for (const field of requiredFields) {
      if (healthStatus[field] === undefined) {
        throw new Error(`健康状态缺少字段: ${field}`)
      }
    }

    console.log('🏥 健康状态:', healthStatus)
  }

  /**
   * 测试配置管理
   */
  private async testConfigManagement(): Promise<void> {
    // 获取当前配置
    const originalConfig = await chatAPI.getAppConfig()

    if (!originalConfig) {
      throw new Error('无法获取应用配置')
    }

    // 验证配置结构
    const requiredSections = ['ai_model', 'voice', 'ui', 'database', 'app_behavior']
    for (const section of requiredSections) {
      if (!originalConfig[section as keyof typeof originalConfig]) {
        throw new Error(`配置缺少部分: ${section}`)
      }
    }

    // 测试配置保存
    const testConfig = { ...originalConfig }
    testConfig.app_behavior.welcome_message = '测试消息 - ' + Date.now()

    await chatAPI.saveAppConfig(testConfig)

    // 验证配置是否保存成功
    const updatedConfig = await chatAPI.getAppConfig()
    if (updatedConfig.app_behavior.welcome_message !== testConfig.app_behavior.welcome_message) {
      throw new Error('配置保存失败')
    }

    // 恢复原始配置
    await chatAPI.saveAppConfig(originalConfig)

    console.log('⚙️ 配置管理测试通过')
  }

  /**
   * 测试对话管理
   */
  private async testConversationManagement(): Promise<void> {
    // 获取对话列表
    const conversations = await chatAPI.getConversations()
    console.log(`💬 当前对话数量: ${conversations.length}`)

    // 创建测试对话
    const testTitle = '测试对话 - ' + Date.now()
    const newConversation = await chatAPI.createConversation(testTitle)

    if (!newConversation || newConversation.title !== testTitle) {
      throw new Error('创建对话失败')
    }

    // 获取对话消息
    const messages = await chatAPI.getConversationMessages(newConversation.id)
    console.log(`📝 对话消息数量: ${messages.length}`)

    // 删除测试对话
    await chatAPI.deleteConversation(newConversation.id)

    console.log('💬 对话管理测试通过')
  }

  /**
   * 测试错误处理
   */
  private async testErrorHandling(): Promise<void> {
    // 测试前端错误处理服务
    const testError = new Error('测试错误')
    testError.name = 'TestError'

    try {
      await errorService.handleError(testError, '测试上下文', {
        showNotification: false,
        logToBackend: false
      })
    } catch (error) {
      throw new Error('前端错误处理失败')
    }

    // 测试错误统计
    const errorStats = errorService.getErrorStats()
    if (errorStats.total === 0) {
      throw new Error('错误统计功能异常')
    }

    console.log('🚨 错误处理测试通过')
  }

  /**
   * 测试日志记录
   */
  private async testLogging(): Promise<void> {
    const timestamp = Date.now()

    // 测试错误日志
    await chatAPI.log_error(
      'TEST_ERROR',
      '测试错误消息',
      `测试时间: ${timestamp}`
    )

    // 测试警告日志
    await chatAPI.log_warning(
      '测试警告消息',
      `测试时间: ${timestamp}`
    )

    // 测试信息日志
    await chatAPI.log_info(
      '测试信息消息',
      `测试时间: ${timestamp}`
    )

    console.log('📝 日志记录测试通过')
  }

  /**
   * 打印测试结果
   */
  private printResults(): void {
    console.log('\n📋 测试结果汇总:')
    console.log('='.repeat(50))

    const passed = this.results.filter(r => r.passed).length
    const failed = this.results.filter(r => !r.passed).length
    const totalDuration = this.results.reduce((sum, r) => sum + r.duration, 0)

    console.log(`✅ 通过: ${passed}`)
    console.log(`❌ 失败: ${failed}`)
    console.log(`⏱️  总耗时: ${totalDuration}ms`)
    console.log(`📊 成功率: ${((passed / this.results.length) * 100).toFixed(1)}%`)

    // 显示失败的测试详情
    const failedTests = this.results.filter(r => !r.passed)
    if (failedTests.length > 0) {
      console.log('\n❌ 失败的测试:')
      failedTests.forEach(test => {
        console.log(`  - ${test.name}: ${test.error}`)
      })
    }

    console.log('='.repeat(50))

    if (failed === 0) {
      console.log('🎉 所有测试通过！Vue + Tauri + Rust 集成正常工作。')
    } else {
      console.log('⚠️  部分测试失败，请检查相关功能。')
    }
  }

  /**
   * 获取测试结果摘要
   */
  public getResultSummary(): {
    total: number
    passed: number
    failed: number
    successRate: number
    totalDuration: number
  } {
    const passed = this.results.filter(r => r.passed).length
    const failed = this.results.filter(r => !r.passed).length
    const totalDuration = this.results.reduce((sum, r) => sum + r.duration, 0)

    return {
      total: this.results.length,
      passed,
      failed,
      successRate: (passed / this.results.length) * 100,
      totalDuration
    }
  }
}

/**
 * 便利函数：运行完整的集成测试
 */
export async function runIntegrationTests(): Promise<TestResult[]> {
  const tester = new IntegrationTester()
  return await tester.runAllTests()
}

/**
 * 便利函数：运行快速连接测试
 */
export async function quickConnectionTest(): Promise<boolean> {
  try {
    const isConnected = await chatAPI.checkConnection()
    if (isConnected) {
      console.log('✅ 快速连接测试通过')
      errorService.showSuccess('后端连接正常')
    } else {
      console.log('❌ 快速连接测试失败')
      errorService.showWarning('后端连接异常')
    }
    return isConnected
  } catch (error) {
    console.error('❌ 快速连接测试错误:', error)
    errorService.showWarning('连接测试失败')
    return false
  }
}

// 导出测试器实例
export const integrationTester = new IntegrationTester()
