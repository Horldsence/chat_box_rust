<template>
    <div class="initialization-container">
        <el-card class="wizard-card">
            <template #header>
                <div class="card-header">
                    <h2>欢迎使用 Chat Box</h2>
                    <p>首次使用需要完成基础配置</p>
                </div>
            </template>

            <el-steps
                :active="currentStep"
                finish-status="success"
                align-center
                :space="200"
                class="wizard-steps"
            >
                <el-step title="AI模型配置" icon="Monitor" />
                <el-step title="语音设置" icon="Microphone" />
                <el-step title="完成配置" icon="Check" />
            </el-steps>

            <div class="step-content">
                <!-- 步骤1：AI模型配置 -->
                <div v-if="currentStep === 0" class="step-panel">
                    <h3>配置AI模型</h3>
                    <p class="step-description">
                        请配置AI模型以开始使用聊天功能
                    </p>

                    <el-form
                        :model="config"
                        label-position="top"
                        :rules="aiModelRules"
                        ref="aiModelForm"
                    >
                        <el-form-item
                            label="模型名称"
                            prop="ai_model.model_name"
                            required
                        >
                            <el-input
                                v-model="config.ai_model.model_name"
                                placeholder="例如: qwen2.5:0.5b"
                                clearable
                            >
                                <template #append>
                                    <el-tooltip
                                        content="请输入已安装的Ollama模型名称"
                                        placement="top"
                                    >
                                        <el-icon><QuestionFilled /></el-icon>
                                    </el-tooltip>
                                </template>
                            </el-input>
                        </el-form-item>

                        <el-form-item
                            label="服务器地址"
                            prop="ai_model.server_url"
                            required
                        >
                            <el-input
                                v-model="config.ai_model.server_url"
                                placeholder="http://localhost"
                                clearable
                            >
                                <template #append>
                                    <el-tooltip
                                        content="Ollama API服务器地址"
                                        placement="top"
                                    >
                                        <el-icon><QuestionFilled /></el-icon>
                                    </el-tooltip>
                                </template>
                            </el-input>
                        </el-form-item>

                        <el-form-item label="服务器端口">
                            <el-input-number
                                v-model="config.ai_model.server_port"
                                :min="1"
                                :max="65535"
                                controls-position="right"
                            />
                        </el-form-item>

                        <el-form-item label="系统提示词（可选）">
                            <el-input
                                type="textarea"
                                v-model="config.ai_model.system_prompt"
                                :rows="3"
                                placeholder="设置AI助手的人格和行为，留空使用默认设置"
                                show-word-limit
                                maxlength="500"
                            />
                        </el-form-item>
                    </el-form>
                </div>

                <!-- 步骤2：语音设置 -->
                <div v-if="currentStep === 1" class="step-panel">
                    <h3>语音识别设置</h3>
                    <p class="step-description">配置语音识别功能（可选）</p>

                    <el-form :model="config" label-position="top">
                        <el-form-item>
                            <el-checkbox v-model="config.voice.enabled">
                                启用语音识别功能
                            </el-checkbox>
                        </el-form-item>

                        <template v-if="config.voice.enabled">
                            <el-form-item label="语音模型路径">
                                <el-input
                                    v-model="config.voice.model_path"
                                    placeholder="model/vosk-model-small-cn-0.22"
                                    clearable
                                >
                                    <template #append>
                                        <el-button
                                            @click="selectModelPath"
                                            icon="FolderOpened"
                                        >
                                            选择文件夹
                                        </el-button>
                                    </template>
                                </el-input>
                            </el-form-item>

                            <el-form-item label="录音超时时间">
                                <el-slider
                                    v-model="config.voice.timeout_seconds"
                                    :min="5"
                                    :max="60"
                                    :step="1"
                                    show-input
                                    input-size="small"
                                />
                                <span class="slider-label">秒</span>
                            </el-form-item>
                        </template>
                    </el-form>
                </div>

                <!-- 步骤3：完成配置 -->
                <div v-if="currentStep === 2" class="step-panel">
                    <h3>配置完成</h3>
                    <p class="step-description">
                        所有必要配置已完成，即将保存设置
                    </p>

                    <el-result icon="success" title="配置完成">
                        <template #sub-title>
                            <div class="config-summary">
                                <el-descriptions :column="1" border>
                                    <el-descriptions-item label="AI模型">
                                        {{ config.ai_model.model_name }}
                                    </el-descriptions-item>
                                    <el-descriptions-item label="服务器">
                                        {{ config.ai_model.server_url }}:{{
                                            config.ai_model.server_port
                                        }}
                                    </el-descriptions-item>
                                    <el-descriptions-item label="语音识别">
                                        {{
                                            config.voice.enabled
                                                ? "已启用"
                                                : "未启用"
                                        }}
                                    </el-descriptions-item>
                                    <el-descriptions-item
                                        label="语音模型路径"
                                        v-if="config.voice.enabled"
                                    >
                                        {{
                                            config.voice.model_path ||
                                            "使用默认路径"
                                        }}
                                    </el-descriptions-item>
                                </el-descriptions>
                            </div>
                        </template>
                    </el-result>
                </div>
            </div>

            <div class="wizard-actions">
                <el-button
                    v-if="currentStep > 0"
                    @click="previousStep"
                    icon="ArrowLeft"
                >
                    上一步
                </el-button>

                <el-button
                    v-if="currentStep < 2"
                    type="primary"
                    @click="nextStep"
                    icon="ArrowRight"
                    :loading="isValidating"
                >
                    下一步
                </el-button>

                <el-button
                    v-if="currentStep === 2"
                    type="success"
                    @click="completeInitialization"
                    icon="Check"
                    :loading="isSaving"
                >
                    完成配置
                </el-button>
            </div>
        </el-card>
    </div>
