import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// DOM 元素
const messagesArea = document.getElementById("messages-area") as HTMLElement;
const messageInput = document.getElementById(
  "message-input",
) as HTMLTextAreaElement;
const sendBtn = document.getElementById("send-btn") as HTMLButtonElement;
const voiceBtn = document.getElementById("voice-btn") as HTMLButtonElement;
const clearBtn = document.getElementById("clear-btn") as HTMLButtonElement;
const exportBtn = document.getElementById("export-btn") as HTMLButtonElement;
const closeBtn = document.getElementById("close-btn") as HTMLButtonElement;
const typingIndicator = document.getElementById(
  "typing-indicator",
) as HTMLElement;

// 状态
let isRecording = false;
let isTyping = false;
let currentConversationId: string | null = null;

// 初始化
const init = async () => {
  await loadOrCreateConversation();
  setupEventListeners();
  setupMessageListener();
  adjustTextareaHeight();
};

// 加载或创建对话
const loadOrCreateConversation = async () => {
  try {
    // 尝试获取当前活动对话
    const conversations = await invoke("get_conversations");
    if (Array.isArray(conversations) && conversations.length > 0) {
      currentConversationId = conversations[0].id;
      await loadMessages();
    } else {
      // 创建新对话
      const newConv = await invoke("create_conversation", {
        title: "聊天窗口对话",
      });
      currentConversationId = (newConv as any).id;
    }
  } catch (error) {
    console.error("加载对话失败:", error);
  }
};

// 加载消息
const loadMessages = async () => {
  if (!currentConversationId) return;

  try {
    const messages = await invoke("get_conversation_messages", {
      conversationId: currentConversationId,
    });

    // 清空欢迎消息
    const welcomeMsg = messagesArea.querySelector(".welcome-message");
    if (welcomeMsg && Array.isArray(messages) && messages.length > 0) {
      welcomeMsg.remove();
    }

    // 显示消息
    if (Array.isArray(messages)) {
      messages.forEach((message: any) => {
        addMessage(
          message.content,
          message.sender,
          new Date(message.timestamp),
        );
      });
    }

    scrollToBottom();
  } catch (error) {
    console.error("加载消息失败:", error);
  }
};

// 设置事件监听
const setupEventListeners = () => {
  // 发送按钮
  sendBtn.addEventListener("click", sendMessage);

  // 输入框事件
  messageInput.addEventListener("keydown", (e) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  });

  messageInput.addEventListener("input", adjustTextareaHeight);

  // 语音按钮
  voiceBtn.addEventListener("click", toggleVoiceRecording);

  // 控制按钮
  clearBtn.addEventListener("click", clearConversation);
  exportBtn.addEventListener("click", exportConversation);
  closeBtn.addEventListener("click", closeWindow);

  // 快捷操作
  document.querySelectorAll(".quick-action").forEach((btn) => {
    btn.addEventListener("click", (e) => {
      const text = (e.target as HTMLElement).dataset.text;
      if (text) {
        messageInput.value = text;
        sendMessage();
      }
    });
  });
};

// 设置消息监听器
const setupMessageListener = async () => {
  try {
    await listen("chat_response", (event: any) => {
      const response = event.payload;
      hideTypingIndicator();
      addMessage(response.content, "assistant");
      scrollToBottom();
    });
  } catch (error) {
    console.error("设置消息监听器失败:", error);
  }
};

// 发送消息
const sendMessage = async () => {
  const message = messageInput.value.trim();
  if (!message || isTyping) return;

  // 清空输入框
  messageInput.value = "";
  adjustTextareaHeight();

  // 隐藏欢迎消息
  const welcomeMsg = messagesArea.querySelector(".welcome-message");
  if (welcomeMsg) {
    welcomeMsg.remove();
  }

  // 添加用户消息
  addMessage(message, "user");
  scrollToBottom();

  // 显示打字指示器
  showTypingIndicator();

  try {
    // 发送到后端
    await invoke("send_message", {
      conversationId: currentConversationId,
      content: message,
      sender: "user",
    });
  } catch (error) {
    console.error("发送消息失败:", error);
    hideTypingIndicator();
    addMessage("抱歉，发送消息时出现错误，请稍后重试。", "assistant");
  }
};

