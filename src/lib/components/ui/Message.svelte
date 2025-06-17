<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { Message as MessageType } from "$lib/types";

  // Props
  export let message: MessageType;
  export let showAvatar = true;
  export let showTime = true;
  export let showCopyButton = true;
  export let isTyping = false;
  export let compact = false;

  // Event dispatcher
  const dispatch = createEventDispatcher();

  // Internal state
  let copied = false;
  let copyTimeout: ReturnType<typeof setTimeout>;

  // Format timestamp
  function formatTime(timestamp: number | string): string {
    const date = new Date(timestamp);
    const now = new Date();
    const diffInHours = (now.getTime() - date.getTime()) / (1000 * 60 * 60);

    if (diffInHours < 24) {
      // Show time for today
      return date.toLocaleTimeString("zh-CN", {
        hour: "2-digit",
        minute: "2-digit",
        hour12: false,
      });
    } else if (diffInHours < 24 * 7) {
      // Show day and time for this week
      return date.toLocaleDateString("zh-CN", {
        weekday: "short",
        hour: "2-digit",
        minute: "2-digit",
        hour12: false,
      });
    } else {
      // Show full date for older messages
      return date.toLocaleDateString("zh-CN", {
        year: "numeric",
        month: "short",
        day: "numeric",
        hour: "2-digit",
        minute: "2-digit",
        hour12: false,
      });
    }
  }

  // Copy message content
  async function copyMessage() {
    try {
      await navigator.clipboard.writeText(message.content);
      copied = true;
      clearTimeout(copyTimeout);
      copyTimeout = setTimeout(() => {
        copied = false;
      }, 2000);
      dispatch("copy", { message });
    } catch (error) {
      console.error("Failed to copy message:", error);
      dispatch("copyError", { message, error });
    }
  }

  // Handle message click
  function handleMessageClick() {
    dispatch("click", { message });
  }

  // Handle message double click
  function handleMessageDoubleClick() {
    if (showCopyButton) {
      copyMessage();
    }
    dispatch("doubleclick", { message });
  }

  // Get avatar emoji/icon
  function getAvatar(sender: string): string {
    return sender === "user" ? "👤" : "🤖";
  }

  // Get message classes
  $: messageClasses = [
    "message",
    `message-${message.sender}`,
    compact && "message-compact",
    isTyping && "message-typing",
  ]
    .filter(Boolean)
    .join(" ");

  $: contentClasses = ["message-content", "message-bubble", `message-${message.sender}`]
    .filter(Boolean)
    .join(" ");
</script>

<div
  class={messageClasses}
  role="article"
  aria-label={`${message.sender === "user" ? "用户" : "AI助手"}的消息`}
  on:click={handleMessageClick}
  on:dblclick={handleMessageDoubleClick}
  on:keydown={(e) => e.key === "Enter" && handleMessageClick()}
  tabindex="0"
