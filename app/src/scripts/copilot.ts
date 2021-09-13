import { navigate } from "svelte-navigator";
import type { Market, AssetStore, } from '../models/JetTypes';
import { COPILOT, MARKET, ASSETS, CURRENT_RESERVE, PREFERRED_LANGUAGE } from '../store';
import { currencyFormatter, getObligationData } from "./utils";
import { dictionary } from './localization';

let market: Market | null;
let assets: AssetStore | null;
let preferredLanguage: string;
MARKET.subscribe(data => market = data);
ASSETS.subscribe(data => assets = data);
PREFERRED_LANGUAGE.subscribe(data => preferredLanguage = data);

const obligation = getObligationData();

// Generate suggestion for user based on their current position and market data
export const generateCopilotSuggestion = (): void => {
  if (!market || !assets) {
    COPILOT.set(null);
    return;
  }

  let bestReserveDepositAPY = market.reserves.SOL;
  let colRatio = getObligationData()?.colRatio ?? 0;

  // Find best deposit APY
  if (market.reserves) {
    for (let a in market.reserves) {
      bestReserveDepositAPY = market.reserves[a];
    };
  }

  // Conditional AI for suggestion generation
  if (obligation.borrowedValue && (colRatio < market?.minColRatio)) {
    COPILOT.set({
      suggestion: {
        good: false,
        overview: dictionary[preferredLanguage].copilot.suggestions.unhealthy.overview,
        detail: dictionary[preferredLanguage].copilot.suggestions.unhealthy.detail
          .replace('{{C-RATIO}}', currencyFormatter(colRatio * 100, false, 1))
          .replace('{{RATIO BELOW AMOUNT}}', Math.abs(Number(currencyFormatter((market.minColRatio - colRatio) * 100, false, 1))))
          .replace('{{JET MIN C-RATIO}}', market.minColRatio * 100),
        solution: dictionary[preferredLanguage].copilot.suggestions.unhealthy.solution,
        action: {
          text: dictionary[preferredLanguage].copilot.suggestions.unhealthy.actionText,
          onClick: () => navigate('/cockpit')
        }
      }
    });
  } else if (bestReserveDepositAPY != null && parseInt(assets.tokens[bestReserveDepositAPY.abbrev].walletTokenBalance?.amount.toString())) {
    CURRENT_RESERVE.set(bestReserveDepositAPY);
    COPILOT.set({
      suggestion: {
        good: true,
        overview: dictionary[preferredLanguage].copilot.suggestions.deposit.overview
          .replace('{{BEST DEPOSIT APY NAME}}', bestReserveDepositAPY.name),
        detail: dictionary[preferredLanguage].copilot.suggestions.deposit.detail
          .replace('{{BEST DEPOSIT APY ABBREV}}', bestReserveDepositAPY.abbrev)
          .replace('{{DEPOSIT APY}}', bestReserveDepositAPY.depositAPY)
          .replace('{{USER BALANCE}}', currencyFormatter(assets.tokens[bestReserveDepositAPY.abbrev].walletTokenBalance.uiAmountFloat, false, 2)),
        action: {
          text: dictionary[preferredLanguage].copilot.suggestions.deposit.actionText,
          onClick: () => navigate('/deposit')
        }
      }
    });
  } else if (obligation.borrowedValue && (colRatio > market?.minColRatio && colRatio <= market?.minColRatio + 10)) {
    COPILOT.set({
      suggestion: {
        good: false,
        overview: dictionary[preferredLanguage].copilot.warning.tenPercent.overview,
        detail: dictionary[preferredLanguage].copilot.warning.tenPercent.detail
          .replace('{{C-RATIO}}', currencyFormatter(colRatio * 100, false, 1))
          .replace('{{JET MIN C-RATIO}}', market.minColRatio * 100),
        action: {
          text: dictionary[preferredLanguage].copilot.warning.tenPercent.actionText,
          onClick: () => navigate('/deposit')
        }
      }
    });
  } else if (obligation.borrowedValue && (colRatio >= market?.minColRatio + 10 && colRatio <= market?.minColRatio + 20)) {
    COPILOT.set({
      suggestion: {
        good: false,
        overview: dictionary[preferredLanguage].copilot.warning.twentyPercent.overview,
        detail: dictionary[preferredLanguage].copilot.warning.twentyPercent.detail
          .replace('{{C-RATIO}}', currencyFormatter(colRatio * 100, false, 1))
          .replace('{{JET MIN C-RATIO}}', market.minColRatio * 100),
        action: {
          text: dictionary[preferredLanguage].copilot.warning.twentyPercent.actionText,
          onClick: () => navigate('/deposit')
        }
      }
    });
  } else {
    COPILOT.set({
      suggestion: {
        good: true,
        overview: dictionary[preferredLanguage].copilot.suggestions.healthy.overview,
        detail: dictionary[preferredLanguage].copilot.suggestions.healthy.detail
      }
    });
  }
};