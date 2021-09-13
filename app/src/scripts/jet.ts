import { AccountInfo, Keypair, PublicKey, Signer, SystemProgram, SYSVAR_RENT_PUBKEY, TransactionInstruction } from '@solana/web3.js';
import * as anchor from '@project-serum/anchor';
import { BN } from '@project-serum/anchor';
import { NATIVE_MINT } from "@solana/spl-token";
import { AccountLayout as TokenAccountLayout, Token, TOKEN_PROGRAM_ID, u64 } from "@solana/spl-token";
import Rollbar from 'rollbar';
import { initializeApp } from 'firebase/app';
import { getFirestore, getDoc, doc } from 'firebase/firestore/lite';
import WalletAdapter from './walletAdapter';
import type { Reserve, AssetStore, SolWindow, WalletProvider, Wallet, Asset, Market, Reserves, MathWallet, SolongWallet } from '../models/JetTypes';
import { MARKET, WALLET, ASSETS, PROGRAM, CURRENT_RESERVE } from '../store';
import { subscribeToAssets, subscribeToMarket } from './subscribe';
import { findDepositNoteAddress, findDepositNoteDestAddress, findLoanNoteAddress, findObligationAddress, sendTransaction, transactionErrorToString, parseTokenAccount, findCollateralAddress, SOL_DECIMALS, parseIdlMetadata } from './programUtil';
import { TokenAmount } from './utils';
import { Buffer } from 'buffer';

const SECONDS_PER_HOUR: BN = new BN(3600);
const SECONDS_PER_6H: BN = SECONDS_PER_HOUR.muln(6);
const SECONDS_PER_DAY: BN = SECONDS_PER_HOUR.muln(24);
const SECONDS_PER_WEEK: BN = SECONDS_PER_DAY.muln(7);
const SECONDS_PER_YEAR: BN = new BN(31_536_000);
const MAX_ACCRUAL_SECONDS: BN = SECONDS_PER_WEEK;

const FAUCET_PROGRAM_ID = new PublicKey(
  "4bXpkKSV8swHSnwqtzuboGPaPDeEgAn4Vt8GfarV5rZt"
);

let wallet: Wallet | MathWallet | SolongWallet;
let assets: AssetStore | null;
let program: anchor.Program | null;
let market: Market;
let idl: any;
WALLET.subscribe(data => wallet = data);
ASSETS.subscribe(data => assets = data);
PROGRAM.subscribe(data => program = data);
MARKET.subscribe(data => market = data);

// Development environment variable
export const inDevelopment: boolean = true;

// Rollbar error logging
export const rollbar = new Rollbar({
  accessToken: 'e29773335de24e1f8178149992226c5e',
  captureUncaught: true,
  captureUnhandledRejections: true,
  payload: {
    environment: inDevelopment ? 'development' : 'production'
  }
});

// Firebase initialize
const firebase = initializeApp({
  apiKey: "AIzaSyBaZt58dCRzNFPNu1uYGqM1BIhngSnY3Tg",
  authDomain: "jet-protocol.firebaseapp.com",
  projectId: "jet-protocol",
  storageBucket: "jet-protocol.appspot.com",
  messagingSenderId: "676105669233",
  appId: "1:676105669233:web:46dacd09dc1fab335769d0",
  measurementId: "G-F88EN9GEQX"
});

// Cast solana injected window type
const solWindow = window as unknown as SolWindow;

// Firestore database
export const firestore = getFirestore(firebase);

// Establish Anchor variables
let connection: anchor.web3.Connection;
let coder: anchor.Coder;

// Fetch IDL Locally
const getIDLLocal = async (): Promise<any> => {
  console.log("Fetching IDL from /idl")
  const resp = await fetch('/idl', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' }
  });

  return await resp.json();
};

// Fetch IDL from Firebase
const getIDLFirebase = async (): Promise<any | undefined> => {
  console.log("Fetching IDL from Firebase");
  const docRef = doc(firestore, 'anchor/program');
  const idlDoc = await getDoc(docRef);
  const idl = idlDoc.data()?.idl;
  return JSON.parse(idl);
};

