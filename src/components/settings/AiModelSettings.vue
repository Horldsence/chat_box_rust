<template>
  <div class="settings-form">
    <h2>AI模型设置</h2>
    <el-form :model="form" label-position="top">
      <el-form-item label="模型名称">
        <el-input v-model="form.ai_model.model_name" placeholder="例如: qwen2.5:0.5b">
          <template #append>
            <el-tooltip content="AI大语言模型名称，根据你的Ollama安装情况选择" placement="top">
              <el-icon><QuestionFilled /></el-icon>
            </el-tooltip>
          </template>
        </el-input>
      </el-form-item>
      
      <el-form-item label="服务器地址">
        <el-input v-model="form.ai_model.server_url" placeholder="例如: http://localhost">
          <template #append>
            <el-tooltip content="Ollama API服务器地址" placement="top">
              <el-icon><QuestionFilled /></el-icon>
            </el-tooltip>
          </template>
        </el-input>
      </el-form-item>
      
      <el-form-item label="服务器端口">
        <el-input-number v-model="form.ai_model.server_port" :min="1" :max="65535" />
      </el-form-item>
      
      <el-form-item label="系统提示词">
        <el-input 
          type="textarea" 
          v-model="form.ai_model.system_prompt" 
          :rows="6"
          placeholder="设置AI助手的人格和行为">
        </el-input>
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
  ai_model: {
    model_name: "",
    server_url: "",
    server_port: 11434,
    system_prompt: ""
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