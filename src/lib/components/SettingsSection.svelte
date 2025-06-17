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
    margin-bottom: 2rem;
  }

  .settings-section:last-child {
    margin-bottom: 0;
  }

  .section-header {
    border-bottom: 1px solid #e5e7eb;
    padding-bottom: 1rem;
    margin-bottom: 1.5rem;
  }

  :global(.dark) .section-header {
    border-bottom-color: #374151;
  }

  .section-content {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
</style>