// Get IDL and market data
export const getMarketAndIDL = async (): Promise<void> => {
  // Fetch IDL
  if (jetDev) {
    console.log("In development");
  }
  idl = jetDev ? await getIDLLocal() : await getIDLFirebase();
  const idlMetadata = parseIdlMetadata(idl.metadata);

  // Establish web3 connection
  connection = new anchor.web3.Connection(idlMetadata.cluster, (anchor.Provider.defaultOptions()).commitment);
  coder = new anchor.Coder(idl);

  // Setup reserve structures
  const reserves: Reserves = {} as Reserves;
  for (const reserveMeta of idlMetadata.reserves) {
    let reserve: Reserve = {
      name: reserveMeta.name,
      abbrev: reserveMeta.abbrev,
      marketSize: TokenAmount.zero(reserveMeta.decimals),
      outstandingDebt: TokenAmount.zero(reserveMeta.decimals),
      utilizationRate: 0,
      depositAPY: 0,
      borrowAPR: 0,
      maximumLTV: 0,
      liquidationPremium: 0,
      price: 0,
      decimals: reserveMeta.decimals,
      depositNoteExchangeRate: new BN(0),
      loanNoteExchangeRate: new BN(0),
      accruedUntil: new BN(0),

      accountPubkey: reserveMeta.accounts.reserve,
      vaultPubkey: reserveMeta.accounts.vault,
      availableLiquidity: TokenAmount.zero(reserveMeta.decimals),
      feeNoteVaultPubkey: reserveMeta.accounts.feeNoteVault,
      tokenMintPubkey: reserveMeta.accounts.tokenMint,
      tokenMint: TokenAmount.zero(reserveMeta.decimals),
      faucetPubkey: reserveMeta.accounts.faucet, 
      depositNoteMintPubkey: reserveMeta.accounts.depositNoteMint,
      depositNoteMint: TokenAmount.zero(reserveMeta.decimals),
      loanNoteMintPubkey: reserveMeta.accounts.loanNoteMint,
      loanNoteMint: TokenAmount.zero(reserveMeta.decimals),
      pythPricePubkey: reserveMeta.accounts.pythPrice,
      pythProductPubkey:  reserveMeta.accounts.pythProduct,
    };
    reserves[reserveMeta.abbrev] = reserve;
  }

  // Set market
  MARKET.set({
    minColRatio: 0,
    accountPubkey: idlMetadata.market.market,
    authorityPubkey: idlMetadata.market.marketAuthority,
    reserves: reserves,
  });

  // Set initial current asset to SOL
  CURRENT_RESERVE.set(reserves[0]);

  // Subscribe to market 
  subscribeToMarket(idlMetadata, connection, coder);
};

// Connect to user's wallet
export const getWalletAndAnchor = async (provider: WalletProvider): Promise<void> => {
  // Wallet adapter or injected wallet setup
  if (provider.name === 'Phantom' && solWindow.solana) {
    wallet = new WalletAdapter(solWindow.solana) as Wallet;
  } else if (provider.name === 'Math Wallet' && solWindow.solana.isMathWallet) {
    wallet = solWindow.solana as unknown as MathWallet;
    wallet.publicKey = new anchor.web3.PublicKey(await solWindow.solana.getAccount());
  } else if (provider.name === 'Solong' && solWindow.solong) {
    wallet = solWindow.solong as unknown as SolongWallet;
    wallet.publicKey = new anchor.web3.PublicKey(await solWindow.solong.selectAccount());
  } else {
    wallet = new WalletAdapter(provider.url) as Wallet;
  };

  // Set wallet
  wallet.name = provider.name;
  WALLET.set(wallet);

  // Setup anchor program
  anchor.setProvider(new anchor.Provider(
    connection,
    wallet as unknown as anchor.Wallet,
    anchor.Provider.defaultOptions()
  ));
  program = new anchor.Program(idl, (new anchor.web3.PublicKey(idl.metadata.address)));
  PROGRAM.set(program);

  // Connect and begin fetching account data
  // Check for newly created token accounts on interval
  if (wallet.name === 'Math Wallet' || wallet.name === 'Solong') {
    await getAssetPubkeys();
    await findWalletTokenAccounts();
    await subscribeToAssets(connection, coder, wallet.publicKey);
  } else {
    wallet.on('connect', async () => {
      await getAssetPubkeys();
      await findWalletTokenAccounts();
      await subscribeToAssets(connection, coder, wallet.publicKey);
    });
    wallet.connect();
  }

  return;
};

