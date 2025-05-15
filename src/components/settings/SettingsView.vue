<template>
  <div class="settings-container">
    <component
      :is="currentSettingComponent"
      :config="config"
      @save="saveConfig"
    />
  </div>
</template>

<script lang="ts" setup>
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { ElMessage } from "element-plus";
import AiModelSettings from "./AiModelSettings.vue";
import VoiceSettings from "./VoiceSettings.vue";
import UiSettings from "./UiSettings.vue";
import AppBehaviorSettings from "./AppBehaviorSettings.vue";
import DatabaseConfigPanel from "./DatabaseConfigPanel.vue";

const props = defineProps({
  settingType: {
    type: String,
    default: "ai-model",
  },
});

const config = ref({
  ai_model: {
    model_name: "",
    server_url: "",
    server_port: 11434,
    system_prompt: "",
  },
  voice: {
    enabled: true,
    model_path: "",
    timeout_seconds: 15,
  },
  ui: {
    sidebar_width: "",
    theme: "light",
    language: "zh-CN",
  },
  app_behavior: {
    log_level: "info",
    default_conversation_title: "",
    welcome_message: "",
    message_chunk_buffer_size: 2,
    message_chunk_send_interval_ms: 3,
  },
});

// 根据当前选中的设置类型显示对应组件
const currentSettingComponent = computed(() => {
  switch (props.settingType) {
    case "ai-model":
      return AiModelSettings;
    case "ai-behavior":
      return AppBehaviorSettings;
    case "database":
      return DatabaseConfigPanel;
    case "voice":
      return VoiceSettings;
    case "ui":
      return UiSettings;
    default:
      return AiModelSettings;
  }
});

onMounted(async () => {
  await loadConfig();
});

const loadConfig = async () => {
  try {
    const appConfig = await invoke("get_app_config") as typeof config.value;
    config.value = appConfig;
  } catch (error) {
    ElMessage.error(`加载配置失败: ${error}`);
  }
};

const saveConfig = async (newConfig: { ai_model: { model_name: string; server_url: string; server_port: number; system_prompt: string; }; voice: { enabled: boolean; model_path: string; timeout_seconds: number; }; ui: { sidebar_width: string; theme: string; language: string; }; app_behavior: { log_level: string; default_conversation_title: string; welcome_message: string; message_chunk_buffer_size: number; message_chunk_send_interval_ms: number; }; } | { ai_model: { model_name: string; server_url: string; server_port: number; system_prompt: string; }; voice: { enabled: boolean; model_path: string; timeout_seconds: number; }; ui: { sidebar_width: string; theme: string; language: string; }; app_behavior: { log_level: string; default_conversation_title: string; welcome_message: string; message_chunk_buffer_size: number; message_chunk_send_interval_ms: number; }; }) => {
  try {
    await invoke("save_app_config", { config: newConfig });
    config.value = newConfig;
    ElMessage.success("保存成功，部分设置可能需要重启应用后生效");
  } catch (error) {
    ElMessage.error(`保存配置失败: ${error}`);
  }
};
</script>

<style scoped>
.settings-container {
  padding: 20px;
  height: 100%;
  overflow-y: auto;
}
</style>
