<template>
  <div class="menu-container">
    <el-menu
      :default-active="activeIndex"
      class="el-menu-vertical-demo"
      :collapse="isCollapse"
      @select="handleSelect"
    >
      <!-- 聊天功能模块 -->
      <el-menu-item index="chat">
        <el-icon><ChatDotSquare /></el-icon>
        <template #title>聊天</template>
      </el-menu-item>

      <!-- AI模型设置 -->
      <el-sub-menu index="ai-settings">
        <template #title>
          <el-icon><Monitor /></el-icon>
          <span>AI设置</span>
        </template>
        <el-menu-item index="ai-model">模型配置</el-menu-item>
        <el-menu-item index="ai-behavior">行为设置</el-menu-item>
      </el-sub-menu>

      <!-- 语音功能设置 -->
      <el-menu-item index="voice">
        <el-icon><Microphone /></el-icon>
        <template #title>语音识别</template>
      </el-menu-item>

      <!-- 数据库功能设置 -->
      <el-menu-item index="database">
        <el-icon><ChatLineSquare /></el-icon>
        <template #title>历史聊天</template>
      </el-menu-item>

      <!-- 界面设置 -->
      <el-menu-item index="ui">
        <el-icon><Operation /></el-icon>
        <template #title>界面设置</template>
      </el-menu-item>

      <!-- 帮助和关于 -->
      <el-sub-menu index="help">
        <template #title>
          <el-icon><QuestionFilled /></el-icon>
          <span>帮助</span>
        </template>
        <el-menu-item index="guide">使用指南</el-menu-item>
        <el-menu-item index="about">关于</el-menu-item>
      </el-sub-menu>
    </el-menu>

    <!-- 折叠/展开按钮 -->
    <div class="collapse-control">
      <el-button
        circle
        size="small"
        @click="toggleCollapse"
      >
        <el-icon v-if="isCollapse"><ArrowLeftBold /></el-icon>
        <el-icon v-else><ArrowRightBold /></el-icon>
      </el-button>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ref } from "vue";
import {
  ChatDotSquare,
  Monitor,
  Microphone,
  Operation,
  QuestionFilled,
  ChatLineSquare,
  ArrowLeftBold,
  ArrowRightBold
} from "@element-plus/icons-vue";

const isCollapse = ref(true);
const activeIndex = ref("chat");

const emit = defineEmits(["select"]);

const toggleCollapse = () => {
  isCollapse.value = !isCollapse.value;
};

const handleSelect = (index: string) => {
  activeIndex.value = index;
  emit("select", index);
};
</script>

<style>
.menu-container {
  position: relative;
  height: 100%;
}

.el-menu-vertical-demo:not(.el-menu--collapse) {
  width: 200px;
  min-height: 400px;
}

.el-menu {
  height: 100%;
  border-right: none;
}

.collapse-control {
  position: absolute;
  right: -12px;
  top: 20px;
  z-index: 10;
}
</style>
