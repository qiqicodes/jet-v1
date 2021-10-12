import type * as anchor from '@project-serum/anchor';
import type { Market, User, Copilot, CustomProgramError, IdlMetadata } from './models/JetTypes';
import { writable } from 'svelte/store';

// Overall app init
export const INIT_FAILED = writable<boolean> (false);

// Market and User
export const MARKET = writable<Market>({reserves: {}} as Market);
export const USER = writable<User>({preferredLanguage: 'en'} as User)
export const COPILOT = writable<Copilot | null> (null);

// Program
export const PROGRAM = writable<anchor.Program | null> (null);
export const CUSTOM_PROGRAM_ERRORS = writable<CustomProgramError[]> ([]);
export const ANCHOR_WEB3_CONNECTION = writable<anchor.web3.Connection> (undefined);
export const ANCHOR_CODER = writable<anchor.Coder> (undefined);
export const IDL_METADATA = writable<IdlMetadata> (undefined);