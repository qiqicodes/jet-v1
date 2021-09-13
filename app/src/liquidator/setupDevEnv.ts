import { Keypair } from "@solana/web3.js";
import { TestToken, toBN, toPublicKeys } from "../jet-test";
import type { ReserveConfig } from "../jet-test/jet";
import { Env, setupEnv } from "./setupEnv";

const INITIAL_PRICES: Record<string, bigint> = {
    'BTC': 45000n,
    'ETH': 3000n,
    'SOL': 75n,
    'SRM': 7n,
};

// How to propagate necessary info into liquidateObligation for testing
// during developement? See deploy.ts for working with JSON.
export const main = async () => {
    const env = await setupEnv();
    const testUtils = env.testUtils;
    const jetUtils = env.jetUtils;

    console.log('Creating market...');
    const marketKeypair = Keypair.generate();
    const marketOwnerKeypair = Keypair.generate();
    await createLendingMarket(env, marketKeypair, marketOwnerKeypair);

    console.log('Creating mints...');
    const mints = await prepareMints(env);

    console.log('Creating oracle accounts...');
    let oracles: Record<string, [Keypair, Keypair]> = {};
    for (const ccy in mints) {
        if (ccy == "USD") continue;

        let productAccount = await testUtils.pyth.createProductAccount();
        let priceAccount = await testUtils.pyth.createPriceAccount();

        oracles[ccy] = [productAccount, priceAccount];

        await testUtils.pyth.updateProductAccount(
            productAccount,
            {
                priceAccount: priceAccount.publicKey,
                attributes: {
                    quote_currency: "USD",
                }
            }
        );
        await testUtils.pyth.updatePriceAccount(
            priceAccount,
            {
                aggregatePriceInfo: {
                    price: INITIAL_PRICES[ccy]
                }
            }
        );
    }

    console.log('Creating reserves...');
    const reserveConfig = toBN({
        utilizationRate1: 8500,
        utilizationRate2: 9500,
        borrowRate0: 50,
        borrowRate1: 392,
        borrowRate2: 3364,
        borrowRate3: 10116,
        minCollateralRatio: 12500,
        liquidationPremium: 300,
    }) as ReserveConfig;
    for (const ccy in mints) {
        const mint = mints[ccy];
        const reserveAccounts = await jetUtils.createReserveAccount(
            mint, toBN(0), marketKeypair, marketOwnerKeypair.publicKey
        );
        await jetUtils.initReserve(
            reserveAccounts,
            reserveConfig,
            toBN(0),
            marketKeypair.publicKey,
            marketOwnerKeypair.publicKey,
            oracles[ccy][0].publicKey,
            oracles[ccy][1].publicKey,
        );
    }
};

export const prepareMints = async (
    env: Env
): Promise<Record<string, TestToken>> => {
    const tokenDecimals: Record<string, number> = {
        'USDC': 6,  // beware
         'BTC': 4,   // i
         'ETH': 6,   // made
         'SOL': 3,   // this
         'SRM': 6,   // up
    };

    let mints: Record<string, TestToken> = {};
    for (const label in tokenDecimals) {
        mints[label] = await env.testUtils.createToken(tokenDecimals[label]);
    }

    return mints;
};

export const createLendingMarket = async (
    env: Env,
    market: Keypair,
    marketOwner: Keypair,
) => {
    const program = env.program;

    let quoteCurrency = Buffer.alloc(32);
    quoteCurrency.write("USD");

    await program.rpc.initMarket(
        marketOwner.publicKey, "USD",
        {
            accounts: toPublicKeys({
                market,
        }),
        signers: [market],
        instructions: [
            await program.account.market.createInstruction(market),
        ],
    });
};

main();
