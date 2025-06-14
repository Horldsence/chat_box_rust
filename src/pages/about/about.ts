import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { invoke } from '@tauri-apps/api/core';

// DOM 元素
const closeBtn = document.getElementById('close-btn') as HTMLButtonElement;
const versionElement = document.getElementById('version') as HTMLElement;
const buildDateElement = document.getElementById('build-date') as HTMLElement;
const githubBtn = document.getElementById('github-btn') as HTMLButtonElement;
const feedbackBtn = document.getElementById('feedback-btn') as HTMLButtonElement;
const checkUpdateBtn = document.getElementById('check-update-btn') as HTMLButtonElement;

// 应用信息
const appInfo = {
  name: 'Chat Box',
  version: '1.0.0',
  buildDate: new Date().toLocaleDateString('zh-CN'),
  author: 'Chat Box Team',
  description: '一个现代化的 AI 聊天助手应用',
  githubUrl: 'https://github.com/your-repo/chat-box-rust',
  website: 'https://chatbox.example.com'
};

// 初始化
const init = async () => {
  await loadAppInfo();
  setupEventListeners();
  startParticleAnimation();
};

// 加载应用信息
const loadAppInfo = async () => {
  try {
    // 尝试从后端获取应用信息
    const info = await invoke('get_app_info');
    if (info) {
      Object.assign(appInfo, info);
    }
  } catch (error) {
    console.log('使用默认应用信息:', error);
  }

  // 更新界面显示
  if (versionElement) {
    versionElement.textContent = `v${appInfo.version}`;
  }
  if (buildDateElement) {
    buildDateElement.textContent = appInfo.buildDate;
  }
};

// 设置事件监听器
const setupEventListeners = () => {
  // 关闭按钮
  if (closeBtn) {
    closeBtn.addEventListener('click', closeWindow);
  }

  // GitHub 按钮
  if (githubBtn) {
    githubBtn.addEventListener('click', () => {
      openUrl(appInfo.githubUrl);
    });
  }

  // 反馈按钮
  if (feedbackBtn) {
    feedbackBtn.addEventListener('click', () => {
      openUrl(`${appInfo.githubUrl}/issues`);
    });
  }

  // 检查更新按钮
  if (checkUpdateBtn) {
    checkUpdateBtn.addEventListener('click', checkForUpdates);
  }

  // 键盘快捷键
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
      closeWindow();
    }
    if (e.ctrlKey && e.key === 'u') {
      e.preventDefault();
      checkForUpdates();
    }
  });

  // Logo 点击彩蛋
  const logo = document.querySelector('.logo');
  if (logo) {
    let clickCount = 0;
    logo.addEventListener('click', () => {
      clickCount++;
      if (clickCount >= 5) {
        showEasterEgg();
        clickCount = 0;
      }
    });
  }
};

// 打开 URL
const openUrl = async (url: string) => {
  try {
    await invoke('open_url', { url });
  } catch (error) {
    console.error('打开链接失败:', error);
    // 备用方案：复制到剪贴板
    try {
      await navigator.clipboard.writeText(url);
      showMessage('链接已复制到剪贴板', 'info');
    } catch (clipboardError) {
      console.error('复制失败:', clipboardError);
    }
  }
};

// 检查更新
const checkForUpdates = async () => {
  if (!checkUpdateBtn) return;

  const originalText = checkUpdateBtn.textContent;
  checkUpdateBtn.textContent = '检查中...';
  checkUpdateBtn.disabled = true;

  try {
    const updateInfo = await invoke('check_for_updates');

    if (updateInfo && (updateInfo as any).hasUpdate) {
      const update = updateInfo as any;
      showMessage(`发现新版本 ${update.version}！`, 'success');

      // 显示更新对话框
      const shouldUpdate = confirm(
        `发现新版本 ${update.version}\n\n更新内容：\n${update.changelog}\n\n是否立即下载更新？`
      );

      if (shouldUpdate) {
        await openUrl(update.downloadUrl);
      }
    } else {
      showMessage('当前已是最新版本', 'success');
    }
  } catch (error) {
    console.error('检查更新失败:', error);
    showMessage('检查更新失败，请稍后重试', 'error');
  } finally {
    checkUpdateBtn.textContent = originalText;
    checkUpdateBtn.disabled = false;
  }
};

