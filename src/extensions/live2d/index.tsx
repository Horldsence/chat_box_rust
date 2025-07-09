import Live2DRender from "../../components/Live2DRender";
import type { Extension } from "../index";

// Live2D 扩展元信息
const meta = {
  id: "live2d",
  name: "Live2D 虚拟形象",
  description: "为聊天界面提供 Live2D 虚拟形象渲染与交互能力。",
  version: "1.0.0",
  author: "系统开发者",
  enabledByDefault: true,
  tags: ["avatar", "virtual", "live2d", "visual"],
  icon: (
    <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
      <circle cx="10" cy="10" r="10" fill="#60A5FA"/>
      <ellipse cx="10" cy="13" rx="5" ry="3" fill="#fff"/>
      <ellipse cx="7.5" cy="9" rx="1.2" ry="2" fill="#222"/>
      <ellipse cx="12.5" cy="9" rx="1.2" ry="2" fill="#222"/>
    </svg>
  ),
};

// 可选：扩展API（可暴露给主程序调用）
const api = {
  // 例如：触发Live2D动作
  triggerMotion: (groupName: string, motionIndex?: number) => {
    // 这里可以通过事件/全局状态/消息机制与Live2DRender通信
    // 这里只是示例
    console.log(`[Live2D扩展] 触发动作: ${groupName}, 索引: ${motionIndex}`);
  },
  setExpression: (expressionName: string) => {
    console.log(`[Live2D扩展] 设置表情: ${expressionName}`);
  },
};

// 生命周期钩子
const onLoad = () => {
  console.log("[Live2D扩展] 已加载");
};
const onUnload = () => {
  console.log("[Live2D扩展] 已卸载");
};

// 扩展主对象
const Live2DExtension: Extension = {
  meta,
  Component: Live2DRender,
  api,
  onLoad,
  onUnload,
};

export default Live2DExtension;
