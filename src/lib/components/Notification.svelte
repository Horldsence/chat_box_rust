<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { writable } from 'svelte/store';
	import { fade, fly } from 'svelte/transition';
	import type { Notification, NotificationType } from '$lib/types';
	import { errorService } from '$lib/services/ErrorService';

	// 通知存储
	const notifications = writable<Notification[]>([]);

	// 自动递增的ID计数器
	let notificationIdCounter = 0;

	// 生成唯一ID
	function generateNotificationId(): string {
		return `notification_${Date.now()}_${++notificationIdCounter}`;
	}

	// 添加通知
	function addNotification(notificationData: Omit<Notification, 'id' | 'timestamp'>) {
		const notification: Notification = {
			id: generateNotificationId(),
			timestamp: Date.now(),
			duration: notificationData.duration ?? getDefaultDuration(notificationData.type),
			...notificationData
		};

		notifications.update(items => [...items, notification]);

		// 如果有持续时间，自动移除
		if (notification.duration && notification.duration > 0) {
			setTimeout(() => {
				removeNotification(notification.id);
			}, notification.duration);
		}

		return notification.id;
	}

	// 移除通知
	function removeNotification(id: string) {
		notifications.update(items => items.filter(item => item.id !== id));
	}

	// 清空所有通知
	function clearAll() {
		notifications.set([]);
	}

	// 获取默认持续时间
	function getDefaultDuration(type: NotificationType): number {
		switch (type) {
			case 'error':
				return 8000;
			case 'warning':
				return 6000;
			case 'success':
				return 4000;
			case 'info':
			default:
				return 5000;
		}
	}

	// 获取通知图标
	function getNotificationIcon(type: NotificationType): string {
		switch (type) {
			case 'success':
				return '✓';
			case 'error':
				return '✕';
			case 'warning':
				return '⚠';
			case 'info':
			default:
				return 'ℹ';
		}
	}

	// 获取通知样式类
	function getNotificationClass(type: NotificationType): string {
		return `notification notification-${type}`;
	}

	// 处理动作点击
	function handleActionClick(action: () => void, notificationId: string) {
		try {
			action();
		} catch (error) {
			console.error('Notification action failed:', error);
		} finally {
			removeNotification(notificationId);
		}
	}

	// 键盘事件处理
	function handleKeydown(event: KeyboardEvent, notificationId: string) {
		if (event.key === 'Escape') {
			removeNotification(notificationId);
		}
	}

	// 生命周期
	onMount(() => {
		// 设置错误服务的通知处理器
		errorService.setNotificationHandler(addNotification);
	});

	onDestroy(() => {
		// 清理
		clearAll();
	});

	// 公开接口给其他组件使用
	export { addNotification, removeNotification, clearAll };
</script>

