<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let testResults: string[] = [];
  let isLoading = false;
  let models: string[] = [];
  let healthStatus: boolean | null = null;

  // 添加测试结果
  function addResult(message: string) {
    testResults = [...testResults, `[${new Date().toLocaleTimeString()}] ${message}`];
  }

  // 清空测试结果
  function clearResults() {
    testResults = [];
  }

  // 测试基本模型功能
  async function testBasicModel() {
    isLoading = true;
    addResult("开始测试 Candle 基本模型功能...");

    try {
      const result = await invoke<string>("test_candle_model");
      addResult(`✅ 基本模型测试成功: ${result}`);
    } catch (error) {
      addResult(`❌ 基本模型测试失败: ${error}`);
    } finally {
      isLoading = false;
    }
  }

  // 测试流式生成
  async function testStreamGeneration() {
    isLoading = true;
    addResult("开始测试 Candle 流式生成...");

    try {
      const result = await invoke<string>("test_candle_stream");
      addResult(`✅ 流式生成测试成功: ${result}`);
    } catch (error) {
      addResult(`❌ 流式生成测试失败: ${error}`);
    } finally {
      isLoading = false;
    }
  }

  // 获取支持的模型列表
  async function getModels() {
    isLoading = true;
    addResult("获取 Candle 支持的模型列表...");

    try {
      const result = await invoke<string[]>("get_candle_models");
      models = result;
      addResult(`✅ 获取到 ${result.length} 个模型: ${result.join(", ")}`);
    } catch (error) {
      addResult(`❌ 获取模型列表失败: ${error}`);
    } finally {
      isLoading = false;
    }
  }

  // 检查健康状态
  async function checkHealth() {
    isLoading = true;
    addResult("检查 Candle 提供者健康状态...");

    try {
      const result = await invoke<boolean>("check_candle_health");
      healthStatus = result;
      addResult(`✅ 健康状态检查完成: ${result ? "正常" : "异常"}`);
    } catch (error) {
      addResult(`❌ 健康状态检查失败: ${error}`);
      healthStatus = false;
    } finally {
      isLoading = false;
    }
  }

  // 使用管理器测试
  async function testWithManager() {
    isLoading = true;
    addResult("使用 LLM 管理器测试 Candle...");

    try {
      const result = await invoke<string>("test_candle_with_manager");
      addResult(`✅ 管理器测试成功: ${result}`);
    } catch (error) {
      addResult(`❌ 管理器测试失败: ${error}`);
    } finally {
      isLoading = false;
    }
  }

  // 运行所有测试
  async function runAllTests() {
    clearResults();
    addResult("开始运行所有 Candle 测试...");

    await checkHealth();
    await getModels();
    await testBasicModel();
    await testStreamGeneration();
    await testWithManager();

    addResult("所有测试完成！");
  }

  onMount(() => {
    addResult("Candle 测试组件已加载");
  });
</script>

