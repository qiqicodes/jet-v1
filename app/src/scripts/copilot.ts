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
      if (market.reserves[a].depositAPY > bestReserveDepositAPY.depositAPY) {
        bestReserveDepositAPY = market.reserves[a];
      }
    };
  }

  // Conditional AI for suggestion generation
  if (obligation.borrowedValue && (colRatio < market?.minColRatio)) {
    COPILOT.set({
      suggestion: {
        good: false,
        overview: dictionary[preferredLanguage].copilot.suggestions.unhealthy.overview,
        detail: dictionary[preferredLanguage].copilot.suggestions.unhealthy.detail
          .replaceAll('{{C-RATIO}}', currencyFormatter(colRatio * 100, false, 1))
          .replaceAll('{{RATIO BELOW AMOUNT}}', Math.abs(Number(currencyFormatter((market.minColRatio - colRatio) * 100, false, 1))))
          .replaceAll('{{JET MIN C-RATIO}}', market.minColRatio * 100),
        solution: dictionary[preferredLanguage].copilot.suggestions.unhealthy.solution
      }
    });
  } else if (bestReserveDepositAPY?.depositAPY && !assets.tokens[bestReserveDepositAPY.abbrev].walletTokenBalance?.amount.isZero()) {
    CURRENT_RESERVE.set(bestReserveDepositAPY);
    COPILOT.set({
      suggestion: {
        good: true,
        overview: dictionary[preferredLanguage].copilot.suggestions.deposit.overview
          .replaceAll('{{BEST DEPOSIT APY NAME}}', bestReserveDepositAPY.name),
        detail: dictionary[preferredLanguage].copilot.suggestions.deposit.detail
          .replaceAll('{{BEST DEPOSIT APY ABBREV}}', bestReserveDepositAPY.abbrev)
          .replaceAll('{{DEPOSIT APY}}', bestReserveDepositAPY.depositAPY)
          .replaceAll('{{USER BALANCE}}', currencyFormatter(assets.tokens[bestReserveDepositAPY.abbrev].walletTokenBalance.uiAmountFloat, false, 2))
      }
    });
  } else if (obligation.borrowedValue && (colRatio > market?.minColRatio && colRatio <= market?.minColRatio + 10)) {
    COPILOT.set({
      suggestion: {
        good: false,
        overview: dictionary[preferredLanguage].copilot.warning.tenPercent.overview,
        detail: dictionary[preferredLanguage].copilot.warning.tenPercent.detail
          .replaceAll('{{C-RATIO}}', currencyFormatter(colRatio * 100, false, 1))
          .replaceAll('{{JET MIN C-RATIO}}', market.minColRatio * 100)
      }
    });
  } else if (obligation.borrowedValue && (colRatio >= market?.minColRatio + 10 && colRatio <= market?.minColRatio + 20)) {
    COPILOT.set({
      suggestion: {
        good: false,
        overview: dictionary[preferredLanguage].copilot.warning.twentyPercent.overview,
        detail: dictionary[preferredLanguage].copilot.warning.twentyPercent.detail
          .replaceAll('{{C-RATIO}}', currencyFormatter(colRatio * 100, false, 1))
          .replaceAll('{{JET MIN C-RATIO}}', market.minColRatio * 100)
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