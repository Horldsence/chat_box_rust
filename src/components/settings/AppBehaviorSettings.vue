<template>
  <div class="settings-form">
    <h2>应用行为设置</h2>
    <el-form :model="form" label-position="top">
      <el-form-item label="日志级别">
        <el-select v-model="form.app_behavior.log_level" style="width: 100%">
          <el-option label="调试" value="debug"></el-option>
          <el-option label="信息" value="info"></el-option>
          <el-option label="警告" value="warn"></el-option>
          <el-option label="错误" value="error"></el-option>
        </el-select>
      </el-form-item>
      
      <el-form-item label="默认对话标题">
        <el-input v-model="form.app_behavior.default_conversation_title" placeholder="例如: 新对话" />
      </el-form-item>
      
      <el-form-item label="欢迎消息">
        <el-input v-model="form.app_behavior.welcome_message" placeholder="例如: 欢迎使用聊天应用!" />
      </el-form-item>
      
      <el-divider>高级设置</el-divider>
      
      <el-form-item label="消息块缓冲大小">
        <el-tooltip content="调整AI响应的文本块大小，较大的值可能减少UI更新频率" placement="top">
          <el-input-number 
            v-model="form.app_behavior.message_chunk_buffer_size" 
            :min="1" 
            :max="10" />
        </el-tooltip>
      </el-form-item>
      
      <el-form-item label="消息发送间隔(毫秒)">
        <el-tooltip content="调整AI响应的发送间隔，较大的值可能减少UI更新频率" placement="top">
          <el-input-number 
            v-model="form.app_behavior.message_chunk_send_interval_ms" 
            :min="1" 
            :max="100" />
        </el-tooltip>
      </el-form-item>
      
      <el-form-item>
        <el-button type="primary" @click="saveSettings">保存设置</el-button>
        <el-button @click="resetForm">重置</el-button>
      </el-form-item>
    </el-form>
  </div>
</template>

<script lang="ts" setup>
import { ref, onMounted, watch } from 'vue'
import { QuestionFilled } from '@element-plus/icons-vue'

const props = defineProps({
  config: {
    type: Object,
    required: true
  }
})

const emit = defineEmits(['save'])

const form = ref({
  app_behavior: {
    log_level: "info",
    default_conversation_title: "",
    welcome_message: "",
    message_chunk_buffer_size: 2,
    message_chunk_send_interval_ms: 3
  }
})

// 监听props变化更新表单
watch(() => props.config, (newConfig) => {
  form.value = JSON.parse(JSON.stringify(newConfig))
}, { deep: true })

onMounted(() => {
  form.value = JSON.parse(JSON.stringify(props.config))
})

const saveSettings = () => {
  emit('save', form.value)
}

const resetForm = () => {
  form.value = JSON.parse(JSON.stringify(props.config))
}
</script>

<style scoped>
.settings-form {
  max-width: 800px;
}
</style>