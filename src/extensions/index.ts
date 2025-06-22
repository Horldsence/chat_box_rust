// 扩展系统类型定义和扩展注册入口

import React from "react";

/**
 * 扩展元信息
 */
export interface ExtensionMeta {
  id: string; // 唯一ID
  name: string; // 展示名称
  description?: string;
  version: string;
  author?: string;
  enabledByDefault?: boolean;
  icon?: React.ReactNode;
  tags?: string[];
}

/**
 * 扩展生命周期钩子
 */
export interface ExtensionLifecycle {
  onLoad?: () => void | Promise<void>;
  onUnload?: () => void | Promise<void>;
  onActivate?: () => void | Promise<void>;
  onDeactivate?: () => void | Promise<void>;
}

/**
 * 扩展主接口
 */
export interface Extension extends ExtensionLifecycle {
  meta: ExtensionMeta;
  // 可选：扩展可以暴露一个React组件用于UI集成
  Component?: React.ComponentType<any>;
  // 可选：扩展可以暴露API
  api?: Record<string, any>;
}

/**
 * 扩展注册表
 */
const extensionRegistry: Record<string, Extension> = {};

// 自动注册内置扩展
import Live2DExtension from "./live2d";
extensionRegistry[Live2DExtension.meta.id] = Live2DExtension;

/**
 * 注册扩展
 */
export function registerExtension(ext: Extension) {
  if (!ext.meta?.id) throw new Error("Extension must have a unique id");
  extensionRegistry[ext.meta.id] = ext;
}

/**
 * 获取所有已注册扩展
 */
export function getAllExtensions(): Extension[] {
  return Object.values(extensionRegistry);
}

/**
 * 获取指定扩展
 */
export function getExtension(id: string): Extension | undefined {
  return extensionRegistry[id];
}

/**
 * 启用扩展
 */
export async function activateExtension(id: string) {
  const ext = extensionRegistry[id];
  if (ext?.onActivate) await ext.onActivate();
}

/**
 * 禁用扩展
 */
export async function deactivateExtension(id: string) {
  const ext = extensionRegistry[id];
  if (ext?.onDeactivate) await ext.onDeactivate();
}

// 默认导出注册表
export { extensionRegistry };