>
  <!-- Avatar -->
  {#if showAvatar}
    <div
      class="message-avatar"
      class:user={message.sender === "user"}
      class:assistant={message.sender === "assistant"}
      aria-hidden="true"
    >
      {getAvatar(message.sender)}
    </div>
  {/if}

  <!-- Content Wrapper -->
  <div class="message-content-wrapper">
    <!-- Message Content -->
    <div class={contentClasses}>
      {#if isTyping}
        <!-- Typing indicator -->
        <div class="typing-indicator">
          <span class="typing-text">正在输入</span>
          <div class="typing-dots">
            <div class="typing-dot"></div>
            <div class="typing-dot"></div>
            <div class="typing-dot"></div>
          </div>
        </div>
      {:else}
        <!-- Message text -->
        <div class="message-text">
          {@html message.content.replace(/\n/g, "<br>")}
        </div>
      {/if}

      <!-- Copy button -->
      {#if showCopyButton && !isTyping}
        <button
          class="message-copy-btn"
          class:copied
          on:click|stopPropagation={copyMessage}
          title={copied ? "已复制!" : "复制消息"}
          aria-label={copied ? "已复制到剪贴板" : "复制消息到剪贴板"}
        >
          {#if copied}
            ✓
          {:else}
            📋
          {/if}
        </button>
      {/if}
    </div>

    <!-- Timestamp and metadata -->
    {#if showTime && !isTyping}
      <div class="message-meta">
        <time
          class="message-time"
          datetime={new Date(message.timestamp).toISOString()}
          title={new Date(message.timestamp).toLocaleString("zh-CN")}
        >
          {formatTime(message.timestamp)}
        </time>

        <!-- Message status indicators could go here -->
        <slot name="status" />
      </div>
    {/if}
  </div>
</div>

<style>
  .message {
    display: flex;
    gap: var(--spacing-sm, 0.5rem);
    align-items: flex-start;
    max-width: 80%;
    margin-bottom: var(--spacing-md, 1rem);
    animation: messageSlideIn 0.3s ease-out;
    cursor: pointer;
    transition: all 0.2s ease-in-out;
  }

  .message:hover {
    transform: translateY(-1px);
  }

  .message:focus {
    outline: 2px solid var(--color-primary, #667eea);
    outline-offset: 2px;
    border-radius: var(--radius-md, 8px);
  }

  .message-user {
    align-self: flex-end;
    flex-direction: row-reverse;
  }

  .message-assistant {
    align-self: flex-start;
  }

  .message-compact {
    margin-bottom: var(--spacing-sm, 0.5rem);
  }

  .message-compact .message-avatar {
    width: 32px;
    height: 32px;
    font-size: 1rem;
  }

  .message-avatar {
    width: 40px;
    height: 40px;
    border-radius: var(--radius-full, 50%);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.2rem;
    flex-shrink: 0;
    background: var(--color-bg-overlay, rgba(255, 255, 255, 0.1));
    backdrop-filter: blur(8px);
    transition: all 0.2s ease-in-out;
  }

  .message-avatar.user {
    background: var(--color-primary, #667eea);
    color: white;
  }

  .message-avatar.assistant {
    background: var(--color-secondary, #4fd1c7);
    color: white;
  }

  .message-content-wrapper {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs, 0.25rem);
    min-width: 0;
    flex: 1;
  }

  .message-content {
    position: relative;
    line-height: 1.5;
    word-wrap: break-word;
    overflow-wrap: break-word;
    hyphens: auto;
    transition: all 0.2s ease-in-out;
  }

  .message-content:hover .message-copy-btn {
    opacity: 1;
  }

  .message-bubble {
    padding: var(--spacing-sm, 0.5rem) var(--spacing-md, 1rem);
    border-radius: var(--radius-lg, 12px);
    box-shadow: var(--shadow-sm, 0 1px 3px rgba(0, 0, 0, 0.1));
    backdrop-filter: blur(8px);
  }

  .message-bubble.message-user {
    background: var(--color-primary, #667eea);
    color: var(--color-text-inverse, white);
    border-bottom-right-radius: var(--radius-sm, 4px);
  }

  .message-bubble.message-assistant {
    background: var(--color-bg-overlay, rgba(255, 255, 255, 0.95));
    color: var(--color-text-primary, #2d3748);
    border: 1px solid var(--color-border-light, rgba(0, 0, 0, 0.1));
    border-bottom-left-radius: var(--radius-sm, 4px);
  }

  .message-text {
    line-height: 1.6;
    white-space: pre-wrap;
  }

  .message-copy-btn {
    position: absolute;
    top: var(--spacing-xs, 0.25rem);
    right: var(--spacing-xs, 0.25rem);
    width: 24px;
    height: 24px;
    border: none;
    background: rgba(0, 0, 0, 0.1);
    color: inherit;
    border-radius: var(--radius-sm, 4px);
    cursor: pointer;
    opacity: 0;
    transition: all 0.2s ease-in-out;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.75rem;
    backdrop-filter: blur(4px);
  }

  .message-copy-btn:hover {
    background: rgba(0, 0, 0, 0.2);
    transform: scale(1.1);
  }

  .message-copy-btn.copied {
    background: var(--color-success, #48bb78);
    color: white;
    opacity: 1;
  }

  .message-meta {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm, 0.5rem);
    font-size: 0.75rem;
    color: var(--color-text-muted, rgba(255, 255, 255, 0.7));
    margin-top: var(--spacing-xs, 0.25rem);
  }

  .message-user .message-meta {
    justify-content: flex-end;
    text-align: right;
  }

  .message-assistant .message-meta {
    justify-content: flex-start;
    text-align: left;
    color: var(--color-text-muted, #718096);
  }

  .message-time {
    font-variant-numeric: tabular-nums;
  }

  /* Typing indicator */
  .typing-indicator {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm, 0.5rem);
    color: var(--color-text-secondary, #4a5568);
    font-style: italic;
  }

  .typing-text {
    font-size: 0.9rem;
  }

  .typing-dots {
    display: flex;
    gap: var(--spacing-xs, 0.25rem);
  }

  .typing-dot {
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--color-text-muted, #718096);
    animation: typingDots 1.4s infinite ease-in-out;
  }

  .typing-dot:nth-child(1) {
    animation-delay: -0.32s;
  }
  .typing-dot:nth-child(2) {
    animation-delay: -0.16s;
  }
  .typing-dot:nth-child(3) {
    animation-delay: 0s;
  }

  /* Animations */
  @keyframes messageSlideIn {
    from {
      opacity: 0;
      transform: translateY(20px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  @keyframes typingDots {
    0%,
    80%,
    100% {
      transform: scale(0);
      opacity: 0.5;
    }
    40% {
      transform: scale(1);
      opacity: 1;
    }
  }

  /* Dark theme adjustments */
  :global([data-theme="dark"]) .message-bubble.message-assistant {
    background: var(--color-bg-secondary, rgba(45, 55, 72, 0.95));
    border-color: var(--color-border-light, rgba(255, 255, 255, 0.1));
    color: var(--color-text-primary, #f7fafc);
  }

  :global([data-theme="dark"]) .message-avatar {
    background: var(--color-bg-overlay, rgba(0, 0, 0, 0.2));
  }

  :global([data-theme="dark"]) .typing-indicator {
    color: var(--color-text-secondary, #e2e8f0);
  }

  /* High contrast mode */
  @media (prefers-contrast: high) {
    .message-bubble {
      border-width: 2px;
    }

    .message-copy-btn {
      background: rgba(0, 0, 0, 0.3);
    }
  }

  /* Reduced motion */
  @media (prefers-reduced-motion: reduce) {
    .message {
      animation: none;
    }

    .message:hover {
      transform: none;
    }

    .typing-dot {
      animation: none;
    }

    .message-content,
    .message-avatar,
    .message-copy-btn {
      transition: none;
    }
  }

  /* Print styles */
  @media print {
    .message {
      break-inside: avoid;
      margin-bottom: 0.5rem;
    }

    .message-copy-btn {
      display: none;
    }

    .message-bubble {
      background: white !important;
      color: black !important;
      border: 1px solid black !important;
      box-shadow: none !important;
    }

    .message-avatar {
      background: white !important;
      color: black !important;
      border: 1px solid black !important;
    }
  }

  /* Mobile optimizations */
  @media (max-width: 768px) {
    .message {
      max-width: 90%;
    }

    .message-compact .message-avatar {
      width: 28px;
      height: 28px;
      font-size: 0.9rem;
    }

    .message-bubble {
      padding: var(--spacing-xs, 0.25rem) var(--spacing-sm, 0.5rem);
    }

    .message-copy-btn {
      width: 20px;
      height: 20px;
      font-size: 0.7rem;
    }
  }

  /* Accessibility improvements */
  .message:focus-visible {
    outline: 2px solid var(--color-primary, #667eea);
    outline-offset: 2px;
  }

  .message-copy-btn:focus-visible {
    outline: 2px solid var(--color-primary, #667eea);
    outline-offset: 1px;
  }

  /* Selection styles */
  .message-text::selection {
    background: rgba(102, 126, 234, 0.3);
    color: inherit;
  }

  :global([data-theme="dark"]) .message-text::selection {
    background: rgba(102, 126, 234, 0.5);
  }
</style>