// 添加消息到界面
const addMessage = (content: string, sender: string, timestamp?: Date) => {
  const messageDiv = document.createElement("div");
  messageDiv.className = `message ${sender}`;

  const now = timestamp || new Date();
  const timeStr = now.toLocaleTimeString("zh-CN", {
    hour12: false,
    hour: "2-digit",
    minute: "2-digit",
  });

  messageDiv.innerHTML = `
    <div class="message-avatar">${sender === "user" ? "👤" : "🤖"}</div>
    <div class="message-content">${content}</div>
    <div class="message-time">${timeStr}</div>
  `;

  messagesArea.appendChild(messageDiv);
  scrollToBottom();
};

// 显示打字指示器
const showTypingIndicator = () => {
  isTyping = true;
  typingIndicator.classList.add("show");
};

// 隐藏打字指示器
const hideTypingIndicator = () => {
  isTyping = false;
  typingIndicator.classList.remove("show");
};

// 调整文本框高度
const adjustTextareaHeight = () => {
  messageInput.style.height = "auto";
  messageInput.style.height = messageInput.scrollHeight + "px";
};

// 滚动到底部
const scrollToBottom = () => {
  setTimeout(() => {
    messagesArea.scrollTop = messagesArea.scrollHeight;
  }, 100);
};

// 切换语音录制
const toggleVoiceRecording = () => {
  if (isRecording) {
    // 停止录制
    isRecording = false;
    voiceBtn.classList.remove("recording");
    voiceBtn.textContent = "🎤";

    // 这里可以添加语音识别逻辑
    console.log("停止语音录制");
  } else {
    // 开始录制
    isRecording = true;
    voiceBtn.classList.add("recording");
    voiceBtn.textContent = "⏹️";

    // 这里可以添加语音录制逻辑
    console.log("开始语音录制");
  }
};

// 清空对话
const clearConversation = async () => {
  if (!currentConversationId) return;

  try {
    const confirmed = confirm("确定要清空当前对话吗？此操作不可撤销。");
    if (!confirmed) return;

    await invoke("clear_conversation", {
      conversationId: currentConversationId,
    });

    // 清空界面
    messagesArea.innerHTML = `
      <div class="welcome-message">
        <h2>👋 欢迎使用 Chat Box</h2>
        <p>我是您的 AI 助手，很高兴为您服务！</p>
        <p>您可以问我任何问题，或者尝试以下快捷操作：</p>
        <div class="quick-actions">
          <button class="quick-action" data-text="解释一下人工智能的发展历史">AI历史</button>
          <button class="quick-action" data-text="推荐一些学习编程的方法">编程学习</button>
          <button class="quick-action" data-text="介绍一下最新的技术趋势">技术趋势</button>
          <button class="quick-action" data-text="帮我写一个简单的Python程序">代码示例</button>
        </div>
      </div>
    `;

    // 重新绑定快捷操作事件
    setupEventListeners();
  } catch (error) {
    console.error("清空对话失败:", error);
  }
};

// 导出对话
const exportConversation = async () => {
  if (!currentConversationId) return;

  try {
    const messages = await invoke("get_conversation_messages", {
      conversationId: currentConversationId,
    });

    if (!Array.isArray(messages) || messages.length === 0) {
      alert("没有可导出的消息");
      return;
    }

    let exportText = "=== Chat Box 对话记录 ===\n\n";
    messages.forEach((msg: any) => {
      const time = new Date(msg.timestamp).toLocaleString("zh-CN");
      const sender = msg.sender === "user" ? "用户" : "AI助手";
      exportText += `[${time}] ${sender}:\n${msg.content}\n\n`;
    });

    // 创建下载链接
    const blob = new Blob([exportText], { type: "text/plain;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `chat-export-${new Date().toISOString().slice(0, 10)}.txt`;
    a.click();
    URL.revokeObjectURL(url);
  } catch (error) {
    console.error("导出对话失败:", error);
  }
};

// 关闭窗口
const closeWindow = async () => {
  try {
    const webview = WebviewWindow.getCurrent();
    await webview.close();
  } catch (error) {
    console.error("关闭窗口失败:", error);
  }
};

// 键盘快捷键
document.addEventListener("keydown", (e) => {
  if (e.key === "Escape") {
    closeWindow();
  }
  if (e.ctrlKey && e.key === "l") {
    e.preventDefault();
    clearConversation();
  }
  if (e.ctrlKey && e.key === "e") {
    e.preventDefault();
    exportConversation();
  }
  if (e.ctrlKey && e.key === "Enter") {
    e.preventDefault();
    sendMessage();
  }
});

// 页面加载完成后初始化
document.addEventListener("DOMContentLoaded", init);
