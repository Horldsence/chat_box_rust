<template>
  <div class="error-handler">
    <!-- 系统状态指示器 -->
    <div v-if="showStatusIndicator" class="status-indicator" :class="statusClass">
      <el-icon :size="16">
        <CircleCheckFilled v-if="systemStatus === 'healthy'" />
        <WarningFilled v-else-if="systemStatus === 'warning'" />
        <CircleCloseFilled v-else />
      </el-icon>
      <span>{{ statusText }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { WarningFilled, CircleCheckFilled, CircleCloseFilled } from '@element-plus/icons-vue'
import { chatAPI } from '@/api'

// 响应式数据
const showStatusIndicator = ref(true)
const systemStatus = ref<'healthy' | 'warning' | 'error'>('healthy')

// 计算属性
const statusClass = computed(() => {
  return `status-${systemStatus.value}`
})

const statusText = computed(() => {
  switch (systemStatus.value) {
    case 'healthy':
      return '系统正常'
    case 'warning':
      return '系统警告'
    case 'error':
      return '系统错误'
    default:
      return '未知状态'
  }
})

// 定义组件的 props 和 emits
defineProps<{
  autoCheckHealth?: boolean
  showDialogOnCriticalError?: boolean
}>()

const emit = defineEmits<{
  statusChange: [status: string]
}>()

// 健康检查定时器
let healthCheckTimer: number | null = null

// 系统健康检查
const checkSystemHealth = async () => {
  try {
    const healthData = await chatAPI.getHealthStatus()

    // 分析健康状态
    const dbConnected = healthData.database_connected
    const voiceAvailable = healthData.voice_recognition_available
    const llmAvailable = healthData.llm_available

    if (!dbConnected || !llmAvailable) {
      systemStatus.value = 'error'
    } else if (!voiceAvailable) {
      systemStatus.value = 'warning'
    } else {
      systemStatus.value = 'healthy'
    }

    emit('statusChange', systemStatus.value)
  } catch (error) {
    systemStatus.value = 'error'
    console.error('健康检查失败:', error)
  }
}

// 连接检查
const checkConnection = async () => {
  try {
    await chatAPI.checkConnection()
  } catch (error) {
    console.error('连接检查失败:', error)
  }
}

// 清理方法
const cleanup = () => {
  if (healthCheckTimer) {
    clearInterval(healthCheckTimer)
    healthCheckTimer = null
  }
}

// 暴露方法给父组件
defineExpose({
  checkSystemHealth,
  checkConnection
})

// 生命周期
onMounted(() => {
  // 初始连接检查
  checkConnection()

  // 初始健康检查
  checkSystemHealth()

  // 定期健康检查 (每5分钟)
  healthCheckTimer = setInterval(checkSystemHealth, 5 * 60 * 1000)
})

onUnmounted(() => {
  cleanup()
})
</script>

<style scoped>
.error-handler {
  position: relative;
}

.status-indicator {
  position: fixed;
  bottom: 20px;
  right: 20px;
  display: flex;
  align-items: center;
  padding: 8px 12px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 500;
  z-index: 1000;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  backdrop-filter: blur(8px);
  transition: all 0.3s ease;
}

.status-indicator span {
  margin-left: 6px;
}

.status-healthy {
  background-color: rgba(103, 194, 58, 0.1);
  color: #67c23a;
  border: 1px solid rgba(103, 194, 58, 0.2);
}

.status-warning {
  background-color: rgba(230, 162, 60, 0.1);
  color: #e6a23c;
  border: 1px solid rgba(230, 162, 60, 0.2);
}

.status-error {
  background-color: rgba(245, 108, 108, 0.1);
  color: #f56c6c;
  border: 1px solid rgba(245, 108, 108, 0.2);
}

/* 响应式设计 */
@media (max-width: 768px) {
  .status-indicator {
    bottom: 10px;
    right: 10px;
    font-size: 11px;
    padding: 6px 10px;
  }
}
</style>