// Get user token accounts
const getAssetPubkeys = async (): Promise<void> => {
  if (program == null || wallet.publicKey == null) {
    return;
  }

  let [obligationPubkey, obligationBump] = await findObligationAddress(program, wallet.publicKey);

  let assetStore: AssetStore = {
    sol: new TokenAmount(new BN(0), SOL_DECIMALS),
    obligationPubkey,
    obligationBump,
    tokens: {}
  } as AssetStore;
  for (const assetAbbrev in market.reserves) {
    let reserve = market.reserves[assetAbbrev];
    let tokenMintPubkey = reserve.tokenMintPubkey;

    let [depositNoteDestPubkey, depositNoteDestBump] = await findDepositNoteDestAddress(program, reserve.accountPubkey, wallet.publicKey);
    let [depositNotePubkey, depositNoteBump] = await findDepositNoteAddress(program, reserve.accountPubkey, wallet.publicKey);
    let [loanNotePubkey, loanNoteBump] = await findLoanNoteAddress(program, reserve.accountPubkey, obligationPubkey, wallet.publicKey);
    let [collateralPubkey, collateralBump] = await findCollateralAddress(program, reserve.accountPubkey, obligationPubkey, wallet.publicKey);

    let asset: Asset = {
      tokenMintPubkey,
      walletTokenExists: false,
      walletTokenBalance: TokenAmount.zero(reserve.decimals),
      depositNotePubkey,
      depositNoteBump,
      depositNoteExists: false,
      depositNoteBalance: TokenAmount.zero(reserve.decimals),
      depositBalance: TokenAmount.zero(reserve.decimals),
      depositNoteDestPubkey,
      depositNoteDestBump,
      depositNoteDestExists: false,
      depositNoteDestBalance: TokenAmount.zero(reserve.decimals),
      loanNotePubkey,
      loanNoteBump,
      loanNoteExists: false,
      loanNoteBalance: TokenAmount.zero(reserve.decimals),
      loanBalance: TokenAmount.zero(reserve.decimals),
      collateralNotePubkey: collateralPubkey,
      collateralNoteBump: collateralBump,
      collateralNoteExists: false,
      collateralNoteBalance: TokenAmount.zero(reserve.decimals),
      collateralBalance: TokenAmount.zero(reserve.decimals),
    };
    assetStore.tokens[assetAbbrev] = asset;

    let accounts: {
      pubkey: PublicKey;
      account: AccountInfo<Buffer>;
    }[] | undefined;
    try {
      accounts = (await connection.getTokenAccountsByOwner(wallet.publicKey, { mint: tokenMintPubkey })).value;
    } catch (e) { }

    if (accounts?.length && assetStore) {
      // Get the first token account found
      // We will want something more sophisticated,
      // where the user selects their active account
      asset.walletTokenPubkey = accounts[0].pubkey
      const tokenAccount = parseTokenAccount(accounts[0].account, accounts[0].pubkey).data;
      asset.walletTokenBalance = TokenAmount.tokenAccount(tokenAccount, reserve.decimals);
    }

    // Set asset
    assetStore.tokens[assetAbbrev] = asset;
    ASSETS.set(assetStore);
  }
};

const findWalletTokenAccounts = async () => {
  for (const assetAbbrev in market.reserves) {
    let reserve = market.reserves[assetAbbrev];
    let tokenMintPubkey = reserve.tokenMintPubkey;
    let asset = assets?.tokens[assetAbbrev];

    let accounts: {
      pubkey: PublicKey;
      account: AccountInfo<Buffer>;
    }[] | undefined;
    try {
      accounts = (await connection.getTokenAccountsByOwner(wallet.publicKey, { mint: tokenMintPubkey })).value;
    } catch (e) { }

    // If there are accounts (we'll end up taking the first),
    // and no current walletTokenPubkey for this asset, store it
    if (accounts?.length && asset && !asset.walletTokenPubkey) {
      asset.walletTokenPubkey = accounts[0].pubkey;
    }
  }
};