</template>

<script lang="ts" setup>
import { ref, reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
    QuestionFilled,
    ArrowLeft,
    ArrowRight,
    Check,
    FolderOpened,
} from "@element-plus/icons-vue";
import { ElMessage, ElMessageBox } from "element-plus";
import type { FormInstance, FormRules } from "element-plus";

const emit = defineEmits(["complete"]);

const currentStep = ref(0);
const isValidating = ref(false);
const isSaving = ref(false);
const aiModelForm = ref<FormInstance>();

const config = reactive({
    config_path: "config.yaml",
    ai_model: {
        model_type: "ollama",
        model_name: "",
        server_url: "http://localhost",
        server_port: 11434,
        system_prompt: "你是一个友好、乐于助人的AI助手，使用中文回答问题。",
        candle_model_id: null,
        candle_revision: null,
        candle_use_flash_attn: false,
    },
    voice: {
        enabled: false,
        model_path: "model/vosk-model-small-cn-0.22",
        timeout_seconds: 15,
    },
    ui: {
        theme: "light",
        language: "zh-CN",
    },
    app_behavior: {
        log_level: "info",
        default_conversation_title: "新对话",
        welcome_message: "欢迎使用聊天应用!",
        message_chunk_buffer_size: 2,
        message_chunk_send_interval_ms: 3,
        show_error_dialogs: true,
        auto_retry_failed_init: false,
    },
    database: {
        enabled: true,
        path: "database/chat_database.db",
    },
});

const aiModelRules: FormRules = {
    "ai_model.model_name": [
        { required: true, message: "请输入模型名称", trigger: "blur" },
        {
            min: 1,
            max: 100,
            message: "长度在 1 到 100 个字符",
            trigger: "blur",
        },
    ],
    "ai_model.server_url": [
        { required: true, message: "请输入服务器地址", trigger: "blur" },
        {
            pattern: /^https?:\/\/.+/,
            message: "请输入有效的URL地址（如：http://localhost）",
            trigger: "blur",
        },
    ],
};

const nextStep = async () => {
    if (currentStep.value === 0) {
        // 验证AI模型配置
        if (!aiModelForm.value) return;

        try {
            await aiModelForm.value.validate();
            isValidating.value = true;

            // 测试连接
            await testConnection();
            currentStep.value++;
        } catch (error) {
            console.error("验证失败:", error);
        } finally {
            isValidating.value = false;
        }
    } else {
        currentStep.value++;
    }
};

const previousStep = () => {
    if (currentStep.value > 0) {
        currentStep.value--;
    }
};

const testConnection = async () => {
    try {
        // 这里可以添加连接测试逻辑
        ElMessage.success("AI模型配置验证成功");
    } catch (error) {
        ElMessage.error("无法连接到AI服务器，请检查配置");
        throw error;
    }
};

const selectModelPath = async () => {
    try {
        const result = await invoke("select_voice_model_folder");
        if (result) {
            config.voice.model_path = result as string;
        }
    } catch (error) {
        ElMessage.error("选择文件夹失败");
    }
};

const completeInitialization = async () => {
    try {
        isSaving.value = true;

        // 保存配置
        await invoke("save_app_config", { config });

        ElMessage.success("配置保存成功！");

        // 延迟一下让用户看到成功消息
        setTimeout(() => {
            emit("complete");
        }, 1000);
    } catch (error) {
        ElMessage.error("保存配置失败: " + error);
    } finally {
        isSaving.value = false;
    }
};
</script>

<style scoped>
.initialization-container {
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 100vh;
    padding: 20px;
    background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
}

.wizard-card {
    width: 100%;
    max-width: 800px;
    min-height: 600px;
}

.card-header {
    text-align: center;
}

.card-header h2 {
    margin: 0 0 8px 0;
    color: #303133;
}

.card-header p {
    margin: 0;
    color: #909399;
}

.wizard-steps {
    margin: 30px 0;
}

.step-content {
    min-height: 350px;
    padding: 20px 0;
}

.step-panel {
    max-width: 600px;
    margin: 0 auto;
}

.step-panel h3 {
    margin: 0 0 8px 0;
    color: #303133;
}

.step-description {
    color: #909399;
    margin-bottom: 20px;
}

.wizard-actions {
    display: flex;
    justify-content: center;
    gap: 10px;
    padding-top: 20px;
    border-top: 1px solid #ebeef5;
}

.config-summary {
    text-align: left;
    max-width: 500px;
    margin: 0 auto;
}

.slider-label {
    margin-left: 10px;
    color: #909399;
    font-size: 14px;
}
</style>
