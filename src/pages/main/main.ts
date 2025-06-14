import { createApp } from "vue";
import ElementPlus from "element-plus";
import "element-plus/dist/index.css";
import App from "../../App.vue";

// 创建Vue应用实例
const app = createApp(App);

// 使用Element Plus
app.use(ElementPlus);

// 挂载应用
app.mount("#app");

// 监听初始化进度事件
import { listen } from "@tauri-apps/api/event";

interface InitProgress {
  current_step: string;
  step_index: number;
  total_steps: number;
  progress_percent: number;
  elapsed_time: number;
  estimated_remaining?: number;
}

// 监听初始化进度
listen<InitProgress>("init_progress", (event) => {
  const progress = event.payload;
  console.log(
    `初始化进度: ${progress.progress_percent.toFixed(1)}% - ${progress.current_step}`,
  );

  // 更新加载文本
  const loadingElement = document.querySelector(".loading");
  if (loadingElement) {
    loadingElement.textContent = `${progress.current_step} (${progress.progress_percent.toFixed(0)}%)`;
  }
}).catch((err) => {
  console.warn("无法监听初始化进度事件:", err);
});

// 应用启动后的初始化检查
document.addEventListener("DOMContentLoaded", () => {
  console.log("Chat Box 应用已启动");

  // 检查是否为开发环境
  if (import.meta.env.DEV) {
    console.log("开发模式已启用");
  }

  // 移除加载指示器
  setTimeout(() => {
    const loadingElement = document.querySelector(".loading") as HTMLElement;
    if (loadingElement) {
      loadingElement.style.display = "none";
    }
  }, 2000);
});

// 全局错误处理
window.addEventListener("error", (event) => {
  console.error("全局错误:", event.error);
});

window.addEventListener("unhandledrejection", (event) => {
  console.error("未处理的Promise拒绝:", event.reason);
});