// Deposit
export const deposit = async (abbrev: string, amount: BN)
  : Promise<[ok: boolean, txid: string | undefined]> => {
  if (!assets || !program) {
    return [false, undefined];
  }

  const [ok, txid] = await refreshOldReserve(abbrev);
  if(!ok) {
    return [false, txid]
  }
  
  let reserve = market.reserves[abbrev];
  let asset = assets.tokens[abbrev];
  let depositSourcePubkey = asset.walletTokenPubkey;

  // Optional signers
  let depositSourceKeypair: Keypair | undefined;

  // Optional instructions
  // Create wrapped sol ixs
  let createTokenAccountIx: TransactionInstruction | undefined;
  let initTokenAccountIx: TransactionInstruction | undefined;
  let closeTokenAccountIx: TransactionInstruction | undefined;

  // Initialize Obligation, deposit notes, collateral notes
  let initObligationIx: TransactionInstruction | undefined;
  let initDepositAccountIx: TransactionInstruction | undefined;
  let initCollateralAccountIx: TransactionInstruction | undefined;

  // When handling SOL, ignore existing wsol accounts and initialize a new wrapped sol account
  if (asset.tokenMintPubkey.equals(NATIVE_MINT)) {

    // Overwrite the deposit source
    // The app will always wrap native sol, ignoring any existing wsol
    depositSourceKeypair = Keypair.generate();
    depositSourcePubkey = depositSourceKeypair.publicKey;

    const rent = await connection.getMinimumBalanceForRentExemption(TokenAccountLayout.span);

    createTokenAccountIx = SystemProgram.createAccount({
      fromPubkey: wallet.publicKey,
      newAccountPubkey: depositSourcePubkey,
      programId: TOKEN_PROGRAM_ID,
      space: TokenAccountLayout.span,
      lamports: parseInt(amount.addn(rent).toString())
    })

    initTokenAccountIx = Token.createInitAccountInstruction(
      TOKEN_PROGRAM_ID,
      NATIVE_MINT,
      depositSourcePubkey,
      wallet.publicKey
    );

    closeTokenAccountIx = Token.createCloseAccountInstruction(
      TOKEN_PROGRAM_ID,
      depositSourcePubkey,
      wallet.publicKey,
      wallet.publicKey,
      []);
  }

  // Create the deposit note dest account if it doesn't exist
  if (!asset.depositNoteExists) {
    initDepositAccountIx = program.instruction.initDepositAccount(asset.depositNoteBump, {
      accounts: {
        market: market.accountPubkey,
        marketAuthority: market.authorityPubkey,

        reserve: reserve.accountPubkey,
        depositNoteMint: reserve.depositNoteMintPubkey,

        depositor: wallet.publicKey,
        depositAccount: asset.depositNotePubkey,

        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      },
    });
  }

  if (!assets.obligation) {
    initObligationIx = initObligation()
  }

  // Obligatory refresh instruction
  const refreshReserveIx = refreshReserve(abbrev);

  const amountArgs = {
    units: { tokens: {} },
    value: amount,
  }
  const depositIx = program.instruction.deposit(asset.depositNoteBump, amountArgs, {
    accounts: {
      market: market.accountPubkey,
      marketAuthority: market.authorityPubkey,

      reserve: reserve.accountPubkey,
      vault: reserve.vaultPubkey,
      depositNoteMint: reserve.depositNoteMintPubkey,

      depositor: wallet.publicKey,
      depositAccount: asset.depositNotePubkey,
      depositSource: depositSourcePubkey,

      tokenProgram: TOKEN_PROGRAM_ID,
    },
  });

  // Initialize the collateral account if it doesn't exist
  if (!asset.collateralNoteExists) {
    initCollateralAccountIx = program.instruction.initCollateralAccount(asset.collateralNoteBump, {
      accounts: {
        market: market.accountPubkey,
        marketAuthority: market.authorityPubkey,

        obligation: assets.obligationPubkey,
        reserve: reserve.accountPubkey,
        depositNoteMint: reserve.depositNoteMintPubkey,

        owner: wallet.publicKey,
        collateralAccount: asset.collateralNotePubkey,

        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      }
    });
  }

  const depositCollateralBumpSeeds = {
    collateralAccount: asset.collateralNoteBump,
    depositAccount: asset.depositNoteBump,
  };
  let depositCollateralIx = program.instruction.depositCollateral(depositCollateralBumpSeeds, amountArgs, {
    accounts: {
      market: market.accountPubkey,
      marketAuthority: market.authorityPubkey,

      reserve: reserve.accountPubkey,

      obligation: assets.obligationPubkey,
      owner: wallet.publicKey,
      depositAccount: asset.depositNotePubkey,
      collateralAccount: asset.collateralNotePubkey,

      tokenProgram: TOKEN_PROGRAM_ID,
    }
  });

  const ix = [
    createTokenAccountIx,
    initTokenAccountIx,
    initDepositAccountIx,
    initObligationIx,
    initCollateralAccountIx,
    refreshReserveIx,
    depositIx,
    depositCollateralIx,
    closeTokenAccountIx
  ].filter(ix => ix) as TransactionInstruction[];
  const signers = [depositSourceKeypair].filter(signer => signer) as Keypair[];

  try {
    return await sendTransaction(program.provider, ix, signers);
  } catch (err) {
    rollbar.error(`Deposit error: ${transactionErrorToString(err)}`);
    console.log(transactionErrorToString(err));
    return [false, undefined];
  }
};