<div class="candle-test-container">
  <div class="test-header">
    <h2 class="test-title">🧪 Candle 模型测试</h2>
    <p class="test-description">
      测试 Candle 本地 AI 模型的功能。Candle 是一个用 Rust 编写的机器学习框架，支持本地运行 Qwen
      模型。
    </p>
  </div>

  <!-- 状态显示 -->
  <div class="status-grid">
    <div class="status-card status-health">
      <h3 class="status-title">健康状态</h3>
      <div class="status-value">
        {#if healthStatus === null}
          ❓ 未知
        {:else if healthStatus}
          ✅ 正常
        {:else}
          ❌ 异常
        {/if}
      </div>
    </div>

    <div class="status-card status-models">
      <h3 class="status-title">支持模型</h3>
      <div class="status-value">
        📦 {models.length} 个
      </div>
    </div>

    <div class="status-card status-results">
      <h3 class="status-title">测试结果</h3>
      <div class="status-value">
        📋 {testResults.length} 条
      </div>
    </div>
  </div>

  <!-- 测试按钮 -->
  <div class="buttons-grid">
    <button on:click={checkHealth} disabled={isLoading} class="test-button test-button-blue">
      {#if isLoading}
        🔄 检查中...
      {:else}
        🏥 健康检查
      {/if}
    </button>

    <button on:click={getModels} disabled={isLoading} class="test-button test-button-green">
      {#if isLoading}
        🔄 获取中...
      {:else}
        📦 获取模型
      {/if}
    </button>

    <button on:click={testBasicModel} disabled={isLoading} class="test-button test-button-purple">
      {#if isLoading}
        🔄 测试中...
      {:else}
        🤖 基本测试
      {/if}
    </button>

    <button
      on:click={testStreamGeneration}
      disabled={isLoading}
      class="test-button test-button-orange"
    >
      {#if isLoading}
        🔄 测试中...
      {:else}
        🌊 流式测试
      {/if}
    </button>

    <button on:click={testWithManager} disabled={isLoading} class="test-button test-button-indigo">
      {#if isLoading}
        🔄 测试中...
      {:else}
        🔧 管理器测试
      {/if}
    </button>

    <button on:click={runAllTests} disabled={isLoading} class="test-button test-button-red">
      {#if isLoading}
        🔄 运行中...
      {:else}
        🚀 运行所有测试
      {/if}
    </button>
  </div>

  <!-- 模型列表 -->
  {#if models.length > 0}
    <div class="models-section">
      <h3 class="section-title">支持的模型列表</h3>
      <div class="models-container">
        <div class="models-grid">
          {#each models as model}
            <div class="model-item">
              {model}
            </div>
          {/each}
        </div>
      </div>
    </div>
  {/if}

  <!-- 测试结果日志 -->
  <div class="logs-section">
    <div class="logs-header">
      <h3 class="section-title">测试日志</h3>
      <button on:click={clearResults} class="clear-button"> 清空日志 </button>
    </div>

    <div class="logs-container">
      {#if testResults.length === 0}
        <div class="logs-empty">暂无测试结果...</div>
      {:else}
        {#each testResults as result}
          <div class="log-entry">{result}</div>
        {/each}
      {/if}
    </div>
  </div>

  <!-- 说明信息 -->
  <div class="info-section">
    <div class="info-card info-usage">
      <h4 class="info-title">💡 使用说明</h4>
      <ul class="info-list">
        <li>• <strong>健康检查</strong>：验证 Candle 提供者是否正常初始化</li>
        <li>• <strong>获取模型</strong>：列出所有支持的 Qwen 模型</li>
        <li>• <strong>基本测试</strong>：测试简单的文本生成功能</li>
        <li>• <strong>流式测试</strong>：测试流式文本生成</li>
        <li>• <strong>管理器测试</strong>：通过 LLM 管理器测试集成</li>
        <li>• <strong>运行所有测试</strong>：按顺序执行所有测试项目</li>
      </ul>
    </div>

    <div class="info-card info-warning">
      <h4 class="info-title">⚠️ 注意事项</h4>
      <ul class="info-list">
        <li>• 首次运行可能需要下载模型文件，请确保网络连接正常</li>
        <li>• 模型运行需要一定的系统资源，建议关闭其他占用资源的应用</li>
        <li>• 如果测试失败，请检查系统日志获取详细错误信息</li>
        <li>• 默认使用 CPU 运行，如需 GPU 加速请修改配置</li>
      </ul>
    </div>
  </div>
</div>

<style>
  .candle-test-container {
    padding: 1.5rem;
    max-width: 1200px;
    margin: 0 auto;
  }

  .test-header {
    margin-bottom: 2rem;
  }

  .test-title {
    font-size: 2rem;
    font-weight: bold;
    color: #1f2937;
    margin-bottom: 0.5rem;
  }

  :global(.dark) .test-title {
    color: #f9fafb;
  }

  .test-description {
    color: #6b7280;
    line-height: 1.6;
  }

  :global(.dark) .test-description {
    color: #d1d5db;
  }

  .status-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
    margin-bottom: 2rem;
  }

  .status-card {
    padding: 1rem;
    border-radius: 0.5rem;
    border: 1px solid #e5e7eb;
  }

  :global(.dark) .status-card {
    border-color: #374151;
  }

  .status-health {
    background: #dbeafe;
  }
  :global(.dark) .status-health {
    background: rgba(59, 130, 246, 0.1);
  }

  .status-models {
    background: #dcfce7;
  }
  :global(.dark) .status-models {
    background: rgba(34, 197, 94, 0.1);
  }

  .status-results {
    background: #fdf4ff;
  }
  :global(.dark) .status-results {
    background: rgba(168, 85, 247, 0.1);
  }

  .status-title {
    font-weight: 600;
    font-size: 0.875rem;
    margin-bottom: 0.5rem;
  }

  .status-value {
    font-size: 1.5rem;
    font-weight: bold;
  }

  .buttons-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 1rem;
    margin-bottom: 2rem;
  }

  .test-button {
    padding: 0.75rem 1rem;
    border-radius: 0.5rem;
    border: none;
    color: white;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .test-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .test-button-blue {
    background: #3b82f6;
  }
  .test-button-blue:hover:not(:disabled) {
    background: #2563eb;
  }

  .test-button-green {
    background: #10b981;
  }
  .test-button-green:hover:not(:disabled) {
    background: #059669;
  }

  .test-button-purple {
    background: #8b5cf6;
  }
  .test-button-purple:hover:not(:disabled) {
    background: #7c3aed;
  }

  .test-button-orange {
    background: #f59e0b;
  }
  .test-button-orange:hover:not(:disabled) {
    background: #d97706;
  }

  .test-button-indigo {
    background: #6366f1;
  }
  .test-button-indigo:hover:not(:disabled) {
    background: #4f46e5;
  }

  .test-button-red {
    background: #ef4444;
  }
  .test-button-red:hover:not(:disabled) {
    background: #dc2626;
  }

  .models-section {
    margin-bottom: 2rem;
  }

  .section-title {
    font-size: 1.25rem;
    font-weight: 600;
    color: #1f2937;
    margin-bottom: 1rem;
  }

  :global(.dark) .section-title {
    color: #f9fafb;
  }

  .models-container {
    background: #f9fafb;
    padding: 1rem;
    border-radius: 0.5rem;
    border: 1px solid #e5e7eb;
  }

  :global(.dark) .models-container {
    background: #374151;
    border-color: #4b5563;
  }

  .models-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 0.5rem;
  }

  .model-item {
    background: white;
    padding: 0.75rem;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    border: 1px solid #e5e7eb;
  }

  :global(.dark) .model-item {
    background: #4b5563;
    border-color: #6b7280;
    color: #f9fafb;
  }

  .logs-section {
    margin-bottom: 2rem;
  }

  .logs-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .clear-button {
    background: #6b7280;
    color: white;
    padding: 0.5rem 1rem;
    border-radius: 0.375rem;
    border: none;
    font-size: 0.875rem;
    cursor: pointer;
    transition: background-color 0.2s;
  }

  .clear-button:hover {
    background: #4b5563;
  }

  .logs-container {
    background: #000;
    color: #22c55e;
    padding: 1rem;
    border-radius: 0.5rem;
    height: 300px;
    overflow-y: auto;
    font-family: "Courier New", monospace;
    font-size: 0.875rem;
    line-height: 1.4;
  }

  .logs-empty {
    color: #6b7280;
  }

  .log-entry {
    margin-bottom: 0.25rem;
    word-wrap: break-word;
  }

  .info-section {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 1rem;
  }

  .info-card {
    padding: 1rem;
    border-radius: 0.5rem;
    border: 1px solid #e5e7eb;
  }

  :global(.dark) .info-card {
    border-color: #374151;
  }

  .info-usage {
    background: #fefce8;
  }
  :global(.dark) .info-usage {
    background: rgba(234, 179, 8, 0.1);
  }

  .info-warning {
    background: #dbeafe;
  }
  :global(.dark) .info-warning {
    background: rgba(59, 130, 246, 0.1);
  }

  .info-title {
    font-weight: 600;
    margin-bottom: 0.75rem;
    color: #1f2937;
  }

  :global(.dark) .info-title {
    color: #f9fafb;
  }

  .info-list {
    list-style: none;
    padding: 0;
    margin: 0;
    color: #4b5563;
    font-size: 0.875rem;
    line-height: 1.6;
  }

  :global(.dark) .info-list {
    color: #d1d5db;
  }

  .info-list li {
    margin-bottom: 0.5rem;
  }

  /* 滚动条样式 */
  .logs-container::-webkit-scrollbar {
    width: 8px;
  }

  .logs-container::-webkit-scrollbar-track {
    background: #1a1a1a;
  }

  .logs-container::-webkit-scrollbar-thumb {
    background: #22c55e;
    border-radius: 4px;
  }

  .logs-container::-webkit-scrollbar-thumb:hover {
    background: #16a34a;
  }

  /* 响应式设计 */
  @media (max-width: 768px) {
    .candle-test-container {
      padding: 1rem;
    }

    .status-grid {
      grid-template-columns: 1fr;
    }

    .buttons-grid {
      grid-template-columns: 1fr;
    }

    .info-section {
      grid-template-columns: 1fr;
    }

    .logs-header {
      flex-direction: column;
      gap: 0.5rem;
      align-items: stretch;
    }
  }
</style>
