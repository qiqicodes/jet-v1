import type { Market, User, } from '../models/JetTypes';
import { COPILOT, MARKET, USER } from '../store';
import { currencyFormatter, getObligationData } from "./util";
import { dictionary } from './localization';

let market: Market | null;
let user: User;
MARKET.subscribe(data => market = data);
USER.subscribe(data => user = data);

// Generate suggestion for user based on their current position and market data
export const generateCopilotSuggestion = (): void => {
  if (!market || !user.assets) {
    COPILOT.set(null);
    return;
  }

  let bestReserveDepositRate = market.reserves.SOL;

  // Find best deposit Rate
  if (market.reserves) {
    for (let a in market.reserves) {
      if (market.reserves[a].depositRate > bestReserveDepositRate.depositRate) {
        bestReserveDepositRate = market.reserves[a];
      }
    };
  }

  // Conditional AI for suggestion generation
  const obligation = getObligationData();
  if (obligation.borrowedValue && (obligation.colRatio < market?.minColRatio)) {
    COPILOT.set({
      suggestion: {
        good: false,
        overview: dictionary[user.preferredLanguage].copilot.suggestions.unhealthy.overview,
        detail: dictionary[user.preferredLanguage].copilot.suggestions.unhealthy.detail
          .replaceAll('{{C-RATIO}}', currencyFormatter(obligation.colRatio * 100, false, 1))
          .replaceAll('{{RATIO BELOW AMOUNT}}', Math.abs(Number(currencyFormatter((market.minColRatio - obligation.colRatio) * 100, false, 1))))
          .replaceAll('{{JET MIN C-RATIO}}', market.minColRatio * 100),
        solution: dictionary[user.preferredLanguage].copilot.suggestions.unhealthy.solution,
        action: {
          onClick: () => USER.update(user => {
            user.warnedOfLiquidation = true;
            return user;
          })
        } 
      }
    });
  } else if (bestReserveDepositRate?.depositRate && !user.assets.tokens[bestReserveDepositRate.abbrev].walletTokenBalance?.amount.isZero()) {
    USER.update(user => {
      user.currentReserve = bestReserveDepositRate;
      return user;
    })
    COPILOT.set({
      suggestion: {
        good: true,
        overview: dictionary[user.preferredLanguage].copilot.suggestions.deposit.overview
          .replaceAll('{{BEST DEPOSIT RATE NAME}}', bestReserveDepositRate.name),
        detail: dictionary[user.preferredLanguage].copilot.suggestions.deposit.detail
          .replaceAll('{{BEST DEPOSIT RATE ABBREV}}', bestReserveDepositRate.abbrev)
          .replaceAll('{{DEPOSIT RATE}}', (bestReserveDepositRate.depositRate * 100).toFixed(2))
          .replaceAll('{{USER BALANCE}}', currencyFormatter(user.assets.tokens[bestReserveDepositRate.abbrev].walletTokenBalance.uiAmountFloat, false, 2))
      }
    });
  } else if (obligation.borrowedValue && (obligation.colRatio > market?.minColRatio && obligation.colRatio <= market?.minColRatio + 10)) {
    COPILOT.set({
      suggestion: {
        good: false,
        overview: dictionary[user.preferredLanguage].copilot.warning.tenPercent.overview,
        detail: dictionary[user.preferredLanguage].copilot.warning.tenPercent.detail
          .replaceAll('{{C-RATIO}}', currencyFormatter(obligation.colRatio * 100, false, 1))
          .replaceAll('{{JET MIN C-RATIO}}', market.minColRatio * 100)
      }
    });
  } else if (obligation.borrowedValue && (obligation.colRatio >= market?.minColRatio + 10 && obligation.colRatio <= market?.minColRatio + 20)) {
    COPILOT.set({
      suggestion: {
        good: false,
        overview: dictionary[user.preferredLanguage].copilot.warning.twentyPercent.overview,
        detail: dictionary[user.preferredLanguage].copilot.warning.twentyPercent.detail
          .replaceAll('{{C-RATIO}}', currencyFormatter(obligation.colRatio * 100, false, 1))
          .replaceAll('{{JET MIN C-RATIO}}', market.minColRatio * 100)
      }
    });
  } else {
    COPILOT.set({
      suggestion: {
        good: true,
        overview: dictionary[user.preferredLanguage].copilot.suggestions.healthy.overview,
        detail: dictionary[user.preferredLanguage].copilot.suggestions.healthy.detail
      }
    });
  }
};