// Withdraw
export const withdraw = async (abbrev: string, amount: BN)
  : Promise<[ok: boolean, txid: string | undefined]> => {
  if (!assets || !program) {
    return [false, undefined];
  }


  const [ok, txid] = await refreshOldReserve(abbrev);
  if(!ok) {
    return [false, txid]
  }

  const reserve = market.reserves[abbrev];
  const asset = assets.tokens[abbrev];

  let withdrawAccount = asset.walletTokenPubkey
  let withdrawKeypair: Keypair | undefined;

  // Close wrapped sol ixs
  let createTokenAccountIx: TransactionInstruction | undefined;
  let closeTokenAccountIx: TransactionInstruction | undefined;
  let initTokenAccountIx: TransactionInstruction | undefined;

  // Create the wallet token account if it doesn't exist
  if (!asset.walletTokenExists || !withdrawAccount) {
    withdrawKeypair = Keypair.generate();
    withdrawAccount = withdrawKeypair.publicKey;

    const rent = await connection.getMinimumBalanceForRentExemption(TokenAccountLayout.span);

    createTokenAccountIx = SystemProgram.createAccount({
      fromPubkey: wallet.publicKey,
      newAccountPubkey: withdrawAccount,
      programId: TOKEN_PROGRAM_ID,
      space: TokenAccountLayout.span,
      lamports: rent
    })

    initTokenAccountIx = Token.createInitAccountInstruction(
      TOKEN_PROGRAM_ID,
      NATIVE_MINT,
      withdrawAccount,
      wallet.publicKey
    );
  }

  // Obligatory refresh instruction
  const refreshReserveIx = refreshReserve(abbrev);

  const amountArgs = {
    units: { tokens: {} },
    value: amount,
  };
  const withdrawCollateralBumps = {
    collateralAccount: asset.collateralNoteBump,
    depositAccount: asset.depositNoteBump,
  };
  const withdrawCollateralIx = program.instruction.withdrawCollateral(withdrawCollateralBumps, amountArgs, {
    accounts: {
      market: market.accountPubkey,
      marketAuthority: market.authorityPubkey,

      reserve: reserve.accountPubkey,

      obligation: assets.obligationPubkey,
      owner: wallet.publicKey,
      depositAccount: asset.depositNotePubkey,
      collateralAccount: asset.collateralNotePubkey,

      tokenProgram: TOKEN_PROGRAM_ID,
    },
  });

  const withdrawIx = program.instruction.withdraw(asset.depositNoteBump, amountArgs, {
    accounts: {
      market: market.accountPubkey,
      marketAuthority: market.authorityPubkey,

      reserve: reserve.accountPubkey,
      vault: reserve.vaultPubkey,
      depositNoteMint: reserve.depositNoteMintPubkey,

      depositor: wallet.publicKey,
      depositAccount: asset.depositNotePubkey,
      withdrawAccount,

      tokenProgram: TOKEN_PROGRAM_ID,
    },
  });

  // If withdrawing SOL, unwrap it first
  if (withdrawAccount && asset.tokenMintPubkey.equals(NATIVE_MINT)) {
    closeTokenAccountIx = Token.createCloseAccountInstruction(
      TOKEN_PROGRAM_ID,
      withdrawAccount,
      wallet.publicKey,
      wallet.publicKey,
      []);
  }

  const ix = [
    createTokenAccountIx,
    initTokenAccountIx,
    refreshReserveIx,
    withdrawCollateralIx,
    withdrawIx,
    closeTokenAccountIx
  ].filter(ix => ix) as TransactionInstruction[];
  const signers = [withdrawKeypair].filter(signer => signer) as Keypair[];

  try {
    return await sendTransaction(program.provider, ix, signers);
  } catch (err) {
    rollbar.error(`Withdraw error: ${transactionErrorToString(err)}`);
    console.log(transactionErrorToString(err));
    return [false, undefined];
  }
};

