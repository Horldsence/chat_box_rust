<template>
    <div class="menu-container">
        <el-menu
            :default-active="activeIndex"
            class="el-menu-vertical-demo"
            :collapse="isCollapse"
            @select="handleSelect"
        >
            <!-- 聊天功能模块 -->
            <el-sub-menu index="chat-options">
                <template #title>
                    <el-icon><ChatDotSquare /></el-icon>
                    <span>聊天</span>
                </template>
                <el-menu-item index="chat">内置聊天</el-menu-item>
                <el-menu-item index="chat-window" @click="openChatWindow"
                    >独立窗口</el-menu-item
                >
            </el-sub-menu>

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
                <el-menu-item index="about-inline">关于</el-menu-item>
                <el-menu-item index="about-window" @click="openAboutWindow"
                    >关于窗口</el-menu-item
                >
            </el-sub-menu>
        </el-menu>

        <!-- 折叠/展开按钮 -->
        <div class="collapse-control">
            <el-button circle size="small" @click="toggleCollapse">
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
    ArrowRightBold,
} from "@element-plus/icons-vue";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

const isCollapse = ref(true);
const activeIndex = ref("chat");

const emit = defineEmits(["select"]);

const toggleCollapse = () => {
    isCollapse.value = !isCollapse.value;
};

const handleSelect = (index: string) => {
    activeIndex.value = index;
    // 如果是内联关于页面，修改为 'about'
    if (index === "about-inline") {
        emit("select", "about");
    } else {
        emit("select", index);
    }
};

// 打开聊天窗口
const openChatWindow = async () => {
    try {
        // 检查聊天窗口是否已经存在
        const chatWindow = await WebviewWindow.getByLabel("chat-window");

        if (chatWindow) {
            // 如果窗口已存在，显示并聚焦
            await chatWindow.show();
            await chatWindow.setFocus();
            console.log("聊天窗口已激活");
        } else {
            // 创建新的聊天窗口
            const webview = new WebviewWindow("chat-window", {
                url: "chat.html",
                title: "Chat Box - 独立聊天窗口",
                width: 1000,
                height: 750,
                minWidth: 700,
                minHeight: 550,
                center: true,
                resizable: true,
                maximizable: true,
                decorations: true,
                alwaysOnTop: false,
                fullscreen: false,
                shadow: true,
            });

            // 监听窗口创建事件
            webview.once("tauri://created", function () {
                console.log("独立聊天窗口创建成功");
            });

            webview.once("tauri://error", function (e: any) {
                console.error("聊天窗口创建失败:", e);
                alert("无法打开聊天窗口，请检查应用配置。");
            });

            // 监听窗口关闭事件
            webview.once("tauri://close-requested", function () {
                console.log("聊天窗口将要关闭");
            });
        }
    } catch (error) {
        console.error("打开聊天窗口失败:", error);
        alert("聊天窗口打开失败: " + error);
    }
};

// 打开关于窗口
const openAboutWindow = async () => {
    try {
        // 检查关于窗口是否已经存在
        const aboutWindow = await WebviewWindow.getByLabel("about");

        if (aboutWindow) {
            // 如果窗口已存在，显示并聚焦
            await aboutWindow.show();
            await aboutWindow.setFocus();
            console.log("关于窗口已激活");
        } else {
            // 创建新的关于窗口
            const webview = new WebviewWindow("about", {
                url: "about.html",
                title: "Chat Box - 关于应用",
                width: 550,
                height: 700,
                minWidth: 480,
                minHeight: 620,
                center: true,
                resizable: true,
                maximizable: false,
                decorations: true,
                alwaysOnTop: false,
                shadow: true,
            });

            // 监听窗口创建事件
            webview.once("tauri://created", function () {
                console.log("关于窗口创建成功");
            });

            webview.once("tauri://error", function (e: any) {
                console.error("关于窗口创建失败:", e);
                alert("无法打开关于窗口，请检查应用配置。");
            });

            // 监听窗口关闭事件
            webview.once("tauri://close-requested", function () {
                console.log("关于窗口将要关闭");
            });
        }
    } catch (error) {
        console.error("打开关于窗口失败:", error);
        alert("关于窗口打开失败: " + error);
    }
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
