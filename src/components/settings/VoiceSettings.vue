<template>
  <div class="settings-form">
    <h2>语音识别设置</h2>
    <el-form :model="form" label-position="top">
      <el-form-item label="启用自定义语音识别">
        <el-switch v-model="form.voice.enabled" />
      </el-form-item>
      
      <el-form-item label="语音模型路径" :disabled="!form.voice.enabled">
        <el-input v-model="form.voice.model_path" placeholder="例如: model/vosk-model-small-cn-0.22" :disabled="!form.voice.enabled">
          <template #append>
            <el-tooltip content="语音识别模型文件夹路径" placement="top">
              <el-icon><QuestionFilled /></el-icon>
            </el-tooltip>
          </template>
        </el-input>
      </el-form-item>
      
      <el-form-item label="录音超时时间（秒）" :disabled="!form.voice.enabled">
        <el-input-number 
          v-model="form.voice.timeout_seconds" 
          :min="5" 
          :max="60" 
          :disabled="!form.voice.enabled" />
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
  voice: {
    enabled: true,
    model_path: "",
    timeout_seconds: 15
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