// Borrow
export const borrow = async (abbrev: string, amount: BN)
  : Promise<[ok: boolean, txid: string | undefined]> => {
  if (!assets || !program) {
    return [false, undefined];
  }

  const [ok, txid] = await refreshOldReserve(abbrev);
  if(!ok) {
    return [false, txid]
  }

  const reserve = market.reserves[abbrev];
  const asset = assets.tokens[abbrev];

  let walletTokenPubkey: PublicKey;
  let walletTokenKeypair: Keypair | undefined;

  // Create token account ixs
  let createTokenAccountIx: TransactionInstruction | undefined;
  let initTokenAccountIx: TransactionInstruction | undefined;

  // Create loan note token ixs
  let createLoanAccountIx: TransactionInstruction | undefined;
  let initLoanAccountIx: TransactionInstruction | undefined;

  // Close account ixs for unwrapping SOL
  let closeTokenAccountIx: TransactionInstruction | undefined;

  // Create the wallet token account if it doesn't exist
  if (asset.walletTokenExists && asset.walletTokenPubkey) {
    walletTokenPubkey = asset.walletTokenPubkey;
  } else {
    walletTokenKeypair = Keypair.generate();
    walletTokenPubkey = walletTokenKeypair.publicKey;

    const rent = await connection.getMinimumBalanceForRentExemption(TokenAccountLayout.span);

    createTokenAccountIx = SystemProgram.createAccount({
      fromPubkey: wallet.publicKey,
      newAccountPubkey: walletTokenPubkey,
      programId: TOKEN_PROGRAM_ID,
      space: TokenAccountLayout.span,
      lamports: rent
    })

    initTokenAccountIx = Token.createInitAccountInstruction(
      TOKEN_PROGRAM_ID,
      NATIVE_MINT,
      walletTokenPubkey,
      wallet.publicKey
    );
  }

  // Create the loan note account if it doesn't exist
  if (!asset.loanNoteExists) {
    const rent = await connection.getMinimumBalanceForRentExemption(TokenAccountLayout.span);

    createLoanAccountIx = SystemProgram.createAccount({
      fromPubkey: wallet.publicKey,
      newAccountPubkey: asset.loanNotePubkey,
      programId: TOKEN_PROGRAM_ID,
      space: TokenAccountLayout.span,
      lamports: rent
    })

    initLoanAccountIx = Token.createInitAccountInstruction(
      TOKEN_PROGRAM_ID,
      reserve.loanNoteMintPubkey,
      asset.loanNotePubkey,
      wallet.publicKey
    );
  }

  // Obligatory refresh instruction
  const refreshReserveIx = refreshReserve(abbrev);

  const borrowIx = program.instruction.borrow(asset.loanNoteBump, amount, {
    accounts: {
      market: market.accountPubkey,
      marketAuthority: market.authorityPubkey,

      obligation: assets.obligationPubkey,
      reserve: reserve.accountPubkey,
      vault: reserve.vaultPubkey,
      loanNoteMint: reserve.loanNoteMintPubkey,

      borrower: wallet.publicKey,
      loanAccount: asset.loanNotePubkey,
      receiverAccount: walletTokenPubkey,

      tokenProgram: TOKEN_PROGRAM_ID,
    },
  });

  // If withdrawing SOL, unwrap it first
  if (asset.tokenMintPubkey.equals(NATIVE_MINT)) {
    closeTokenAccountIx = Token.createCloseAccountInstruction(
      TOKEN_PROGRAM_ID,
      reserve.accountPubkey,
      wallet.publicKey,
      wallet.publicKey,
      []);
  }

  const ix = [
    createTokenAccountIx,
    initTokenAccountIx,
    createLoanAccountIx,
    initLoanAccountIx,
    refreshReserveIx,
    borrowIx,
    closeTokenAccountIx
  ].filter(ix => ix) as TransactionInstruction[];
  const signers = [walletTokenKeypair].filter(signer => signer) as Keypair[];

  try {
    // Make deposit RPC call
    return await sendTransaction(program.provider, ix, signers);
  } catch (err) {
    rollbar.error(`Borrow error: ${transactionErrorToString(err)}`);
    console.log(transactionErrorToString(err));
    return [false, undefined];
  }
};

