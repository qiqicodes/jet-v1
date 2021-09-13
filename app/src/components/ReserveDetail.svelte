<script lang="ts">
  import type { Reserve } from '../models/JetTypes';
  import { COPILOT, CURRENT_RESERVE, PREFERRED_LANGUAGE } from '../store';
  import { currencyFormatter, } from '../scripts/utils';
  import { dictionary, definitions } from '../scripts/localization';
  import Button from './Button.svelte';
  import PercentageChart from './PercentageChart.svelte';

  export let reserveDetail: Reserve;
  export let updateValues: Function;
  export let closeReserveDetail: Function;
</script>

{#if reserveDetail}
  <div class="modal-bg flex align-center justify-center"
    on:click={() => closeReserveDetail()}>
  </div>
  <div class="reserve-detail modal flex align-center justify-center column">
    <div class="modal-section flex align-center justify-center column">
      <div class="flex align-center-justify-center">
        <img src={`img/cryptos/${reserveDetail.abbrev}.png`} 
          alt={`${reserveDetail.abbrev} Logo`}
        />
        <h1 class="modal-header">
          {reserveDetail.name.toUpperCase()}
        </h1>
      </div>
      <span>
        1 {reserveDetail.abbrev} ≈ {currencyFormatter(reserveDetail.price, true, 2)}
      </span>
    </div>
    <div class="divider">
    </div>
    <div class="modal-section flex align-center justify-center column">
      <span class="flex align-center justify-center">
        {dictionary[$PREFERRED_LANGUAGE].reserveDetail.reserveSize.toUpperCase()}
      </span>
      <h2 class="modal-subheader text-gradient">
        {currencyFormatter(reserveDetail.marketSize.muln(reserveDetail.price).uiAmountFloat, true, 2)}
      </h2>
    </div>
    <div class="divider">
    </div>
    <div class="modal-section flex align-center justify-evenly">
      <PercentageChart percentage={reserveDetail.utilizationRate} 
        text={dictionary[$PREFERRED_LANGUAGE].reserveDetail.utilisationRate.toUpperCase()} 
      />
      <div class="flex align-start justify-center column">
        <div class="flex align-start justify-center" style="margin: var(--spacing-sm);">
          <div class="asset-info-color"
            style="background: var(--failure); box-shadow: var(--neu-shadow-inset-failure);">
          </div>
          <span style="text-align: start;">
            {dictionary[$PREFERRED_LANGUAGE].reserveDetail.totalBorrowed.toUpperCase()}
            <br>
            <p>
              {currencyFormatter(reserveDetail.outstandingDebt.uiAmountFloat, false, 2)}
            </p>
          </span>
        </div>
        <div class="flex align-start justify-center" style="margin: var(--spacing-sm);">
          <div class="asset-info-color"
            style="background: var(--success); box-shadow: var(--neu-shadow-inset-success);">
          </div>
          <span style="text-align: start;">
            {dictionary[$PREFERRED_LANGUAGE].reserveDetail.availableLiquidity.toUpperCase()}
            <br>
            <p>
              {currencyFormatter(reserveDetail.availableLiquidity.uiAmountFloat, false, 2)}
            </p>
          </span>
        </div>
      </div>
    </div>
    <div class="divider">
    </div>
    <div class="modal-section flex align-center justify-center">
      <div class="modal-detail flex align-center justify-center column">
        <span>
          {dictionary[$PREFERRED_LANGUAGE].reserveDetail.maximumLTV.toUpperCase()}
          <sup class="far fa-question-circle"
            on:click={() => COPILOT.set({
              definition: definitions[$PREFERRED_LANGUAGE].maximumLtv
            })}>
          </sup>
        </span>
        <p>
          {reserveDetail.maximumLTV / 100}%
        </p>
      </div>
      <div class="modal-detail flex align-center justify-center column">
        <span>
          {dictionary[$PREFERRED_LANGUAGE].reserveDetail.liquidationPremium.toUpperCase()}
          <sup on:click={() => COPILOT.set({
            definition: definitions[$PREFERRED_LANGUAGE].liquidationPremium
          })} 
            class="far fa-question-circle">
          </sup>
        </span>
        <p>
          {reserveDetail.liquidationPremium / 100}%
        </p>
      </div>
    </div>
    <div class="divider">
    </div>
    <div class="modal-section flex align-center justify-center">
      <Button text={`${dictionary[$PREFERRED_LANGUAGE].reserveDetail.tradeAsset.replace('{{ASSET}}', reserveDetail.abbrev)}`} 
        onClick={() => {
          closeReserveDetail();
          CURRENT_RESERVE.set(reserveDetail);
          updateValues();
        }} 
      />
    </div>
    <i on:click={() => closeReserveDetail()} class="jet-icons close">
      ✕
    </i>
  </div>
{/if}

<style>
  .modal-bg {
    z-index: 100;
  }
  .modal {
    padding: var(--spacing-lg) var(--spacing-sm);
  }
  .reserve-detail {
    z-index: 101;
  }
  .asset-info-color {
    width: 10px;
    height: 12px;
    margin: 2.5px var(--spacing-sm);
  }
  img {
    width: 40px;
    height: 40px;
    padding: 0 var(--spacing-sm);
  }
  sup {
    position: absolute;
    top: 2px;
    right: -13px;
  }

  @media screen and (max-width: 1000px) {
    .modal, .reserve-detail {
      width: 100%;
      max-width: unset;
      height: calc((var(--vh, 1vh) * 95) - var(--mobile-nav-height));
      position: fixed;
      display: block;
      top: 0;
      left: 50%;
      transform: translate(-50%, 0);
      box-shadow: unset;
      border-radius: unset;
      background: var(--white);
      overflow-y: scroll;
    }
    .asset-info-color {
      width: 6px;
      height: 6px;
      margin: var(--spacing-md) var(--spacing-sm);
    }
    img {
      width: 30px;
      height: 30px;
    }
  }
</style>