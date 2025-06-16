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
    @apply flex items-start justify-between p-4 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 transition-all duration-200;
  }

  .settings-item:hover {
    @apply border-gray-300 dark:border-gray-600 shadow-sm;
  }

  .settings-item.disabled {
    @apply opacity-50 cursor-not-allowed;
  }

  .item-info {
    @apply flex-1 mr-4;
  }

  .item-title {
    @apply text-base font-medium text-gray-900 dark:text-gray-50 mb-1;
  }

  .item-description {
    @apply text-sm text-gray-600 dark:text-gray-400 leading-relaxed;
  }

  .item-control {
    @apply flex-shrink-0 flex items-center;
  }

  /* 响应式调整 */
  @media (max-width: 640px) {
    .settings-item {
      @apply flex-col items-start space-y-3;
    }

    .item-info {
      @apply mr-0 w-full;
    }

    .item-control {
      @apply w-full justify-start;
    }
  }
</style>
