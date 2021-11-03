pub mod init_obligation {
    use anchor_lang::prelude::*;
    use anchor_lang::Key;
    use crate::state::*;
    # [instruction (bump : u8)]
    pub struct InitializeObligation<'info> {
        /// The relevant market
        # [account (has_one = market_authority)]
        pub market: Loader<'info, Market>,
        /// The market's authority account
        pub market_authority: AccountInfo<'info>,
        /// The user/authority that is responsible for owning this obligation.
        #[account(mut, signer)]
        pub borrower: AccountInfo<'info>,
        /// The new account to track information about the borrower's loan,
        /// such as the collateral put up.
        # [account (init , seeds = [b"obligation" . as_ref () , market . key () . as_ref () , borrower . key . as_ref ()] , bump = bump , space = 8 + std :: mem :: size_of :: < Obligation > () , payer = borrower)]
        pub obligation: Loader<'info, Obligation>,
        pub token_program: AccountInfo<'info>,
        pub system_program: AccountInfo<'info>,
    }
    #[automatically_derived]
    impl<'info> anchor_lang::Accounts<'info> for InitializeObligation<'info>
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
            let borrower: AccountInfo =
                anchor_lang::Accounts::try_accounts(program_id, accounts, ix_data)?;
            let obligation = &accounts[0];
            *accounts = &accounts[1..];
            let token_program: AccountInfo =
                anchor_lang::Accounts::try_accounts(program_id, accounts, ix_data)?;
            let system_program: AccountInfo =
                anchor_lang::Accounts::try_accounts(program_id, accounts, ix_data)?;
            let __anchor_rent = Rent::get()?;
            let obligation = {
                let space = 8 + std::mem::size_of::<Obligation>();
                let payer = borrower.to_account_info();
                let __current_lamports = obligation.to_account_info().lamports();
                if __current_lamports == 0 {
                    let lamports = __anchor_rent.minimum_balance(space);
                    anchor_lang::solana_program::program::invoke_signed(
                        &anchor_lang::solana_program::system_instruction::create_account(
                            payer.to_account_info().key,
                            obligation.to_account_info().key,
                            lamports,
                            space as u64,
                            program_id,
                        ),
                        &[
                            payer.to_account_info(),
                            obligation.to_account_info(),
                            system_program.to_account_info(),
                        ],
                        &[&[
                            b"obligation".as_ref(),
                            market.key().as_ref(),
                            borrower.key.as_ref(),
                            &[bump],
                        ][..]],
                    )?;
                } else {
                    let required_lamports = __anchor_rent
                        .minimum_balance(space)
                        .max(1)
                        .saturating_sub(__current_lamports);
                    if required_lamports > 0 {
                        anchor_lang::solana_program::program::invoke(
                            &anchor_lang::solana_program::system_instruction::transfer(
                                payer.to_account_info().key,
                                obligation.to_account_info().key,
                                required_lamports,
                            ),
                            &[
                                payer.to_account_info(),
                                obligation.to_account_info(),
                                system_program.to_account_info(),
                            ],
                        )?;
                    }
                    anchor_lang::solana_program::program::invoke_signed(
                        &anchor_lang::solana_program::system_instruction::allocate(
                            obligation.to_account_info().key,
                            space as u64,
                        ),
                        &[
                            obligation.to_account_info(),
                            system_program.to_account_info(),
                        ],
                        &[&[
                            b"obligation".as_ref(),
                            market.key().as_ref(),
                            borrower.key.as_ref(),
                            &[bump],
                        ][..]],
                    )?;
                    anchor_lang::solana_program::program::invoke_signed(
                        &anchor_lang::solana_program::system_instruction::assign(
                            obligation.to_account_info().key,
                            program_id,
                        ),
                        &[
                            obligation.to_account_info(),
                            system_program.to_account_info(),
                        ],
                        &[&[
                            b"obligation".as_ref(),
                            market.key().as_ref(),
                            borrower.key.as_ref(),
                            &[bump],
                        ][..]],
                    )?;
                }
                let pa: anchor_lang::Loader<Obligation> =
                    anchor_lang::Loader::try_from_unchecked(program_id, &obligation)?;
                pa
            };
            let (__program_signer, __bump) =
                anchor_lang::solana_program::pubkey::Pubkey::find_program_address(
                    &[
                        b"obligation".as_ref(),
                        market.key().as_ref(),
                        borrower.key.as_ref(),
                    ],
                    program_id,
                );
            if obligation.to_account_info().key != &__program_signer {
                return Err(anchor_lang::__private::ErrorCode::ConstraintSeeds.into());
            }
            if __bump != bump {
                return Err(anchor_lang::__private::ErrorCode::ConstraintSeeds.into());
            }
            if !obligation.to_account_info().is_writable {
                return Err(anchor_lang::__private::ErrorCode::ConstraintMut.into());
            }
            if !__anchor_rent.is_exempt(
                obligation.to_account_info().lamports(),
                obligation.to_account_info().try_data_len()?,
            ) {
                return Err(anchor_lang::__private::ErrorCode::ConstraintRentExempt.into());
            }
            if &market.load()?.market_authority != market_authority.to_account_info().key {
                return Err(anchor_lang::__private::ErrorCode::ConstraintHasOne.into());
            }
            if !borrower.to_account_info().is_writable {
                return Err(anchor_lang::__private::ErrorCode::ConstraintMut.into());
            }
            if true {
                if !borrower.to_account_info().is_signer {
                    return Err(anchor_lang::__private::ErrorCode::ConstraintSigner.into());
                }
            }
            Ok(InitializeObligation {
                market,
                market_authority,
                borrower,
                obligation,
                token_program,
                system_program,
            })
        }
    }
    #[automatically_derived]
    impl<'info> anchor_lang::ToAccountInfos<'info> for InitializeObligation<'info>
    where
        'info: 'info,
    {
        fn to_account_infos(
            &self,
        ) -> Vec<anchor_lang::solana_program::account_info::AccountInfo<'info>> {
            let mut account_infos = ::alloc::vec::Vec::new();
            account_infos.extend(self.market.to_account_infos());
            account_infos.extend(self.market_authority.to_account_infos());
            account_infos.extend(self.borrower.to_account_infos());
            account_infos.extend(self.obligation.to_account_infos());
            account_infos.extend(self.token_program.to_account_infos());
            account_infos.extend(self.system_program.to_account_infos());
            account_infos
        }
    }
    #[automatically_derived]
    impl<'info> anchor_lang::ToAccountMetas for InitializeObligation<'info> {
        fn to_account_metas(
            &self,
            is_signer: Option<bool>,
        ) -> Vec<anchor_lang::solana_program::instruction::AccountMeta> {
            let mut account_metas = ::alloc::vec::Vec::new();
            account_metas.extend(self.market.to_account_metas(None));
            account_metas.extend(self.market_authority.to_account_metas(None));
            account_metas.extend(self.borrower.to_account_metas(Some(true)));
            account_metas.extend(self.obligation.to_account_metas(None));
            account_metas.extend(self.token_program.to_account_metas(None));
            account_metas.extend(self.system_program.to_account_metas(None));
            account_metas
        }
    }
    #[automatically_derived]
    impl<'info> anchor_lang::AccountsExit<'info> for InitializeObligation<'info>
    where
        'info: 'info,
    {
        fn exit(
            &self,
            program_id: &anchor_lang::solana_program::pubkey::Pubkey,
        ) -> anchor_lang::solana_program::entrypoint::ProgramResult {
            anchor_lang::AccountsExit::exit(&self.borrower, program_id)?;
            anchor_lang::AccountsExit::exit(&self.obligation, program_id)?;
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
    pub(crate) mod __client_accounts_initialize_obligation {
        use super::*;
        use anchor_lang::prelude::borsh;
        pub struct InitializeObligation {
            pub market: anchor_lang::solana_program::pubkey::Pubkey,
            pub market_authority: anchor_lang::solana_program::pubkey::Pubkey,
            pub borrower: anchor_lang::solana_program::pubkey::Pubkey,
            pub obligation: anchor_lang::solana_program::pubkey::Pubkey,
            pub token_program: anchor_lang::solana_program::pubkey::Pubkey,
            pub system_program: anchor_lang::solana_program::pubkey::Pubkey,
        }
        impl borsh::ser::BorshSerialize for InitializeObligation
        where
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
                borsh::BorshSerialize::serialize(&self.borrower, writer)?;
                borsh::BorshSerialize::serialize(&self.obligation, writer)?;
                borsh::BorshSerialize::serialize(&self.token_program, writer)?;
                borsh::BorshSerialize::serialize(&self.system_program, writer)?;
                Ok(())
            }
        }
        #[automatically_derived]
        impl anchor_lang::ToAccountMetas for InitializeObligation {
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
                    self.borrower,
                    true,
                ));
                account_metas.push(anchor_lang::solana_program::instruction::AccountMeta::new(
                    self.obligation,
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
                account_metas
            }
        }
    }
    /// Initialize an account that tracks a portfolio of collateral deposits and loans.
    pub fn handler(ctx: Context<InitializeObligation>, _bump: u8) -> ProgramResult {
        let mut obligation = ctx.accounts.obligation.load_init()?;
        obligation.market = ctx.accounts.market.key();
        obligation.owner = *ctx.accounts.borrower.key;
        ::solana_program::log::sol_log("initialized obligation account");
        Ok(())
    }
}
