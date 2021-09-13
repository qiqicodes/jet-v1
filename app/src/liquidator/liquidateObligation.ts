import type { ObligationAccount, ObligationPositionStruct } from "../models/JetTypes";
import { PublicKey, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import type { Env } from "./setupEnv";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { findCollateralAddress, findLoanNoteAddress } from "../scripts/programUtil";
import { BN } from "@project-serum/anchor";

export const liquidateObligation = async (
    env: Env,
    obligationPubkey: PublicKey,
    obligation: ObligationAccount,
) => {
    console.log(obligationPubkey);
    console.log(obligation);

    const loan = highestValuePosition(env, obligation.loans, "loan");
    const collateral = highestValuePosition(env, obligation.collateral, "collateral");

    if (!loan || !collateral) {
        return;
    }

    const loanReserve = env.metadata.reserves[loan.reserveIndex];
    const collateralReserve = env.metadata.reserves[collateral.reserveIndex];

    const [, loanNoteBump] = await findLoanNoteAddress(env.program, loanReserve.accounts.reserve, obligationPubkey, obligation.owner);
    const [, collateralNoteBump] = await findCollateralAddress(env.program, collateralReserve.accounts.reserve, obligationPubkey, obligation.owner);

    // Multiple hops is not supported yet, assume exactly one market
    const serumMarket = collateralReserve.accounts.dexMarket;
    
    const bumps = {
        loanAccount: loanNoteBump,
        collateralAccount: collateralNoteBump,
    }
    const side = { ask: {} } // { bid: {} }
    await env.program.rpc.liquidateDex(bumps, side, {
        market: env.metadata.market.market,
        marketAuthority: env.metadata.market.marketAuthority,

        obligation: obligationPubkey,
        owner: obligation.owner,

        loanReserve: loanReserve.accounts.reserve,
        loanReserveVault: loanReserve.accounts.vault,
        loanNoteMint: loanReserve.accounts.loanNoteMint,
        loanAccount: loan.account,

        collateralReserve: collateralReserve.accounts.reserve,
        collateralReserveVault: collateralReserve.accounts.vault,
        depositNoteMint: collateralReserve.accounts.depositNoteMint,
        collateralAccount: collateral.account,

        //dexMarket: serumMarket.pubkeys, todo

        //dexProgram: env.metadata.serumProgramId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
    });
};

const highestValuePosition = (env: Env, positions: ObligationPositionStruct[], side: "collateral" | "loan") => {
    let highestValue = new BN(0);
    let highestValuePosition: ObligationPositionStruct | undefined;
    for (const position of positions) {
        const price = env.state.prices[position.reserveIndex];
        const market = env.state.market;
        if (!price || !market) {
            continue;
        }

        const marketReserve = market.reserves[position.reserveIndex];
        const decimals = env.metadata.reserves[position.reserveIndex].decimals
        const noteExchangeRate = side === "collateral"
            ? marketReserve.depositNoteExchangeRate
            : marketReserve.loanNoteExchangeRate;

        const value = position.amount
            .mul(noteExchangeRate)
            .muln(price)
            .divn(Math.pow(10, decimals));

        if (value.gt(highestValue)) {
            highestValuePosition = position;
            highestValue = value;
        }
    }
    return highestValuePosition;
}