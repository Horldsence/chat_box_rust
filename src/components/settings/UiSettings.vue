<template>
  <div class="settings-form">
    <h2>界面设置</h2>
    <el-form :model="form" label-position="top">
      <el-form-item label="侧边栏宽度">
        <el-input v-model="form.ui.sidebar_width" placeholder="例如: 280px">
          <template #append>
            <el-tooltip
              content="设置对话列表侧边栏宽度（如: 280px）"
              placement="top"
            >
              <el-icon><QuestionFilled /></el-icon>
            </el-tooltip>
          </template>
        </el-input>
      </el-form-item>

      <el-form-item label="界面主题">
        <el-select
          v-model="form.ui.theme"
          style="width: 100%"
          @change="applyTheme"
        >
          <el-option label="浅色主题" value="light"></el-option>
          <el-option label="深色主题" value="dark"></el-option>
        </el-select>
      </el-form-item>

      <el-form-item label="界面语言">
        <el-select v-model="form.ui.language" style="width: 100%">
          <el-option label="简体中文" value="zh-CN"></el-option>
          <el-option label="English" value="en-US"></el-option>
        </el-select>
      </el-form-item>

      <el-form-item>
        <el-button type="primary" @click="saveSettings">保存设置</el-button>
        <el-button @click="resetForm">重置</el-button>
      </el-form-item>
    </el-form>

    <!-- 主题预览 -->
    <div class="theme-preview">
      <h3>主题预览</h3>
      <div class="preview-container">
        <el-card>
          <template #header>
            <div class="card-header">
              <span>当前主题样式</span>
            </div>
          </template>
          <div>
            <el-row :gutter="12">
              <el-col :span="12">
                <el-button type="primary">主要按钮</el-button>
                <el-button type="success">成功按钮</el-button>
              </el-col>
              <el-col :span="12">
                <el-switch v-model="demoSwitch" />
                <el-slider v-model="demoSlider" />
              </el-col>
            </el-row>

            <el-divider />

            <el-row :gutter="12">
              <el-col :span="12">
                <el-input placeholder="输入框示例" v-model="demoInput" />
              </el-col>
              <el-col :span="12">
                <el-tag>标签</el-tag>
                <el-tag type="success">成功</el-tag>
                <el-tag type="warning">警告</el-tag>
              </el-col>
            </el-row>
          </div>
        </el-card>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ref, onMounted, watch, nextTick } from "vue";
import { QuestionFilled } from "@element-plus/icons-vue";
import { useDark, useToggle } from "@vueuse/core";

const props = defineProps({
  config: {
    type: Object,
    required: true,
  },
});

const emit = defineEmits(["save"]);

// 主题切换状态
const isDark = useDark({
  selector: "html",
  attribute: "class",
  valueDark: "dark",
  valueLight: "",
  storageKey: "chat-box-theme-preference",
});
const toggleDark = useToggle(isDark);

// 演示用状态
const demoSwitch = ref(true);
const demoSlider = ref(50);
const demoInput = ref("这是个输入框示例");

const form = ref({
  ui: {
    sidebar_width: "280px",
    theme: "light",
    language: "zh-CN",
  },
});

// 监听props变化更新表单
watch(
  () => props.config,
  (newConfig) => {
    form.value = JSON.parse(JSON.stringify(newConfig));
    // 当配置改变时应用主题
    applyTheme(form.value.ui.theme);
  },
  { deep: true }
);

// 应用主题
const applyTheme = (theme: string) => {
  console.log(`应用主题: ${theme}`);

  // 使用 Element Plus 的深色模式
  if (theme === "dark") {
    document.documentElement.classList.add("dark");
    isDark.value = true;
  } else {
    document.documentElement.classList.remove("dark");
    isDark.value = false;
  }

  // 更新 element-plus 的主题
  document.documentElement.setAttribute("data-theme", theme);

  // 通知 vueuse 主题已更改
  toggleDark(theme === "dark");
};

onMounted(() => {
  form.value = JSON.parse(JSON.stringify(props.config));

  // 组件挂载时应用当前主题
  nextTick(() => {
    applyTheme(form.value.ui.theme);
  });
});

const saveSettings = () => {
  emit("save", form.value);
};

const resetForm = () => {
  form.value = JSON.parse(JSON.stringify(props.config));
  applyTheme(form.value.ui.theme); // 重置主题
};
</script>

<style scoped>
.settings-form {
  max-width: 800px;
}

.theme-preview {
  margin-top: 30px;
}

.preview-container {
  margin-top: 16px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
</style>
