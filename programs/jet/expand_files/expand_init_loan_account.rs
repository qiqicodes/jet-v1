pub mod init_loan_account {
    use anchor_lang::prelude::*;
    use anchor_lang::Key;
    use crate::state::*;
    # [instruction (bump : u8)]
    pub struct InitializeLoanAccount<'info> {
        /// The relevant market this loan is for
        # [account (has_one = market_authority)]
        pub market: Loader<'info, Market>,
        /// The market's authority account
        pub market_authority: AccountInfo<'info>,
        /// The obligation the loan account is used for
        # [account (mut , has_one = market , has_one = owner)]
        pub obligation: Loader<'info, Obligation>,
        /// The reserve that the loan comes from
        # [account (has_one = market , has_one = loan_note_mint)]
        pub reserve: Loader<'info, Reserve>,
        /// The mint for the loan notes being used as loan
        pub loan_note_mint: AccountInfo<'info>,
        /// The user/authority that owns the loan
        #[account(mut, signer)]
        pub owner: AccountInfo<'info>,
        /// The account that will store the loan notes
        # [account (init , seeds = [b"loan" . as_ref () , reserve . key () . as_ref () , obligation . key () . as_ref () , owner . key . as_ref ()] , bump = bump , token :: mint = loan_note_mint , token :: authority = market_authority , payer = owner)]
        pub loan_account: AccountInfo<'info>,
        # [account (address = anchor_spl :: token :: ID)]
        pub token_program: AccountInfo<'info>,
        pub system_program: AccountInfo<'info>,
        pub rent: Sysvar<'info, Rent>,
    }
    #[automatically_derived]
    impl<'info> anchor_lang::Accounts<'info> for InitializeLoanAccount<'info>
    where
        'info: 'info,
    {
        #[inline(never)]
        fn try_accounts(
            program_id: &anchor_lang::solana_program::pubkey::Pubkey,
            accounts: &mut &[anchor_lang::solana_program::account_info::AccountInfo<'info>],
            ix_data: &[u8],
        ) -> std::result::Result<Self, anchor_lang::solana_program::program_error::ProgramError>
        {
            let mut ix_data = ix_data;
            struct __Args {
                bump: u8,
            }
            impl borsh::ser::BorshSerialize for __Args
            where
                u8: borsh::ser::BorshSerialize,
            {
                fn serialize<W: borsh::maybestd::io::Write>(
                    &self,
                    writer: &mut W,
                ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                    borsh::BorshSerialize::serialize(&self.bump, writer)?;
                    Ok(())
                }
            }
            impl borsh::de::BorshDeserialize for __Args
            where
                u8: borsh::BorshDeserialize,
            {
                fn deserialize(
                    buf: &mut &[u8],
                ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
                    Ok(Self {
                        bump: borsh::BorshDeserialize::deserialize(buf)?,
                    })
                }
            }
            let __Args { bump } = __Args::deserialize(&mut ix_data)
                .map_err(|_| anchor_lang::__private::ErrorCode::InstructionDidNotDeserialize)?;
            let market: anchor_lang::Loader<Market> =
                anchor_lang::Accounts::try_accounts(program_id, accounts, ix_data)?;
            let market_authority: AccountInfo =
                anchor_lang::Accounts::try_accounts(program_id, accounts, ix_data)?;
            let obligation: anchor_lang::Loader<Obligation> =
                anchor_lang::Accounts::try_accounts(program_id, accounts, ix_data)?;
            let reserve: anchor_lang::Loader<Reserve> =
                anchor_lang::Accounts::try_accounts(program_id, accounts, ix_data)?;
            let loan_note_mint: AccountInfo =
                anchor_lang::Accounts::try_accounts(program_id, accounts, ix_data)?;
            let owner: AccountInfo =
                anchor_lang::Accounts::try_accounts(program_id, accounts, ix_data)?;
            let loan_account = &accounts[0];
            *accounts = &accounts[1..];
            let token_program: AccountInfo =
                anchor_lang::Accounts::try_accounts(program_id, accounts, ix_data)?;
            let system_program: AccountInfo =
                anchor_lang::Accounts::try_accounts(program_id, accounts, ix_data)?;
            let rent: Sysvar<Rent> =
                anchor_lang::Accounts::try_accounts(program_id, accounts, ix_data)?;
            let __anchor_rent = Rent::get()?;
            let loan_account: AccountInfo = {
                let payer = owner.to_account_info();
                let __current_lamports = loan_account.to_account_info().lamports();
                if __current_lamports == 0 {
                    let lamports =
                        __anchor_rent.minimum_balance(anchor_spl::token::TokenAccount::LEN);
                    anchor_lang::solana_program::program::invoke_signed(
                        &anchor_lang::solana_program::system_instruction::create_account(
                            payer.to_account_info().key,
                            loan_account.to_account_info().key,
                            lamports,
                            anchor_spl::token::TokenAccount::LEN as u64,
                            token_program.to_account_info().key,
                        ),
                        &[
                            payer.to_account_info(),
                            loan_account.to_account_info(),
                            system_program.to_account_info(),
                        ],
                        &[&[
                            b"loan".as_ref(),
                            reserve.key().as_ref(),
                            obligation.key().as_ref(),
                            owner.key.as_ref(),
                            &[bump],
                        ][..]],
                    )?;
                } else {
                    let required_lamports = __anchor_rent
                        .minimum_balance(anchor_spl::token::TokenAccount::LEN)
                        .max(1)
                        .saturating_sub(__current_lamports);
                    if required_lamports > 0 {
                        anchor_lang::solana_program::program::invoke(
                            &anchor_lang::solana_program::system_instruction::transfer(
                                payer.to_account_info().key,
                                loan_account.to_account_info().key,
                                required_lamports,
                            ),
                            &[
                                payer.to_account_info(),
                                loan_account.to_account_info(),
                                system_program.to_account_info(),
                            ],
                        )?;
                    }
                    anchor_lang::solana_program::program::invoke_signed(
                        &anchor_lang::solana_program::system_instruction::allocate(
                            loan_account.to_account_info().key,
                            anchor_spl::token::TokenAccount::LEN as u64,
                        ),
                        &[
                            loan_account.to_account_info(),
                            system_program.to_account_info(),
                        ],
                        &[&[
                            b"loan".as_ref(),
                            reserve.key().as_ref(),
                            obligation.key().as_ref(),
                            owner.key.as_ref(),
                            &[bump],
                        ][..]],
                    )?;
                    anchor_lang::solana_program::program::invoke_signed(
                        &anchor_lang::solana_program::system_instruction::assign(
                            loan_account.to_account_info().key,
                            token_program.to_account_info().key,
                        ),
                        &[
                            loan_account.to_account_info(),
                            system_program.to_account_info(),
                        ],
                        &[&[
                            b"loan".as_ref(),
                            reserve.key().as_ref(),
                            obligation.key().as_ref(),
                            owner.key.as_ref(),
                            &[bump],
                        ][..]],
                    )?;
                }
                let cpi_program = token_program.to_account_info();
                let accounts = anchor_spl::token::InitializeAccount {
                    account: loan_account.to_account_info(),
                    mint: loan_note_mint.to_account_info(),
                    authority: market_authority.to_account_info(),
                    rent: rent.to_account_info(),
                };
                let cpi_ctx = CpiContext::new(cpi_program, accounts);
                anchor_spl::token::initialize_account(cpi_ctx)?;
                let pa: AccountInfo = loan_account.to_account_info();
                pa
            };
            let (__program_signer, __bump) =
                anchor_lang::solana_program::pubkey::Pubkey::find_program_address(
                    &[
                        b"loan".as_ref(),
                        reserve.key().as_ref(),
                        obligation.key().as_ref(),
                        owner.key.as_ref(),
                    ],
                    program_id,
                );
            if loan_account.to_account_info().key != &__program_signer {
                return Err(anchor_lang::__private::ErrorCode::ConstraintSeeds.into());
            }
            if __bump != bump {
                return Err(anchor_lang::__private::ErrorCode::ConstraintSeeds.into());
            }
            if !loan_account.to_account_info().is_writable {
                return Err(anchor_lang::__private::ErrorCode::ConstraintMut.into());
            }
            if !__anchor_rent.is_exempt(
                loan_account.to_account_info().lamports(),
                loan_account.to_account_info().try_data_len()?,
            ) {
                return Err(anchor_lang::__private::ErrorCode::ConstraintRentExempt.into());
            }
            if &market.load()?.market_authority != market_authority.to_account_info().key {
                return Err(anchor_lang::__private::ErrorCode::ConstraintHasOne.into());
            }
            if !obligation.to_account_info().is_writable {
                return Err(anchor_lang::__private::ErrorCode::ConstraintMut.into());
            }
            if &obligation.load()?.market != market.to_account_info().key {
                return Err(anchor_lang::__private::ErrorCode::ConstraintHasOne.into());
            }
            if &obligation.load()?.owner != owner.to_account_info().key {
                return Err(anchor_lang::__private::ErrorCode::ConstraintHasOne.into());
            }
            if &reserve.load()?.market != market.to_account_info().key {
                return Err(anchor_lang::__private::ErrorCode::ConstraintHasOne.into());
            }
            if &reserve.load()?.loan_note_mint != loan_note_mint.to_account_info().key {
                return Err(anchor_lang::__private::ErrorCode::ConstraintHasOne.into());
            }
            if !owner.to_account_info().is_writable {
                return Err(anchor_lang::__private::ErrorCode::ConstraintMut.into());
            }
            if true {
                if !owner.to_account_info().is_signer {
                    return Err(anchor_lang::__private::ErrorCode::ConstraintSigner.into());
                }
            }
            if token_program.to_account_info().key != &anchor_spl::token::ID {
                return Err(anchor_lang::__private::ErrorCode::ConstraintAddress.into());
            }
            Ok(InitializeLoanAccount {
                market,
                market_authority,
                obligation,
                reserve,
                loan_note_mint,
                owner,
                loan_account,
                token_program,
                system_program,
                rent,
            })
        }
    }
    #[automatically_derived]
    impl<'info> anchor_lang::ToAccountInfos<'info> for InitializeLoanAccount<'info>
    where
        'info: 'info,
    {
        fn to_account_infos(
            &self,
        ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
            let mut account_infos = ::alloc::vec::Vec::new();
            account_infos.extend(self.market.to_account_infos());
            account_infos.extend(self.market_authority.to_account_infos());
            account_infos.extend(self.obligation.to_account_infos());
            account_infos.extend(self.reserve.to_account_infos());
            account_infos.extend(self.loan_note_mint.to_account_infos());
            account_infos.extend(self.owner.to_account_infos());
            account_infos.extend(self.loan_account.to_account_infos());
            account_infos.extend(self.token_program.to_account_infos());
            account_infos.extend(self.system_program.to_account_infos());
            account_infos.extend(self.rent.to_account_infos());
            account_infos
        }
    }
    #[automatically_derived]
    impl<'info> anchor_lang::ToAccountMetas for InitializeLoanAccount<'info> {
        fn to_account_metas(
            &self,
            is_signer: Option<bool>,
        ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
            let mut account_metas = ::alloc::vec::Vec::new();
            account_metas.extend(self.market.to_account_metas(None));
            account_metas.extend(self.market_authority.to_account_metas(None));
            account_metas.extend(self.obligation.to_account_metas(None));
            account_metas.extend(self.reserve.to_account_metas(None));
            account_metas.extend(self.loan_note_mint.to_account_metas(None));
            account_metas.extend(self.owner.to_account_metas(Some(true)));
            account_metas.extend(self.loan_account.to_account_metas(None));
            account_metas.extend(self.token_program.to_account_metas(None));
            account_metas.extend(self.system_program.to_account_metas(None));
            account_metas.extend(self.rent.to_account_metas(None));
            account_metas
        }
    }
    #[automatically_derived]
    impl<'info> anchor_lang::AccountsExit<'info> for InitializeLoanAccount<'info>
    where
        'info: 'info,
    {
        fn exit(
            &self,
            program_id: &anchor_lang::solana_program::pubkey::Pubkey,
        ) -> anchor_lang::solana_program::entrypoint::ProgramResult {
            anchor_lang::AccountsExit::exit(&self.obligation, program_id)?;
            anchor_lang::AccountsExit::exit(&self.owner, program_id)?;
            anchor_lang::AccountsExit::exit(&self.loan_account, program_id)?;
            Ok(())
        }
    }
    /// An internal, Anchor generated module. This is used (as an
    /// implementation detail), to generate a struct for a given
    /// `#[derive(Accounts)]` implementation, where each field is a Pubkey,
    /// instead of an `AccountInfo`. This is useful for clients that want
    /// to generate a list of accounts, without explicitly knowing the
    /// order all the fields should be in.
    ///
    /// To access the struct in this module, one should use the sibling
    /// `accounts` module (also generated), which re-exports this.
    pub(crate) mod __client_accounts_initialize_loan_account {
        use super::*;
        use anchor_lang::prelude::borsh;
        pub struct InitializeLoanAccount {
            pub market: anchor_lang::solana_program::pubkey::Pubkey,
            pub market_authority: anchor_lang::solana_program::pubkey::Pubkey,
            pub obligation: anchor_lang::solana_program::pubkey::Pubkey,
            pub reserve: anchor_lang::solana_program::pubkey::Pubkey,
            pub loan_note_mint: anchor_lang::solana_program::pubkey::Pubkey,
            pub owner: anchor_lang::solana_program::pubkey::Pubkey,
            pub loan_account: anchor_lang::solana_program::pubkey::Pubkey,
            pub token_program: anchor_lang::solana_program::pubkey::Pubkey,
            pub system_program: anchor_lang::solana_program::pubkey::Pubkey,
            pub rent: anchor_lang::solana_program::pubkey::Pubkey,
        }
        impl borsh::ser::BorshSerialize for InitializeLoanAccount
        where
            anchor_lang::solana_program::pubkey::Pubkey: borsh::ser::BorshSerialize,
            anchor_lang::solana_program::pubkey::Pubkey: borsh::ser::BorshSerialize,
            anchor_lang::solana_program::pubkey::Pubkey: borsh::ser::BorshSerialize,
            anchor_lang::solana_program::pubkey::Pubkey: borsh::ser::BorshSerialize,
            anchor_lang::solana_program::pubkey::Pubkey: borsh::ser::BorshSerialize,
            anchor_lang::solana_program::pubkey::Pubkey: borsh::ser::BorshSerialize,
            anchor_lang::solana_program::pubkey::Pubkey: borsh::ser::BorshSerialize,
            anchor_lang::solana_program::pubkey::Pubkey: borsh::ser::BorshSerialize,
            anchor_lang::solana_program::pubkey::Pubkey: borsh::ser::BorshSerialize,
            anchor_lang::solana_program::pubkey::Pubkey: borsh::ser::BorshSerialize,
        {
            fn serialize<W: borsh::maybestd::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
                borsh::BorshSerialize::serialize(&self.market, writer)?;
                borsh::BorshSerialize::serialize(&self.market_authority, writer)?;
                borsh::BorshSerialize::serialize(&self.obligation, writer)?;
                borsh::BorshSerialize::serialize(&self.reserve, writer)?;
                borsh::BorshSerialize::serialize(&self.loan_note_mint, writer)?;
                borsh::BorshSerialize::serialize(&self.owner, writer)?;
                borsh::BorshSerialize::serialize(&self.loan_account, writer)?;
                borsh::BorshSerialize::serialize(&self.token_program, writer)?;
                borsh::BorshSerialize::serialize(&self.system_program, writer)?;
                borsh::BorshSerialize::serialize(&self.rent, writer)?;
                Ok(())
            }
        }
        #[automatically_derived]
        impl anchor_lang::ToAccountMetas for InitializeLoanAccount {
            fn to_account_metas(
                &self,
                is_signer: Option<bool>,
            ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
                let mut account_metas = ::alloc::vec::Vec::new();
                account_metas.push(
                    anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                        self.market,
                        false,
                    ),
                );
                account_metas.push(
                    anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                        self.market_authority,
                        false,
                    ),
                );
                account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new(
                    self.obligation,
                    false,
                ));
                account_metas.push(
                    anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                        self.reserve,
                        false,
                    ),
                );
                account_metas.push(
                    anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                        self.loan_note_mint,
                        false,
                    ),
                );
                account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new(
                    self.owner, true,
                ));
                account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new(
                    self.loan_account,
                    false,
                ));
                account_metas.push(
                    anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                        self.token_program,
                        false,
                    ),
                );
                account_metas.push(
                    anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                        self.system_program,
                        false,
                    ),
                );
                account_metas.push(
                    anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                        self.rent, false,
                    ),
                );
                account_metas
            }
        }
    }
    /// Initialize an account that can be used to store loan notes to represent debt in an obligation
    pub fn handler(ctx: Context<InitializeLoanAccount>, _bump: u8) -> ProgramResult {
        let mut obligation = ctx.accounts.obligation.load_mut()?;
        let reserve = &ctx.accounts.reserve.load()?;
        let account = ctx.accounts.loan_account.key();
        obligation.register_loan(&account, reserve.index)?;
        ::solana_program::log::sol_log("initialized loan account");
        Ok(())
    }
}
