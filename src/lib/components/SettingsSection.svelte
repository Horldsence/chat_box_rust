<script lang="ts">
  export let title: string;
  export let description: string = "";
  export let icon: string = "";
</script>

<section class="settings-section">
  <div class="section-header">
    <div class="flex items-center space-x-3 mb-2">
      {#if icon}
        <span class="text-2xl">{icon}</span>
      {/if}
      <h2 class="text-xl font-semibold text-gray-900 dark:text-gray-50">{title}</h2>
    </div>
    {#if description}
      <p class="text-gray-600 dark:text-gray-400 mb-6">{description}</p>
    {/if}
  </div>

  <div class="section-content space-y-6">
    <slot />
  </div>
</section>

<style>
  .settings-section {
    @apply mb-8 last:mb-0;
  }

  .section-header {
    @apply border-b border-gray-200 dark:border-gray-700 pb-4 mb-6;
  }

  .section-content {
    @apply space-y-4;
  }
</style>