// 显示消息
const showMessage = (message: string, type: 'success' | 'error' | 'info' = 'info') => {
  // 移除现有消息
  const existingMessage = document.querySelector('.message-toast');
  if (existingMessage) {
    existingMessage.remove();
  }

  // 创建消息元素
  const messageDiv = document.createElement('div');
  messageDiv.className = `message-toast ${type}`;
  messageDiv.textContent = message;

  // 设置样式
  messageDiv.style.cssText = `
    position: fixed;
    top: 20px;
    right: 20px;
    padding: 12px 20px;
    border-radius: 8px;
    color: white;
    font-size: 14px;
    z-index: 1000;
    animation: slideInRight 0.3s ease-out;
    max-width: 300px;
    word-wrap: break-word;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  `;

  // 根据类型设置背景色
  const colors = {
    success: '#48bb78',
    error: '#f56565',
    info: '#4299e1'
  };
  messageDiv.style.background = colors[type];

  // 添加动画样式
  if (!document.querySelector('#message-animation-style')) {
    const style = document.createElement('style');
    style.id = 'message-animation-style';
    style.textContent = `
      @keyframes slideInRight {
        from {
          transform: translateX(100%);
          opacity: 0;
        }
        to {
          transform: translateX(0);
          opacity: 1;
        }
      }
    `;
    document.head.appendChild(style);
  }

  document.body.appendChild(messageDiv);

  // 3秒后自动移除
  setTimeout(() => {
    if (messageDiv.parentNode) {
      messageDiv.style.animation = 'slideInRight 0.3s ease-out reverse';
      setTimeout(() => {
        messageDiv.remove();
      }, 300);
    }
  }, 3000);
};

// 显示彩蛋
const showEasterEgg = () => {
  const messages = [
    '🎉 你发现了一个彩蛋！',
    '💎 感谢你对 Chat Box 的支持！',
    '🚀 我们会继续努力改进产品！',
    '🌟 你真是个细心的用户！',
    '🎊 Keep exploring and stay curious!'
  ];

  const randomMessage = messages[Math.floor(Math.random() * messages.length)];
  showMessage(randomMessage, 'success');

  // 添加特效
  createConfetti();
};

// 创建彩色纸屑效果
const createConfetti = () => {
  const colors = ['#667eea', '#764ba2', '#f093fb', '#f5576c', '#4facfe', '#00f2fe'];

  for (let i = 0; i < 50; i++) {
    const confetti = document.createElement('div');
    confetti.style.cssText = `
      position: fixed;
      width: 10px;
      height: 10px;
      background: ${colors[Math.floor(Math.random() * colors.length)]};
      left: ${Math.random() * 100}%;
      top: -10px;
      z-index: 9999;
      animation: confettiFall ${2 + Math.random() * 3}s linear forwards;
      transform: rotate(${Math.random() * 360}deg);
    `;

    document.body.appendChild(confetti);

    // 清理
    setTimeout(() => {
      confetti.remove();
    }, 5000);
  }

  // 添加下落动画
  if (!document.querySelector('#confetti-animation-style')) {
    const style = document.createElement('style');
    style.id = 'confetti-animation-style';
    style.textContent = `
      @keyframes confettiFall {
        to {
          transform: translateY(100vh) rotate(720deg);
          opacity: 0;
        }
      }
    `;
    document.head.appendChild(style);
  }
};

// 粒子动画
const startParticleAnimation = () => {
  const canvas = document.getElementById('particle-canvas') as HTMLCanvasElement;
  if (!canvas) return;

  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  // 设置画布大小
  const resizeCanvas = () => {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
  };

  resizeCanvas();
  window.addEventListener('resize', resizeCanvas);

  // 粒子类
  class Particle {
    x: number;
    y: number;
    vx: number;
    vy: number;
    radius: number;
    opacity: number;

    constructor() {
      this.x = Math.random() * canvas.width;
      this.y = Math.random() * canvas.height;
      this.vx = (Math.random() - 0.5) * 0.5;
      this.vy = (Math.random() - 0.5) * 0.5;
      this.radius = Math.random() * 2 + 1;
      this.opacity = Math.random() * 0.5 + 0.2;
    }

    update() {
      this.x += this.vx;
      this.y += this.vy;

      if (this.x < 0 || this.x > canvas.width) this.vx *= -1;
      if (this.y < 0 || this.y > canvas.height) this.vy *= -1;
    }

    draw() {
      if (!ctx) return;
      ctx.beginPath();
      ctx.arc(this.x, this.y, this.radius, 0, Math.PI * 2);
      ctx.fillStyle = `rgba(102, 126, 234, ${this.opacity})`;
      ctx.fill();
    }
  }

  // 创建粒子
  const particles: Particle[] = [];
  for (let i = 0; i < 50; i++) {
    particles.push(new Particle());
  }

  // 动画循环
  const animate = () => {
    if (!ctx) return;

    ctx.clearRect(0, 0, canvas.width, canvas.height);

    particles.forEach(particle => {
      particle.update();
      particle.draw();
    });

    requestAnimationFrame(animate);
  };

  animate();
};

// 关闭窗口
const closeWindow = async () => {
  try {
    const webview = WebviewWindow.getCurrent();
    await webview.close();
  } catch (error) {
    console.error('关闭窗口失败:', error);
  }
};

// 页面加载完成后初始化
document.addEventListener('DOMContentLoaded', init);