<!-- 通知容器 -->
<div class="notifications-container" role="region" aria-label="通知消息">
	{#each $notifications as notification (notification.id)}
		<div
			class={getNotificationClass(notification.type)}
			role="alert"
			aria-live="polite"
			tabindex="-1"
			in:fly={{ y: -30, duration: 300 }}
			out:fade={{ duration: 200 }}
			on:keydown={(e) => handleKeydown(e, notification.id)}
		>
			<!-- 通知图标 -->
			<div class="notification-icon" aria-hidden="true">
				{getNotificationIcon(notification.type)}
			</div>

			<!-- 通知内容 -->
			<div class="notification-content">
				{#if notification.title}
					<div class="notification-title">{notification.title}</div>
				{/if}
				<div class="notification-message">{notification.message}</div>

				<!-- 操作按钮 -->
				{#if notification.actions && notification.actions.length > 0}
					<div class="notification-actions">
						{#each notification.actions as action}
							<button
								class="notification-action btn-{action.type}"
								on:click={() => handleActionClick(action.action, notification.id)}
							>
								{action.label}
							</button>
						{/each}
					</div>
				{/if}
			</div>

			<!-- 关闭按钮 -->
			<button
				class="notification-close"
				on:click={() => removeNotification(notification.id)}
				aria-label="关闭通知"
				title="关闭通知"
			>
				×
			</button>

			<!-- 进度条（如果有持续时间） -->
			{#if notification.duration && notification.duration > 0}
				<div class="notification-progress">
					<div
						class="notification-progress-bar"
						style="animation-duration: {notification.duration}ms"
					></div>
				</div>
			{/if}
		</div>
	{/each}

	<!-- 清空所有按钮（当有多个通知时显示） -->
	{#if $notifications.length > 1}
		<div class="notifications-clear-all" transition:fade={{ duration: 200 }}>
			<button class="btn-ghost btn-sm" on:click={clearAll}>
				清空所有通知
			</button>
		</div>
	{/if}
</div>

<style>
	.notifications-container {
		position: fixed;
		top: var(--spacing-lg);
		right: var(--spacing-lg);
		z-index: 9999;
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
		max-width: 400px;
		width: 100%;
		pointer-events: none;
	}

	.notification {
		position: relative;
		display: flex;
		align-items: flex-start;
		gap: var(--spacing-sm);
		padding: var(--spacing-md);
		border-radius: var(--radius-lg);
		box-shadow: var(--shadow-xl);
		backdrop-filter: var(--blur-md);
		border: 1px solid transparent;
		pointer-events: auto;
		animation: slideIn 0.3s ease-out;
		overflow: hidden;
	}

	.notification-success {
		background: rgba(72, 187, 120, 0.95);
		color: white;
		border-color: rgba(72, 187, 120, 0.3);
	}

	.notification-error {
		background: rgba(245, 101, 101, 0.95);
		color: white;
		border-color: rgba(245, 101, 101, 0.3);
	}

	.notification-warning {
		background: rgba(237, 137, 54, 0.95);
		color: white;
		border-color: rgba(237, 137, 54, 0.3);
	}

	.notification-info {
		background: rgba(102, 126, 234, 0.95);
		color: white;
		border-color: rgba(102, 126, 234, 0.3);
	}

	.notification-icon {
		flex-shrink: 0;
		width: 24px;
		height: 24px;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 1.2rem;
		font-weight: bold;
		border-radius: var(--radius-full);
		background: rgba(255, 255, 255, 0.2);
	}

	.notification-content {
		flex: 1;
		min-width: 0;
	}

	.notification-title {
		font-weight: 600;
		font-size: 1rem;
		margin-bottom: var(--spacing-xs);
		line-height: 1.3;
	}

	.notification-message {
		font-size: 0.9rem;
		line-height: 1.4;
		opacity: 0.95;
		word-wrap: break-word;
	}

	.notification-actions {
		display: flex;
		gap: var(--spacing-xs);
		margin-top: var(--spacing-sm);
	}

	.notification-action {
		padding: var(--spacing-xs) var(--spacing-sm);
		border: none;
		border-radius: var(--radius-sm);
		font-size: 0.8rem;
		font-weight: 500;
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.notification-action.btn-primary {
		background: rgba(255, 255, 255, 0.9);
		color: var(--color-text-primary);
	}

	.notification-action.btn-primary:hover {
		background: white;
		transform: translateY(-1px);
	}

	.notification-action.btn-secondary {
		background: rgba(255, 255, 255, 0.2);
		color: white;
		border: 1px solid rgba(255, 255, 255, 0.3);
	}

	.notification-action.btn-secondary:hover {
		background: rgba(255, 255, 255, 0.3);
	}

	.notification-close {
		flex-shrink: 0;
		width: 24px;
		height: 24px;
		border: none;
		background: rgba(255, 255, 255, 0.2);
		color: white;
		border-radius: var(--radius-full);
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 1.2rem;
		line-height: 1;
		transition: all var(--transition-fast);
	}

	.notification-close:hover {
		background: rgba(255, 255, 255, 0.3);
		transform: scale(1.1);
	}

	.notification-progress {
		position: absolute;
		bottom: 0;
		left: 0;
		right: 0;
		height: 3px;
		background: rgba(255, 255, 255, 0.2);
		overflow: hidden;
	}

	.notification-progress-bar {
		height: 100%;
		background: rgba(255, 255, 255, 0.8);
		animation: progressBar linear forwards;
		transform-origin: left;
	}

	.notifications-clear-all {
		display: flex;
		justify-content: center;
		margin-top: var(--spacing-sm);
	}

	.notifications-clear-all button {
		font-size: 0.8rem;
		opacity: 0.8;
		transition: opacity var(--transition-fast);
	}

	.notifications-clear-all button:hover {
		opacity: 1;
	}

	/* 动画 */
	@keyframes slideIn {
		from {
			transform: translateX(100%);
			opacity: 0;
		}
		to {
			transform: translateX(0);
			opacity: 1;
		}
	}

	@keyframes progressBar {
		from {
			transform: scaleX(1);
		}
		to {
			transform: scaleX(0);
		}
	}

	/* 响应式设计 */
	@media (max-width: 768px) {
		.notifications-container {
			top: var(--spacing-sm);
			right: var(--spacing-sm);
			left: var(--spacing-sm);
			max-width: none;
		}

		.notification {
			padding: var(--spacing-sm);
		}

		.notification-title {
			font-size: 0.9rem;
		}

		.notification-message {
			font-size: 0.8rem;
		}
	}

	/* 暗色主题适配 */
	:global([data-theme="dark"]) .notification-success {
		background: rgba(72, 187, 120, 0.9);
	}

	:global([data-theme="dark"]) .notification-error {
		background: rgba(245, 101, 101, 0.9);
	}

	:global([data-theme="dark"]) .notification-warning {
		background: rgba(237, 137, 54, 0.9);
	}

	:global([data-theme="dark"]) .notification-info {
		background: rgba(102, 126, 234, 0.9);
	}

	/* 高对比度模式 */
	@media (prefers-contrast: high) {
		.notification {
			border-width: 2px;
			border-style: solid;
		}

		.notification-icon {
			background: rgba(255, 255, 255, 0.4);
		}
	}

	/* 减少动画模式 */
	@media (prefers-reduced-motion: reduce) {
		.notification {
			animation: none;
		}

		.notification-progress-bar {
			animation: none;
		}
	}

	/* 打印样式 */
	@media print {
		.notifications-container {
			display: none;
		}
	}

	/* 焦点样式 */
	.notification:focus {
		outline: 2px solid rgba(255, 255, 255, 0.8);
		outline-offset: 2px;
	}

	.notification-close:focus {
		outline: 2px solid rgba(255, 255, 255, 0.8);
		outline-offset: 1px;
	}

	.notification-action:focus {
		outline: 2px solid rgba(255, 255, 255, 0.8);
		outline-offset: 1px;
	}
</style>
