<svelte:head>
  <title>Jet Protocol | {dictionary[$USER.preferredLanguage].cockpit.title}</title>
</svelte:head>
<script lang="ts">
  import { Datatable, rows } from 'svelte-simple-datatables';
  import RangeSlider from "svelte-range-slider-pips";
  import { NATIVE_MINT } from '@solana/spl-token';
  import type { Reserve, Obligation } from '../models/JetTypes';
  import { INIT_FAILED, MARKET, USER, COPILOT } from '../store';
  import { inDevelopment, airdrop, deposit, withdraw, borrow, repay, addTransactionLog } from '../scripts/jet';
  import { currencyFormatter, totalAbbrev, addNotification, getObligationData, TokenAmount, Amount } from '../scripts/util';
  import { generateCopilotSuggestion } from '../scripts/copilot';
  import { dictionary, definitions } from '../scripts/localization'; 
  import Loader from '../components/Loader.svelte';
  import ReserveDetail from '../components/ReserveDetail.svelte';
  import Toggle from '../components/Toggle.svelte';
  import InitFailed from '../components/InitFailed.svelte';
  import ConnectWalletButton from '../components/ConnectWalletButton.svelte';

  let walletBalances: Record<string, TokenAmount> = {};
  let collateralBalances: Record<string, number> = {};
  let loanBalances: Record<string, number> = {};
  let maxBorrowAmounts: Record<string, number> = {};
  let maxWithdrawAmounts: Record<string, number> = {};
  let assetsAreCurrentDeposit: Record<string, boolean> = {};
  let assetsAreCurrentBorrow: Record<string, boolean> = {};
  let obligation: Obligation;
  let adjustedRatio: number;
  let belowMinCRatio: boolean = false;
  let noDeposits: boolean = true;
  let inputAmount: number | null = null;
  let maxInputValue: number;
  let inputError: string;
  let disabledInput: boolean = true;
  let disabledMessage: string = '';
  let reserveDetail: Reserve | null = null;
  let sendingTrade: boolean = false;

  // Datatable settings
  let tableData: Reserve[] = [];
  const tableSettings: any = {
    sortable: false,
    pagination: false,
    scrollY: false,
    blocks: {
      searchInput: true
    },
    labels: {
        search: dictionary[$USER.preferredLanguage].cockpit.search,    
    }
  };

  // If in development, can request airdrop for testing
  const doAirdrop = async (reserve: Reserve): Promise<void> => {
    let amount = TokenAmount.tokens("100", reserve.decimals);
    if(reserve.tokenMintPubkey.equals(NATIVE_MINT)) {
      amount = TokenAmount.tokens("1", reserve.decimals);
    }

    const [ok, txid] = await airdrop(reserve.abbrev, amount.amount);
    if (ok && txid) {
      COPILOT.set({
        alert: {
          good: true,
          header: dictionary[$USER.preferredLanguage].copilot.alert.success,
          text:  dictionary[$USER.preferredLanguage].copilot.alert.airdropSuccess
            .replaceAll('{{UI AMOUNT}}', amount.uiAmount)
            .replaceAll('{{RESERVE ABBREV}}', reserve.abbrev)
        }
      });
    } else if (!ok && !txid) {
      COPILOT.set({
        alert: {
          good: false,
          header: dictionary[$USER.preferredLanguage].copilot.alert.failed,
          text: dictionary[$USER.preferredLanguage].cockpit.txFailed
        }
      });
    }
  };

  // Change current reserve
  const changeReserve = (reserve: Reserve): void => {
    if (sendingTrade) {
      return;
    }

    inputError = '';
    inputAmount = null;
    USER.update(user => {
      user.currentReserve = reserve;
      return user;
    });

    updateValues();
  };

  // Check if user input should be disabled
  // depending on wallet balance and position
  const checkDisabledInput = (): void => {
    if (!$USER.assets || !$USER.currentReserve) {
      return;
    }

    disabledMessage = '';
    disabledInput = false;
    if ($USER.tradeAction === 'deposit' && (walletBalances[$USER.currentReserve.abbrev]?.amount.isZero() || assetsAreCurrentBorrow[$USER.currentReserve.abbrev])) {
      disabledInput = true;
      if (walletBalances[$USER.currentReserve.abbrev]?.amount.isZero()) {
        disabledMessage = dictionary[$USER.preferredLanguage].cockpit.noBalanceForDeposit
          .replaceAll('{{ASSET}}', $USER.currentReserve.abbrev);
      } else if (assetsAreCurrentBorrow[$USER.currentReserve.abbrev]) {
        disabledMessage = dictionary[$USER.preferredLanguage].cockpit.assetIsCurrentBorrow
          .replaceAll('{{ASSET}}', $USER.currentReserve.abbrev);
      }
    } else if ($USER.tradeAction === 'withdraw' && (!collateralBalances[$USER.currentReserve.abbrev] || belowMinCRatio)) {
      disabledInput = true;
      if (!collateralBalances[$USER.currentReserve.abbrev]) {
        disabledMessage = disabledMessage = dictionary[$USER.preferredLanguage].cockpit.noDepositsForWithdraw
          .replaceAll('{{ASSET}}', $USER.currentReserve.abbrev);
      } else {
        disabledMessage = disabledMessage = dictionary[$USER.preferredLanguage].cockpit.belowMinCRatio;
      }
    } else if ($USER.tradeAction === 'borrow' && (noDeposits || belowMinCRatio || assetsAreCurrentDeposit[$USER.currentReserve.abbrev] || !$USER.currentReserve.availableLiquidity?.uiAmountFloat)) {
      disabledInput = true;
      if (noDeposits) {
        disabledMessage = disabledMessage = dictionary[$USER.preferredLanguage].cockpit.noDepositsForBorrow;
      } else if (belowMinCRatio) {
        disabledMessage = disabledMessage = dictionary[$USER.preferredLanguage].cockpit.belowMinCRatio;
      } else if (assetsAreCurrentDeposit[$USER.currentReserve.abbrev]) {
        disabledMessage = disabledMessage = dictionary[$USER.preferredLanguage].cockpit.assetIsCurrentDeposit
          .replaceAll('{{ASSET}}', $USER.currentReserve.abbrev);
      }
    } else if ($USER.tradeAction === 'repay' && !loanBalances[$USER.currentReserve.abbrev]) {
      disabledInput = true;
      disabledMessage = disabledMessage = dictionary[$USER.preferredLanguage].cockpit.noDebtForRepay
          .replaceAll('{{ASSET}}', $USER.currentReserve.abbrev);
    }

    return;
  };

  // Get the maximum value a user can input
  // depending on wallet balance and position
  const getMaxInputValue = (): void => {
    if (!$USER.assets || !$USER.currentReserve) {
      return;
    }

    if ($USER.tradeAction === 'deposit') {
      maxInputValue = walletBalances[$USER.currentReserve.abbrev]?.uiAmountFloat;
    } else if ($USER.tradeAction === 'withdraw') {
      maxInputValue = maxWithdrawAmounts[$USER.currentReserve.abbrev];
    } else if ($USER.tradeAction === 'borrow') {
      maxInputValue = maxBorrowAmounts[$USER.currentReserve.abbrev];
    } else {
      maxInputValue =  loanBalances[$USER.currentReserve.abbrev]
    }

    return;
  };

  // Adjust user input and calculate updated c-ratio if 
  // they were to submit current trade
  const adjustCollateralizationRatio = (): void => {
    if (!$USER.currentReserve || !$USER.assets || inputAmount === null) {
      return;
    }

    // If input is negative, reset to zero
    if (inputAmount < 0) {
      inputAmount = 0;
    }
    
    if ($USER.tradeAction === 'deposit') {
      adjustedRatio = (obligation.depositedValue + inputAmount * $USER.currentReserve.price) / (
          obligation.borrowedValue > 0
            ? obligation.borrowedValue
              : 1
        );
    } else if ($USER.tradeAction === 'withdraw') {
      adjustedRatio = (obligation.depositedValue - inputAmount * $USER.currentReserve.price) / (
          obligation.borrowedValue > 0 
            ? obligation.borrowedValue
              : 1
        );
    } else if ($USER.tradeAction === 'borrow') {
      adjustedRatio = obligation.depositedValue / (
        (obligation.borrowedValue + inputAmount * $USER.currentReserve.price) > 0
            ? (obligation.borrowedValue + ((inputAmount ?? 0) * $USER.currentReserve.price))
              : 1
        );
    } else if ($USER.tradeAction === 'repay') {
      adjustedRatio = obligation.depositedValue / (
        (obligation.borrowedValue - inputAmount * $USER.currentReserve.price) > 0
            ? (obligation.borrowedValue - inputAmount * $USER.currentReserve.price)
             : 1
      );
    }
  };

  // Update input and adjusted ratio on slider change
  const sliderUpdate = (e: any) => {
    inputAmount = maxInputValue * (e.detail.value / 100);
    adjustCollateralizationRatio();
  };

  // Update all market/user data
  const updateValues = (): void => {
    tableData = [];
    obligation = getObligationData();
    for (let r in $MARKET.reserves) {
      if ($MARKET.reserves[r]) {
        tableData.push($MARKET.reserves[r]);
      }

      // Position balances
      collateralBalances[r] = $USER.assets?.tokens[r]?.collateralBalance?.uiAmountFloat ?? 0;
      loanBalances[r] = $USER.assets?.tokens[r]?.loanBalance?.uiAmountFloat ?? 0;

      // Deposit data
      if ($USER.assets) {
        walletBalances[r] = $USER.assets.tokens[r]?.tokenMintPubkey.equals(NATIVE_MINT) 
          ? $USER.assets.sol
          : $USER.assets.tokens[r]?.walletTokenBalance;
      }
      
      // Withdraw data
      maxWithdrawAmounts[r] = obligation?.borrowedValue
        ? (obligation.depositedValue - ($MARKET.minColRatio * obligation.borrowedValue)) / $MARKET.reserves[r].price
          : collateralBalances[r];
      if (maxWithdrawAmounts[r] > collateralBalances[r]) {
        maxWithdrawAmounts[r] = collateralBalances[r];
      }

      // Borrow data
      belowMinCRatio = obligation.depositedValue / obligation.borrowedValue <= $MARKET.minColRatio;
      noDeposits = !obligation.depositedValue;
      assetsAreCurrentDeposit[r] = collateralBalances[r] > 0;
      assetsAreCurrentBorrow[r] = loanBalances[r] > 0;
      maxBorrowAmounts[r] = ((obligation.depositedValue / $MARKET.minColRatio) - obligation.borrowedValue) / $MARKET.reserves[r].price;
      if (maxBorrowAmounts[r] > $MARKET.reserves[r].availableLiquidity?.uiAmountFloat) {
        maxBorrowAmounts[r] = $MARKET.reserves[r].availableLiquidity?.uiAmountFloat;
      }
    }

    // Set adjusted ratio to current ratio
    if (!adjustedRatio && obligation?.colRatio) {
      adjustedRatio = obligation.colRatio;
    }

    // Check if user's current position shouldn't allow trades
    checkDisabledInput();

    // Get max input value for current trade scenario
    getMaxInputValue();
  };

  // Check scenario and submit trade
  const checkSubmit = () => {
    if (disabledInput) {
      return;
    }

    // If depositing all SOL, inform user about insufficient lamports and reject 
    if ($USER.currentReserve?.abbrev === 'SOL' && inputAmount && inputAmount <= maxInputValue && $USER.tradeAction === 'deposit'
      && (walletBalances[$USER.currentReserve.abbrev]?.uiAmountFloat - 0.02) <= inputAmount) {
      COPILOT.set({
        suggestion: {
          good: false,
          detail: dictionary[$USER.preferredLanguage].cockpit.insufficientLamports
        }
      });
    // If trade would result in c-ratio below min ratio, inform user and reject
  } else if ((obligation?.borrowedValue || $USER.tradeAction === 'borrow') 
      && adjustedRatio < $MARKET.minColRatio) {
      if (adjustedRatio < obligation?.colRatio) {
        COPILOT.set({
          suggestion: {
          good: false,
          detail: dictionary[$USER.preferredLanguage].cockpit.rejectTrade
            .replaceAll('{{NEW-C-RATIO}}', currencyFormatter(adjustedRatio * 100, false, 1))
            .replaceAll('{{JET MIN C-RATIO}}', $MARKET.minColRatio * 100)
          }
        });
      } else {
        // If this trade still results in undercollateralization, inform user
        COPILOT.set({
          suggestion: {
            good: false,
            detail: dictionary[$USER.preferredLanguage].cockpit.stillUndercollateralized
              .replaceAll('{{NEW-C-RATIO}}', currencyFormatter(adjustedRatio * 100, false, 1))
              .replaceAll('{{JET MIN C-RATIO}}', $MARKET.minColRatio * 100),
            action: {
              text: dictionary[$USER.preferredLanguage].cockpit.confirm,
              onClick: () => submitTrade()
            }
          }
        });
      }
    // If trade would result in possible undercollateralization, inform user
  } else if ((obligation?.borrowedValue && adjustedRatio < obligation?.colRatio || $USER.tradeAction === 'borrow')
      && adjustedRatio <= $MARKET.minColRatio + 0.2 && adjustedRatio >= $MARKET.minColRatio) {
        COPILOT.set({
          suggestion: {
            good: false,
            detail: dictionary[$USER.preferredLanguage].cockpit.subjectToLiquidation
              .replaceAll('{{NEW-C-RATIO}}', currencyFormatter(adjustedRatio * 100, false, 1)),                        
            action: {
              text: dictionary[$USER.preferredLanguage].cockpit.confirm,
              onClick: () => submitTrade()
            }
          }
        });
    } else {
      submitTrade();
    }
  };

  // Check user input and submit trade RPC call
  const submitTrade = async (): Promise<void> => {
    if (!$USER.currentReserve || !$USER.assets) {
      return;
    }

    if (!inputAmount) {
      inputError = dictionary[$USER.preferredLanguage].cockpit.noInputAmount;
      inputAmount = null;
      sendingTrade = false;
      return;
    }

    sendingTrade = true;
    let tradeAction = $USER.tradeAction;
    let tradeAmount = inputAmount;
    let ok;
    let txid;
    if (tradeAction === 'deposit') {
      if (TokenAmount.tokens(tradeAmount.toString(), walletBalances[$USER.currentReserve.abbrev]?.decimals).amount.gt(walletBalances[$USER.currentReserve.abbrev]?.amount)) {
        inputError = dictionary[$USER.preferredLanguage].cockpit.notEnoughAsset
          .replaceAll('{{ASSET}}', $USER.currentReserve.abbrev);
        inputAmount = null;
        sendingTrade = false;
        return;
      }

      inputError = '';
      const depositLamports = TokenAmount.tokens(tradeAmount.toString(), $USER.currentReserve.decimals).amount;
      [ok, txid] = await deposit($USER.currentReserve.abbrev, depositLamports);
    } else if (tradeAction === 'withdraw') {
      if (TokenAmount.tokens(tradeAmount.toString(), $USER.currentReserve.decimals).amount.gt($USER.currentReserve.availableLiquidity.amount)) {
        inputAmount = null;
        inputError = dictionary[$USER.preferredLanguage].cockpit.noLiquidity;
        sendingTrade = false;
        return;
      }

      let collateralBalance = $USER.assets.tokens[$USER.currentReserve.abbrev]?.collateralBalance;
      if (TokenAmount.tokens(tradeAmount.toString(), $USER.currentReserve.decimals).amount.gt(collateralBalance.amount)) {
        inputAmount = null;
        inputError = dictionary[$USER.preferredLanguage].cockpit.lessFunds;
        sendingTrade = false;
        return;
      }

      inputError = '';
      const withdrawLamports = TokenAmount.tokens(tradeAmount.toString(), $USER.currentReserve.decimals).amount;
      const withdrawAmount = tradeAmount === collateralBalances[$USER.currentReserve.abbrev] ?
        Amount.depositNotes($USER.assets.tokens[$USER.currentReserve.abbrev].collateralNoteBalance.amount) :
        Amount.tokens(withdrawLamports);
      [ok, txid] = await withdraw($USER.currentReserve.abbrev, withdrawAmount);
    } else if (tradeAction === 'borrow') {
      if (TokenAmount.tokens(tradeAmount.toString(), $USER.currentReserve.decimals).amount.gt($USER.currentReserve.availableLiquidity.amount)) {
        inputAmount = null;
        inputError = dictionary[$USER.preferredLanguage].cockpit.noLiquidity;
        sendingTrade = false;
        return;
      }

      if ((adjustedRatio && Math.ceil((adjustedRatio * 1000) / 1000) < $MARKET.minColRatio) || tradeAmount > maxBorrowAmounts[$USER.currentReserve.abbrev]) {
        inputAmount = null;
        inputError = dictionary[$USER.preferredLanguage].cockpit.belowMinCRatio;
        sendingTrade = false;
        return;
      }

      inputError = '';
      const borrowLamports = TokenAmount.tokens(tradeAmount.toString(), $USER.currentReserve.decimals);
      const borrowAmount = Amount.tokens(borrowLamports.amount);
      [ok, txid] = await borrow($USER.currentReserve.abbrev, borrowAmount);
    } else if (tradeAction === 'repay') {
      let loanBalance = $USER.assets.tokens[$USER.currentReserve.abbrev].loanBalance;
      if(!loanBalance) {
        sendingTrade = false;
        return;
      }

      if (TokenAmount.tokens(tradeAmount.toString(), $USER.currentReserve.decimals).amount.gt(loanBalance.amount)) {
        inputAmount = null;
        inputError = dictionary[$USER.preferredLanguage].cockpit.oweLess;
        sendingTrade = false;
        return;
      }

      inputError = '';
      const repayLamports = TokenAmount.tokens(tradeAmount.toString(), $USER.currentReserve.decimals).amount;
      const repayAmount = tradeAmount === loanBalances[$USER.currentReserve.abbrev]
        ? Amount.loanNotes($USER.assets.tokens[$USER.currentReserve.abbrev].loanNoteBalance.amount)
        : Amount.tokens(repayLamports);
      [ok, txid] = await repay($USER.currentReserve.abbrev, repayAmount);
    }
    
    if (ok && txid) {
      addNotification({
        success: true,
        text: dictionary[$USER.preferredLanguage].cockpit.txSuccess
          .replaceAll('{{TRADE ACTION}}', tradeAction)
          .replaceAll('{{AMOUNT AND ASSET}}', `${tradeAmount} ${$USER.currentReserve.abbrev}`)
      });
      addTransactionLog(txid);
      inputAmount = null;
    } else if (!ok && !txid) {
      addNotification({
        success: false,
        text: dictionary[$USER.preferredLanguage].cockpit.txFailed
      });
      inputAmount = null;
    }

    updateValues();
    adjustCollateralizationRatio();
    sendingTrade = false;
    return;
  };

  // Reactive statement to update data
  // on any reserve, user account or price change
  // every 2 seconds, on tx call, trade action or reserve change
  let updateTime: number = 0;
  $: if ($MARKET || $USER.assets || $USER.currentReserve || $USER.tradeAction) {
    const currentTime = performance.now();
    if (currentTime > updateTime) {
      updateValues();
      updateTime = currentTime + 2000;
    }

    // Add search icon to table search input
    if (!document.querySelector('.dt-search i')) {
      const searchIcon = document.createElement('i');
      searchIcon.classList.add('search', 'text-gradient', 'fas', 'fa-search');
      document.querySelector('.dt-search')?.appendChild(searchIcon);
    }

    // Hardcode min c-ratio to 130% for now
    MARKET.update(market => {
      market.minColRatio = 1.3;
      return market;
    });

    // If user is subject to liquidation, warn them once
    if ($USER.wallet && !$USER.warnedOfLiquidation && obligation?.borrowedValue && obligation?.colRatio <= $MARKET.minColRatio) {
      generateCopilotSuggestion();
    }
  }
