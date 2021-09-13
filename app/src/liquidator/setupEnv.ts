import { Coder, Program, Provider, Wallet } from '@project-serum/anchor';
import { Connection } from '@solana/web3.js';
import fs from 'fs';
import { resolve } from "path/posix";
import { TestUtils } from '../jet-test';
import { JetUtils } from '../jet-test/jet';
import type { IdlMetadata, MarketAccount } from '../models/JetTypes';
import { getAccountInfoAndSubscribe, parseIdlMetadata, parseMarketAccount } from '../scripts/programUtil';
import { parsePriceData } from '@pythnetwork/client';
import { NodeWallet } from '@project-serum/anchor/dist/provider';

export interface Env {
    connection: Connection,
    wallet: Wallet,
    metadata: IdlMetadata,
    coder: Coder,
    program: Program,
    testUtils: TestUtils,
    jetUtils: JetUtils,
    state: {
        prices: Record<string, number | undefined>,
        market?: MarketAccount,
    }
};

export const setupEnv = async (): Promise<Env> => {
    const idlPath = 'target/idl/jet.json'; // load from firestore
    const absIdlPath = resolve(idlPath);
    const idl = JSON.parse(fs.readFileSync(absIdlPath, 'utf-8'));

    const metadata = parseIdlMetadata(idl.metadata);
    const connection  = new Connection(metadata.cluster);
    const wallet = NodeWallet.local(); // load from google secrets
    const provider = new Provider(connection,wallet, Provider.defaultOptions());
    const program = new Program(idl, idl.metadata.address, provider);
    const coder = program.coder;

    const testUtils = new TestUtils(provider.connection, wallet);
    const jetUtils = new JetUtils(provider.connection, wallet, program);

    const env: Env = { connection, wallet, metadata, coder, program, testUtils, jetUtils, state: { prices: {} } };

    subscribe(env);

    return env;
};

const subscribe = (env: Env) => {
    // Market
    getAccountInfoAndSubscribe(env.connection, env.metadata.market.market, account => {
        if (account != null) {
            env.state.market = parseMarketAccount(account.data, env.coder);
        }
    });

    for (const reserveMeta of env.metadata.reserves) {
        // Pyth Price
        const reserveIndex = reserveMeta.reserveIndex;
        getAccountInfoAndSubscribe(env.connection, reserveMeta.accounts.pythPrice, account => {
            if (account != null) {
                env.state.prices[reserveIndex] = parsePriceData(account.data).price;
            }
        });
    }
};