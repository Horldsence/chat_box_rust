<script lang="ts">
  export let title: string;
  export let description: string = "";
  export let icon: string = "";
  export let required: boolean = false;
  export let disabled: boolean = false;
</script>

<div class="settings-item" class:disabled>
  <div class="item-info">
    <div class="flex items-center space-x-2">
      {#if icon}
        <span class="text-lg">{icon}</span>
      {/if}
      <h3 class="item-title">
        {title}
        {#if required}
          <span class="text-red-500 ml-1">*</span>
        {/if}
      </h3>
    </div>
    {#if description}
      <p class="item-description">{description}</p>
    {/if}
  </div>

  <div class="item-control">
    <slot />
  </div>
</div>

<style>
  .settings-item {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    padding: 1rem;
    background-color: white;
    border-radius: 0.5rem;
    border: 1px solid #e5e7eb;
    transition: all 0.2s;
  }

  :global(.dark) .settings-item {
    background-color: #1f2937;
    border-color: #374151;
  }

  .settings-item:hover {
    border-color: #d1d5db;
    box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
  }

  :global(.dark) .settings-item:hover {
    border-color: #4b5563;
  }

  .settings-item.disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .item-info {
    flex: 1;
    margin-right: 1rem;
  }

  .item-title {
    font-size: 1rem;
    font-weight: 500;
    color: #111827;
    margin-bottom: 0.25rem;
  }

  :global(.dark) .item-title {
    color: #f9fafb;
  }

  .item-description {
    font-size: 0.875rem;
    color: #6b7280;
    line-height: 1.625;
  }

  :global(.dark) .item-description {
    color: #9ca3af;
  }

  .item-control {
    flex-shrink: 0;
    display: flex;
    align-items: center;
  }

  /* 响应式调整 */
  @media (max-width: 640px) {
    .settings-item {
      flex-direction: column;
      align-items: flex-start;
      gap: 0.75rem;
    }

    .item-info {
      margin-right: 0;
      width: 100%;
    }

    .item-control {
      width: 100%;
      justify-content: flex-start;
    }
  }
</style>