</script>

{#if $MARKET && $USER.currentReserve && !$INIT_FAILED}
  <div class="view-container flex justify-center column">
    <h1 class="view-title text-gradient">
      {dictionary[$USER.preferredLanguage].cockpit.title}
    </h1>
    <div class="connect-wallet-btn">
      <ConnectWalletButton />
    </div>
    <div class="cockpit-top flex align-center justify-between">
      <div class="trade-market-tvl flex align-start justify-center column">
        <div class="divider">
        </div>
        <h2 class="view-subheader">
          {dictionary[$USER.preferredLanguage].cockpit.totalValueLocked}
        </h2>
        {#key $MARKET.reserves}
          <h1 class="view-header text-gradient">
            {totalAbbrev($MARKET.totalValueLocked())}
          </h1>
        {/key}
      </div>
      <div class="trade-position-snapshot flex align-center justify-center">
        <div class="trade-position-ratio flex align-start justify-center column">
          <div class="flex align-center justify-center">
            <h2 class="view-subheader">
              {dictionary[$USER.preferredLanguage].cockpit.yourRatio}
            </h2>
            <i class="info fas fa-info-circle"
              on:click={() => COPILOT.set({
                definition: definitions[$USER.preferredLanguage].collateralizationRatio
              })}>
            </i>
          </div>
          <h1 class="view-header"
            style={`margin-bottom: -20px; 
            ${$USER.walletInit ? (obligation?.borrowedValue && (obligation?.colRatio <= $MARKET.minColRatio) 
              ? 'color: var(--failure);' 
                : 'color: var(--success);') : ''}`}>
            {#if $USER.walletInit}
              {#if obligation?.borrowedValue && obligation?.colRatio > 10}
                &gt;1000
              {:else if obligation?.borrowedValue && obligation?.colRatio < 10}
                {currencyFormatter(obligation?.colRatio * 100, false, 1)}
              {:else}
                ∞
              {/if}
            {:else}
              --
            {/if}
            {#if obligation?.borrowedValue}
              <span style="color: inherit;">
                %
              </span>
            {/if}
          </h1>
        </div>
        <div class="flex align-center justify-center column">
          <div class="trade-position-value flex align-center justify-center column">
            <h2 class="view-subheader">
              {dictionary[$USER.preferredLanguage].cockpit.totalDepositedValue}
            </h2>
            <p class={`${$USER.wallet ? 'text-gradient' : ''} bicyclette`}>
              {$USER.walletInit ? totalAbbrev(obligation?.depositedValue ?? 0) : '--'}
            </p>
          </div>
          <div class="trade-position-value flex align-center justify-center column">
            <h2 class="view-subheader">
              {dictionary[$USER.preferredLanguage].cockpit.totalBorrowedValue}
            </h2>
            <p class={`${$USER.wallet ? 'text-gradient' : ''} bicyclette`}>
              {$USER.walletInit ? totalAbbrev(obligation?.borrowedValue ?? 0) : '--'}
            </p>
          </div>
        </div>
      </div>
    </div>
    <Datatable settings={tableSettings} data={tableData}>
      <thead>
        <th data-key="name">
          {dictionary[$USER.preferredLanguage].cockpit.asset} 
        </th>
        <th data-key="abbrev"
          class="native-toggle">
          <Toggle onClick={() => USER.update(user => {
            user.nativeValues = !$USER.nativeValues;
            return user;
          })}
            active={!$USER.nativeValues} 
            native 
          />
        </th>
        <th data-key="availableLiquidity">
          {dictionary[$USER.preferredLanguage].cockpit.availableLiquidity}
        </th>
        <th data-key="depositRate">
          {dictionary[$USER.preferredLanguage].cockpit.depositRate}
          <i class="info fas fa-info-circle"
              on:click={() => COPILOT.set({
                definition: definitions[$USER.preferredLanguage].depositRate
              })}>
          </i>
        </th>
        <th data-key="borrowRate" class="datatable-border-right">
          {dictionary[$USER.preferredLanguage].cockpit.borrowRate}
          <i class="info fas fa-info-circle"
              on:click={() => COPILOT.set({
                definition: definitions[$USER.preferredLanguage].borrowRate
              })}>
          </i>
        </th>
        <th data-key="">
          {dictionary[$USER.preferredLanguage].cockpit.walletBalance}
        </th>
        <th data-key="">
          {dictionary[$USER.preferredLanguage].cockpit.amountDeposited}
        </th>
        <th data-key="">
          {dictionary[$USER.preferredLanguage].cockpit.amountBorrowed}
        </th>
        <th data-key="">
          <!--Empty column for arrow-->
        </th>
      </thead>
      <div class="datatable-divider">
      </div>
      <tbody>
        {#each $rows as row, i}
          <tr class="datatable-spacer">
            <td><!-- Extra Row for spacing --></td>
          </tr>
          <tr class:active={$USER.currentReserve.abbrev === $rows[i].abbrev}>
            <td class="dt-asset" on:click={() => changeReserve($rows[i])}>
              <img src={`img/cryptos/${$rows[i].abbrev}.png`} 
                alt={`${$rows[i].abbrev} Icon`}
              />
              <span>
                {$rows[i].name}
              </span>
              <span>
                ≈ {currencyFormatter($rows[i].price, true, 2)}
              </span>
            </td>
            <td on:click={() => reserveDetail = $rows[i]} 
              class="reserve-detail">
              {$rows[i].abbrev} {dictionary[$USER.preferredLanguage].cockpit.detail}
            </td>
            <td on:click={() => changeReserve($rows[i])}>
              {totalAbbrev(
                $rows[i].availableLiquidity?.uiAmountFloat,
                $rows[i].price,
                $USER.nativeValues,
                2
              )}
            </td>
            <td on:click={() => changeReserve($rows[i])}>
              {$rows[i].depositRate ? ($rows[i].depositRate * 100).toFixed(2) : 0}%
            </td>
            <td on:click={() => changeReserve($rows[i])} 
              class="datatable-border-right">
              {$rows[i].borrowRate ? ($rows[i].borrowRate * 100).toFixed(2) : 0}%
            </td>
            <td class:dt-bold={walletBalances[$rows[i].abbrev]?.uiAmountFloat} 
              class:dt-balance={walletBalances[$rows[i].abbrev]?.uiAmountFloat} 
              on:click={() => changeReserve($rows[i])}>
              {#if $USER.wallet}
                {#if walletBalances[$rows[i].abbrev]?.uiAmountFloat && walletBalances[$rows[i].abbrev]?.uiAmountFloat < 0.0005}
                  ~0
                {:else}
                  {totalAbbrev(
                    walletBalances[$rows[i].abbrev]?.uiAmountFloat ?? 0,
                    $rows[i].price,
                    $USER.nativeValues,
                    3
                  )}
                {/if}
              {:else}
                  --
              {/if}
            </td>
            <td class:dt-bold={collateralBalances[$rows[i].abbrev]} 
              on:click={() => changeReserve($rows[i])}
              style={collateralBalances[$rows[i].abbrev] ? 
                'color: var(--jet-green) !important;' : ''}>
              {#if $USER.walletInit}
                {#if collateralBalances[$rows[i].abbrev] && collateralBalances[$rows[i].abbrev] < 0.0005}
                  ~0
                {:else}
                  {totalAbbrev(
                    collateralBalances[$rows[i].abbrev],
                    $rows[i].price,
                    $USER.nativeValues,
                    3
                  )}
                {/if}
              {:else}
                  --
              {/if}
            </td>
            <td class:dt-bold={loanBalances[$rows[i].abbrev]} 
              on:click={() => changeReserve($rows[i])}
              style={loanBalances[$rows[i].abbrev] ? 
              'color: var(--jet-blue) !important;' : ''}>
              {#if $USER.walletInit}
                {#if loanBalances[$rows[i].abbrev] && loanBalances[$rows[i].abbrev] < 0.0005}
                  ~0
                {:else}
                  {totalAbbrev(
                    loanBalances[$rows[i].abbrev],
                    $rows[i].price,
                    $USER.nativeValues,
                    3
                  )}
                {/if}
              {:else}
                --
              {/if}
            </td>
            <!--Faucet for testing if in development-->
            <!--Replace with inDevelopment for mainnet-->
            {#if inDevelopment}
              <td class="faucet" on:click={() => doAirdrop($rows[i])}>
                <i class="text-gradient fas fa-parachute-box"
                  title={`Airdrop ${$rows[i].abbrev}`}
                  style="margin-right: var(--spacing-lg); font-size: 18px !important;">
                </i>
              </td>
            {:else}
              <td on:click={() => changeReserve($rows[i])}>
                  <i class="text-gradient jet-icons">
                    ➜
                  </i>
                </td>
            {/if}
          </tr>
          <tr class="datatable-spacer">
            <td><!-- Extra Row for spacing --></td>
          </tr>
        {/each}
      </tbody>
    </Datatable>
    <div class="trade-action flex align-center justify-start">
      <div class="trade-action-select-container flex align-center justify-between">
        {#each ['deposit', 'withdraw', 'borrow', 'repay'] as action}
          <div on:click={() => {
              if (sendingTrade) {
                return;
              }

              inputAmount = null;
              inputError = '';
              USER.update(user => {
                user.tradeAction = action;
                return user;
              })
              updateValues();
            }} 
            class="trade-action-select flex justify-center align-center"
            class:active={$USER.tradeAction === action}>
            <p class="bicyclette">
              {dictionary[$USER.preferredLanguage].cockpit[action]}
            </p>
          </div>
        {/each}
      </div>
      {#if disabledMessage}
        <div class="trade-action-section trade-disabled-message flex align-center justify-center column">
          <span>
            {disabledMessage}
          </span>
        </div>
      {:else}
        <div class="trade-action-section flex align-center justify-center column"
          class:disabled={disabledInput}>
          <span>
            {#if $USER.tradeAction === 'deposit'}
              {dictionary[$USER.preferredLanguage].cockpit.walletBalance.toUpperCase()}
            {:else if $USER.tradeAction === 'withdraw'}
              {dictionary[$USER.preferredLanguage].cockpit.availableFunds.toUpperCase()}
            {:else if $USER.tradeAction === 'borrow'}
              {dictionary[$USER.preferredLanguage].cockpit.maxBorrowAmount.toUpperCase()}
            {:else if $USER.tradeAction === 'repay'}
              {dictionary[$USER.preferredLanguage].cockpit.amountOwed.toUpperCase()}
            {/if}
          </span>
          <div class="flex align-center justify-center">
            {#if $USER.walletInit}
              {#if $USER.tradeAction === 'deposit'}
                <p>
                  {currencyFormatter(
                    walletBalances[$USER.currentReserve.abbrev]?.uiAmountFloat ?? 0,
                    false,
                    $USER.currentReserve.decimals
                  )} 
                  {$USER.currentReserve.abbrev}
                </p>
              {:else if $USER.tradeAction === 'withdraw'}
                <p>
                  {currencyFormatter(maxWithdrawAmounts[$USER.currentReserve.abbrev], false, $USER.currentReserve.decimals)} 
                  {$USER.currentReserve.abbrev}
                </p>
              {:else if $USER.tradeAction === 'borrow'}
                <p>
                  {currencyFormatter(maxBorrowAmounts[$USER.currentReserve.abbrev], false, $USER.currentReserve.decimals)} 
                  {$USER.currentReserve.abbrev}
                </p>
              {:else if $USER.tradeAction === 'repay'}
                <p>
                  {currencyFormatter(
                    loanBalances[$USER.currentReserve.abbrev],
                    false, 
                    $USER.currentReserve.decimals
                  )} 
                  {$USER.currentReserve.abbrev}
                </p>
              {/if}
            {:else}
              <p>
                --
              </p>
            {/if}
          </div>
        </div>
        <div class="trade-action-section flex align-center justify-center column"
          class:disabled={disabledInput}>
          <div class="flex align-center justify-center">
            <span>
              {dictionary[$USER.preferredLanguage].cockpit.adjustedCollateralization.toUpperCase()}
            </span>
            <i class="info fas fa-info-circle" 
              style="color: var(--white);"
              on:click={() => COPILOT.set({
                definition: definitions[$USER.preferredLanguage].adjustedCollateralizationRatio
              })}>
            </i>
          </div>
          <p class="bicyclette">
            {#if $USER.walletInit}
              {#if (obligation?.borrowedValue || ($USER.tradeAction === 'borrow' && inputAmount)) && adjustedRatio > 10}
                &gt; 1000%
              {:else if (obligation?.borrowedValue || ($USER.tradeAction === 'borrow' && inputAmount)) && adjustedRatio < 10}
                {currencyFormatter(adjustedRatio * 100, false, 1) + '%'}
              {:else}
                ∞
              {/if}
            {:else}
              --
            {/if}
          </p>
        </div>
      {/if}
      <div class="trade-action-section flex align-center justify-center column">
        <div class="flex align-center justify-center">
          <div class="submit-input flex align-center justify-center"
            class:active={inputAmount} class:disabled={disabledInput}>
            <input on:keyup={() => {
                // If input is negative, reset to zero
                if (inputAmount && inputAmount < 0) {
                  inputAmount = 0;
                }
                adjustCollateralizationRatio();
              }}
              on:keypress={(e) => {
                if (e.key === "Enter"){
                  return checkSubmit()
                }
              }}
              on:click={() => inputError = ''}
              bind:value={inputAmount}
              placeholder={inputError ?? ''}
              class={inputError ? 'input-error' : ''}
              type="number" max={maxInputValue}
              disabled={disabledInput} 
            />
            <img src={`img/cryptos/${$USER.currentReserve?.abbrev}.png`} alt={`${$USER.currentReserve?.name} Logo`} />
            <div class="asset-abbrev-usd flex align-end justify-center column">
              <span>
                {$USER.currentReserve?.abbrev}
              </span>
              <span>
                ≈ {currencyFormatter(
                    (inputAmount ?? 0) * $USER.currentReserve.price,
                    true,
                    2
                  )}
              </span>
            </div>
          </div>
          <div class="submit-input-btn flex align-center justify-center"
            class:active={sendingTrade}
            class:disabled={disabledInput}
            on:click={() => checkSubmit()}>
            {#if sendingTrade}
              <Loader button />
            {:else}
              <i class="jet-icons"
                title="Submit Trade">
                ➜
              </i>
            {/if}
          </div>
        </div>
        <RangeSlider float 
          values={[0]}
          min={0} max={100} 
          step={25} suffix="%" 
          disabled={disabledInput}
          springValues={{stiffness: 1, damping: 1}}
          on:change={sliderUpdate}
        />
      </div>
    </div>
  </div>
  {#if reserveDetail}
    <ReserveDetail {reserveDetail}
      {updateValues}
      closeModal={() => {
        if (reserveDetail?.abbrev !== $USER.currentReserve?.abbrev) {
          inputAmount = null;
        }
        reserveDetail = null;
      }} />
  {/if}
{:else if $INIT_FAILED || $USER.isGeobanned}
  <InitFailed />
{:else}
  <Loader fullview />
{/if}

<style>
  .view-container {
    position: relative;
  }
  .cockpit-top {
    flex-wrap: wrap;
    padding: var(--spacing-xs) 0 var(--spacing-lg) 0;
  }
  .connect-wallet-btn {
    position: absolute;
    top: var(--spacing-md);
    right: var(--spacing-sm);
  }
  .trade-market-tvl .divider {
    margin: 0 0 var(--spacing-lg) 0;
  }
  .trade-position-snapshot {
    min-width: 275px;
    border-radius: var(--border-radius);
    box-shadow: var(--neu-shadow-inset);
    padding: var(--spacing-sm) var(--spacing-lg);
    background: var(--light-grey);
  }
  .trade-position-snapshot p {
    font-size: 25px;
  }
  .trade-position-ratio {
    padding-right: 50px;
  }
  .trade-position-value {
    padding: var(--spacing-sm) 0;
  }
  .trade-action {
    position: relative;
    width: 100%;
    padding-top: calc(var(--spacing-lg) * 1.75);
    border-bottom-left-radius: var(--border-radius);
    border-bottom-right-radius: var(--border-radius);
    box-shadow: var(--neu--datatable-bottom-shadow);
    background: var(--gradient);
    overflow: hidden;
    z-index: 10;
  }
  .trade-action-select-container {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    z-index: 11;
  }
  .trade-action-select {
    width: 25%;
    border-right: 1px solid var(--white);
    padding: var(--spacing-sm) 0;
    background: rgba(255, 255, 255, 0.2);
    opacity: var(--disabled-opacity);
    cursor: pointer;
  }
  .trade-action-select:last-of-type {
    border-right: unset;
  }
  .trade-action-select.active {
    background: unset;
    opacity: 1;
  }
  .trade-action-select p {
    position: relative;
    font-size: 16px;
    letter-spacing: 0.5px;
    line-height: 17px;
    color: var(--white);
  }
  .trade-action-section {
    position: relative;
    width: calc(25% - (var(--spacing-sm) * 2));
    padding: var(--spacing-lg) var(--spacing-sm) var(--spacing-xs) var(--spacing-sm);
  }
  .trade-action-section:last-of-type {
    width: calc(50% - (var(--spacing-sm) * 2));
  }
  .trade-action-section p, .trade-action-section span {
    text-align: center;
    color: var(--white);
  }
  .trade-action-section span {
    font-weight: bold;
    font-size: 10px;
    letter-spacing: 0.5px;
  }
  .trade-action-section p {
    font-size: 21px;
  }
  .trade-action-section .max-input:active span, .trade-action-section .max-input.active span {
    background-image: var(--gradient) !important;
    -webkit-background-clip: text !important;
    -webkit-text-fill-color: transparent !important;
  }
  .trade-disabled-message {
    width: calc(50% - (var(--spacing-sm) * 2))
  }
  .trade-disabled-message span {
    font-weight: 400;
    font-size: 14px;
    padding: var(--spacing-sm);
  }
  
  @media screen and (max-width: 1100px) {
    .cockpit-top {
      flex-direction: column;
      align-items: flex-start;
      padding-top: unset;
    }
    .connect-wallet-btn {
      display: none;
    }
    .trade-market-tvl, .trade-position-snapshot {
      min-width: unset;
      margin: var(--spacing-xs) 0;
    }
    .trade-position-snapshot h1 {
      font-size: 50px;
      line-height: 50px;
    }
    .trade-position-snapshot p {
      font-size: 20px;
      line-height: 20px;
    }
    .trade-position-ratio {
      padding-right: 20px;
    }
    .trade-action {
      padding-top: 55px;
      flex-direction: column;
      justify-content: center;
    }
    .trade-action-select p {
      font-size: 12px;
    }
    .trade-action-section {
      width: 100% !important;
      padding: var(--spacing-xs) 0;
    }
    .trade-action-section:last-of-type {
      padding-bottom: unset;
    }
    .trade-action-section p {
      font-size: 25px;
    }
    .trade-disabled-message span {
      max-width: 200px;
    }
  }
</style>