// Repay
export const repay = async (abbrev: string, amount: BN)
  : Promise<[ok: boolean, txid: string | undefined]> => {
  if (!assets || !program) {
    return [false, undefined];
  }

  const [ok, txid] = await refreshOldReserve(abbrev);
  if(!ok) {
    return [false, txid]
  }

  const reserve = market.reserves[abbrev];
  const asset = assets.tokens[abbrev];

  if (!asset.walletTokenExists || !asset.walletTokenPubkey) {
    return [false, undefined];
  }

  // Obligatory refresh instruction
  const refreshReserveIx = refreshReserve(abbrev);

  const repayIx = program.instruction.repay(amount, {
    accounts: {
      market: market.accountPubkey,
      marketAuthority: market.authorityPubkey,

      obligation: assets.obligationPubkey,
      reserve: reserve.accountPubkey,
      vault: reserve.vaultPubkey,
      loanNoteMint: reserve.loanNoteMintPubkey,

      payer: wallet.publicKey,
      loanAccount: asset.loanNotePubkey,
      payerAccount: asset.walletTokenPubkey,

      tokenProgram: TOKEN_PROGRAM_ID,
    },
  });

  const ix = [
    refreshReserveIx,
    repayIx,
  ].filter(ix => ix) as TransactionInstruction[];

  try {
    return await sendTransaction(program.provider, ix);
  } catch (err) {
    rollbar.error(`Repay error: ${transactionErrorToString(err)}`);
    console.log(transactionErrorToString(err));
    return [false, undefined];
  }
};

export const initObligation = ()
  : TransactionInstruction | undefined => {
  if (!program || !assets) {
    return;
  }

  return program.instruction.initObligation(assets.obligationBump, {
    accounts: {
      market: market.accountPubkey,
      marketAuthority: market.authorityPubkey,

      borrower: wallet.publicKey,
      obligation: assets.obligationPubkey,

      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    },
  });
};

export const refreshOldReserve = async (abbrev: string)
: Promise<[ok: boolean, txid: string | undefined]> => {
  if (!program) {
    return [false, undefined];
  }

  let reserve = market.reserves[abbrev];
  let accruedUntil = reserve.accruedUntil;
  let result: [ok: boolean, txid: string | undefined] = [true, undefined];

  while (accruedUntil && accruedUntil.add(MAX_ACCRUAL_SECONDS).ltn(Date.now() / 1000)) {
    const refreshReserveIx = refreshReserve(abbrev);
    
    const ix = [
      refreshReserveIx
    ].filter(ix => ix) as TransactionInstruction[];
  
    try {
      result = await sendTransaction(program.provider, ix);
    } catch (err) {
      console.log(transactionErrorToString(err));
      return [false, undefined];
    }
    accruedUntil = accruedUntil.add(MAX_ACCRUAL_SECONDS);
  }
  return result;
}

