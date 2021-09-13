<script>
  import { GEOBANNED, PREFERRED_LANGUAGE } from '../store';
  import { dictionary } from '../scripts/localization';
</script>

<div class="view-container flex align-center justify-center column">
  <h2 class="view-header">
    {dictionary[$PREFERRED_LANGUAGE].copilot.alert.failed}
  </h2>
  {#if $GEOBANNED}
    <span>
      {dictionary[$PREFERRED_LANGUAGE].cockpit.geobanned}
    </span>
  {:else}
    <span>
      {dictionary[$PREFERRED_LANGUAGE].cockpit.noMarket}
    </span>
  {/if}
</div>

<style>
  .view-container {
    position: absolute !important;
    top: 0 !important;
    left: 0 !important;
    width: 100vw !important;
    height: 100vh !important;
    padding: unset !important;
  }
  .view-header {
    color: var(--failure);
    font-size: 30px;
  }
</style>