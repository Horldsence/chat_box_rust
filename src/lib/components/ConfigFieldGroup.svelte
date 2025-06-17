<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import SettingsItem from "./SettingsItem.svelte";

  export let title: string;
  export let description: string = "";
  export let icon: string = "";
  export let collapsible: boolean = false;
  export let collapsed: boolean = false;
  export let disabled: boolean = false;
  export let required: boolean = false;
  export let error: string = "";
  export let warning: string = "";
  export let info: string = "";

  const dispatch = createEventDispatcher<{
    toggle: { collapsed: boolean };
    expand: void;
    collapse: void;
  }>();

  function handleToggle() {
    if (!collapsible) return;

    collapsed = !collapsed;
    dispatch("toggle", { collapsed });

    if (collapsed) {
      dispatch("collapse");
    } else {
      dispatch("expand");
    }
  }

  $: hasMessage = error || warning || info;
  $: messageType = error ? "error" : warning ? "warning" : "info";
  $: messageText = error || warning || info;
</script>

<div class="config-field-group" class:disabled class:has-error={error}>
  <!-- Group Header -->
  {#if collapsible}
    <button
      type="button"
      class="group-header clickable"
      on:click={handleToggle}
      on:keydown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          handleToggle();
        }
      }}
      aria-expanded={!collapsed}
      aria-controls="group-content-{title}"
    >
      <div class="header-content">
        <div class="title-section">
          {#if icon}
            <span class="icon" aria-hidden="true">{icon}</span>
          {/if}

          <div class="title-text">
            <h3 class="title">
              {title}
              {#if required}
                <span class="required-indicator" aria-label="必填" title="必填">*</span>
              {/if}
            </h3>

            {#if description}
              <p class="description">{description}</p>
            {/if}
          </div>
        </div>

        <div class="collapse-toggle">
          <svg
            class="collapse-icon"
            class:rotated={!collapsed}
            width="16"
            height="16"
            viewBox="0 0 16 16"
            fill="none"
            aria-hidden="true"
          >
            <path
              d="M6 4l4 4-4 4"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </div>
      </div>

      <!-- Status Messages -->
      {#if hasMessage}
        <div class="message {messageType}" role="alert">
          <svg
            class="message-icon"
            width="16"
            height="16"
            viewBox="0 0 16 16"
            fill="none"
            aria-hidden="true"
          >
            {#if messageType === "error"}
              <circle cx="8" cy="8" r="8" fill="currentColor" />
              <path d="M6 6l4 4M10 6l-4 4" stroke="white" stroke-width="2" stroke-linecap="round" />
            {:else if messageType === "warning"}
              <path d="M8 1l7 14H1L8 1z" fill="currentColor" />
              <path d="M8 6v4M8 12h.01" stroke="white" stroke-width="2" stroke-linecap="round" />
            {:else}
              <circle cx="8" cy="8" r="8" fill="currentColor" />
              <path
                d="M6 8l2 2 4-4"
                stroke="white"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            {/if}
          </svg>
          <span class="message-text">{messageText}</span>
        </div>
      {/if}
    </button>
  {:else}
    <div class="group-header">
      <div class="header-content">
        <div class="title-section">
          {#if icon}
            <span class="icon" aria-hidden="true">{icon}</span>
          {/if}

          <div class="title-text">
            <h3 class="title">
              {title}
              {#if required}
                <span class="required-indicator" aria-label="必填" title="必填">*</span>
              {/if}
            </h3>

            {#if description}
              <p class="description">{description}</p>
            {/if}
          </div>
        </div>
      </div>

      <!-- Status Messages -->
      {#if hasMessage}
        <div class="message {messageType}" role="alert">
          <svg
            class="message-icon"
            width="16"
            height="16"
            viewBox="0 0 16 16"
            fill="none"
            aria-hidden="true"
          >
            {#if messageType === "error"}
              <circle cx="8" cy="8" r="8" fill="currentColor" />
              <path d="M6 6l4 4M10 6l-4 4" stroke="white" stroke-width="2" stroke-linecap="round" />
            {:else if messageType === "warning"}
              <path d="M8 1l7 14H1L8 1z" fill="currentColor" />
              <path d="M8 6v4M8 12h.01" stroke="white" stroke-width="2" stroke-linecap="round" />
            {:else}
              <circle cx="8" cy="8" r="8" fill="currentColor" />
              <path
                d="M6 8l2 2 4-4"
                stroke="white"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            {/if}
          </svg>
          <span class="message-text">{messageText}</span>
        </div>
      {/if}
    </div>
  {/if}

  <!-- Group Content -->
  {#if !collapsed}
    <div id="group-content-{title}" class="group-content" class:disabled>
      <slot />
    </div>
  {/if}
</div>

<style>
  .config-field-group {
    margin-bottom: 1.5rem;
    background-color: white;
    border-radius: 0.5rem;
    border: 1px solid #e5e7eb;
    transition: all 0.2s;
  }

  :global(.dark) .config-field-group {
    background-color: #1f2937;
    border-color: #374151;
  }

  .config-field-group:hover {
    border-color: #d1d5db;
  }

  :global(.dark) .config-field-group:hover {
    border-color: #4b5563;
  }

  .config-field-group.disabled {
    opacity: 0.5;
    pointer-events: none;
  }

  .config-field-group.has-error {
    border-color: #fca5a5;
  }

  :global(.dark) .config-field-group.has-error {
    border-color: #dc2626;
  }

  .group-header {
    padding: 1rem;
    border-bottom: 1px solid #f3f4f6;
  }

  :global(.dark) .group-header {
    border-bottom-color: #374151;
  }

  .group-header.clickable {
    cursor: pointer;
    user-select: none;
    background: none;
    border: none;
    width: 100%;
    text-align: left;
    font: inherit;
  }

  .group-header.clickable:hover {
    background-color: #f9fafb;
  }

  :global(.dark) .group-header.clickable:hover {
    background-color: #111827;
  }

  .group-header.clickable:focus {
    outline: none;
    box-shadow: 0 0 0 2px #3b82f6;
    opacity: 0.5;
  }

  :global(.dark) .group-header.clickable:focus {
    box-shadow: 0 0 0 2px #60a5fa;
  }

  .header-content {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
  }

  .title-section {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    flex: 1;
  }

  .icon {
    font-size: 1.25rem;
    margin-top: 0.25rem;
    flex-shrink: 0;
  }

  .title-text {
    flex: 1;
    min-width: 0;
  }

  .title {
    font-size: 1.125rem;
    font-weight: 600;
    color: #111827;
    margin-bottom: 0.25rem;
    display: flex;
    align-items: center;
  }

  :global(.dark) .title {
    color: #f9fafb;
  }

  .required-indicator {
    color: #ef4444;
    margin-left: 0.25rem;
    font-size: 0.875rem;
  }

  .description {
    font-size: 0.875rem;
    color: #6b7280;
    line-height: 1.625;
  }

  :global(.dark) .description {
    color: #9ca3af;
  }

  .collapse-toggle {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2rem;
    height: 2rem;
    border-radius: 50%;
    transition: background-color 0.15s;
    flex-shrink: 0;
    margin-left: 0.5rem;
  }

  .collapse-toggle:hover {
    background-color: #f3f4f6;
  }

  :global(.dark) .collapse-toggle:hover {
    background-color: #374151;
  }

  .collapse-toggle:focus {
    outline: none;
    box-shadow: 0 0 0 2px #3b82f6;
    opacity: 0.5;
  }

  :global(.dark) .collapse-toggle:focus {
    box-shadow: 0 0 0 2px #60a5fa;
  }

  .collapse-icon {
    color: #6b7280;
    transition: transform 0.2s;
  }

  :global(.dark) .collapse-icon {
    color: #9ca3af;
  }

  .collapse-icon.rotated {
    transform: rotate(90deg);
  }

  .message {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-top: 0.75rem;
    padding: 0.5rem;
    border-radius: 0.375rem;
    font-size: 0.875rem;
  }

  .message.error {
    background-color: #fef2f2;
    color: #b91c1c;
  }

  :global(.dark) .message.error {
    background-color: rgba(185, 28, 28, 0.2);
    color: #fca5a5;
  }

  .message.warning {
    background-color: #fffbeb;
    color: #d97706;
  }

  :global(.dark) .message.warning {
    background-color: rgba(217, 119, 6, 0.2);
    color: #fcd34d;
  }

  .message.info {
    background-color: #eff6ff;
    color: #1d4ed8;
  }

  :global(.dark) .message.info {
    background-color: rgba(29, 78, 216, 0.2);
    color: #93c5fd;
  }

  .message-icon {
    flex-shrink: 0;
  }

  .message-text {
    flex: 1;
  }

  .group-content {
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .group-content.disabled {
    opacity: 0.5;
  }

  /* 动画效果 */
  .group-content {
    animation: slideDown 0.2s ease-out;
  }

  @keyframes slideDown {
    from {
      opacity: 0;
      transform: translateY(-8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  /* 响应式调整 */
  @media (max-width: 640px) {
    .header-content {
      flex-direction: column;
      gap: 0.75rem;
    }

    .title-section {
      width: 100%;
    }

    .collapse-toggle {
      align-self: flex-start;
    }

    .group-header {
      padding: 0.75rem;
    }

    .group-content {
      padding: 0.75rem;
    }
  }

  /* 暗色主题特殊处理 */
  @media (prefers-color-scheme: dark) {
    .config-field-group {
      box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
    }
  }

  /* 高对比度模式 */
  @media (prefers-contrast: high) {
    .config-field-group {
      border-width: 2px;
    }

    .message {
      border: 1px solid currentColor;
    }
  }

  /* 减少动画模式 */
  @media (prefers-reduced-motion: reduce) {
    .collapse-icon,
    .config-field-group,
    .group-content {
      transition: none;
    }

    @keyframes slideDown {
      from,
      to {
        opacity: 1;
        transform: translateY(0);
      }
    }
  }
</style>