export const refreshReserve = (abbrev: string) => {
  if (!program) {
    return;
  }

  let reserve = market.reserves[abbrev];

  const refreshInstruction = program.instruction.refreshReserve({
    accounts: {
      market: market.accountPubkey,
      marketAuthority: market.authorityPubkey,

      reserve: reserve.accountPubkey,
      feeNoteVault: reserve.feeNoteVaultPubkey,
      depositNoteMint: reserve.depositNoteMintPubkey,

      pythOraclePrice: reserve.pythPricePubkey,
      tokenProgram: TOKEN_PROGRAM_ID,
    },
  });

  return refreshInstruction;
};

// Faucet
export const airdrop = async (abbrev: string, lamports: BN)
  : Promise<[ok: boolean, txid: string | undefined]> => {
  if (program == null || assets == null) {
    return [false, undefined];
  }

  let reserve = market.reserves[abbrev];
  const asset = Object.values(assets.tokens).find(asset => asset.tokenMintPubkey.equals(reserve.tokenMintPubkey));

  if (asset == null) {
    return [false, undefined];
  }

  let tokenPubkey = asset.walletTokenPubkey;

  let ix: TransactionInstruction[] = [];
  let signers: Signer[] = [];

  //optionally create a token account for wallet

  if (!asset.walletTokenExists || !tokenPubkey) {
    const tokenAccount = Keypair.generate();
    tokenPubkey = tokenAccount.publicKey;
    signers.push(tokenAccount);
    
    const createAccountIx = SystemProgram.createAccount({
      programId: TOKEN_PROGRAM_ID,
      newAccountPubkey: tokenPubkey,
      space: TokenAccountLayout.span,
      fromPubkey: wallet.publicKey,
      lamports: await Token.getMinBalanceRentForExemptAccount(connection),
    });
    ix.push(createAccountIx);

    const initTokenAccountIx = Token.createInitAccountInstruction(TOKEN_PROGRAM_ID, reserve.tokenMintPubkey, tokenAccount.publicKey, wallet.publicKey);
    ix.push(initTokenAccountIx);
  }

  if (reserve.tokenMintPubkey.equals(NATIVE_MINT)) {
    // Special case for WSOL
    //sol airdrop
    try {
      // web3js doesn't use BN
      const txid = await connection.requestAirdrop(wallet.publicKey, parseInt(lamports.toString()));
      console.log('Airdrop Transaction ID: ', txid);
      return [true, txid]
    } catch (error) {
      rollbar.error(`Airdrop error: ${transactionErrorToString(error)}`);
      return [false, undefined]
    }
  } else if(reserve.faucetPubkey) {
    //faucet airdrop
    const faucetAirdropIx = await buildFaucetAirdropIx(
      lamports,
      reserve.tokenMintPubkey,
      tokenPubkey,
      reserve.faucetPubkey
    );
    ix.push(faucetAirdropIx);

    const [ok, txid] = await sendTransaction(program.provider, ix, signers);
    return [ok, txid];
  } else {
    // Mint to the destination token account
    const mintToIx = Token.createMintToInstruction(TOKEN_PROGRAM_ID, reserve.tokenMintPubkey, tokenPubkey, wallet.publicKey, [], new u64(lamports.toArray()));
    ix.push(mintToIx);

    const [ok, txid] = await sendTransaction(program.provider, ix, signers);
    return [ok, txid];
  }
};

const buildFaucetAirdropIx = async (
  amount: BN,
  tokenMintPublicKey: PublicKey,
  destinationAccountPubkey: PublicKey,
  faucetPubkey: PublicKey
) => {
  const pubkeyNonce = await PublicKey.findProgramAddress([new TextEncoder().encode("faucet")], FAUCET_PROGRAM_ID);

  const keys = [
    { pubkey: pubkeyNonce[0], isSigner: false, isWritable: false },
    {
      pubkey: tokenMintPublicKey,
      isSigner: false,
      isWritable: true
    },
    { pubkey: destinationAccountPubkey, isSigner: false, isWritable: true },
    { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    { pubkey: faucetPubkey, isSigner: false, isWritable: false }
  ];

  return new TransactionInstruction({
    programId: FAUCET_PROGRAM_ID,
    data: Buffer.from([1, ...amount.toArray("le", 8)]),
    keys
  });
};
