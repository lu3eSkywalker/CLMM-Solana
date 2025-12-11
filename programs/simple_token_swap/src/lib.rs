use anchor_lang::prelude::*;
use anchor_spl::token::{
    self, InitializeAccount, Mint, Token, TokenAccount, TokenAccount as SPLTokenAccount, Transfer,
};

declare_id!("EG889onCgMUMMHhJ4JpnykPDrfH6e8Nx7GM1SZXNqyBX");

#[program]
pub mod Simple_Token_Swap {
    use super::*;

    pub fn initialize_vault_token_a(ctx: Context<InitializeVaultTokenA>) -> Result<()> {
        Ok(())
    }

    pub fn initialize_vault_token_b(ctx: Context<InitializeVaultTokenB>) -> Result<()> {
        Ok(())
    }

    pub fn token_a_deposit_in_pda_vault(ctx: Context<DepositToVaultTokenA>, amount: u64) -> Result<()> {
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        let state = &mut ctx.accounts.user_vault_state_a;
        state.owner = ctx.accounts.user.key();
        state.amount = state.amount.checked_add(amount).unwrap();

        Ok(())
    }

    pub fn token_b_deposit_in_pda_vault(ctx: Context<DepositToVaultTokenB>, amount: u64) -> Result<()> {
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        let state = &mut ctx.accounts.user_vault_state_b;
        state.owner = ctx.accounts.user.key();
        state.amount = state.amount.checked_add(amount).unwrap();

        Ok(())
    }

    pub fn swap_b_for_a(ctx: Context<TokenSwap>, amountOfTokenB: u64) -> Result<()> {
        let token_a_quantity = ctx.accounts.vault_token_a_account.amount;
        let token_b_quantity = ctx.accounts.vault_token_b_account.amount;

        let (x) = amm_calculation(token_a_quantity, token_b_quantity)?;

        let tokenAToSend = (x / ((token_b_quantity as u128) + (amountOfTokenB as u128)))
            .try_into()
            .map_err(|_| error!(TokenSwapError::CalculationError))?;

        let tokenAtoGive = (token_a_quantity as u128)
                .checked_sub(tokenAToSend)
                .ok_or(error!(TokenSwapError::CalculationError))?;

        require!(
            tokenAtoGive <= token_a_quantity as u128,
            TokenSwapError::InsufficientTokenA
        );

        // Transfer Token B from user to Token Vault
        deposit_to_vault_token_b(
            &ctx.accounts.user.to_account_info(),
            &ctx.accounts.user_token_account_for_token_b,
            &ctx.accounts.vault_token_b_account,
            &ctx.accounts.token_program,
            amountOfTokenB,
        )?;

        // Transfer Token A from TokenVault to user
        let mint_a_key = ctx.accounts.mint_a.key(); // prevent temp drop

        let seeds = &[
            b"vault_auth_a",
            mint_a_key.as_ref(),
            &[ctx.bumps.vault_auth_a],
        ];

        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_token_a_account.to_account_info(),
            to: ctx.accounts.user_token_account_for_token_a.to_account_info(),
            authority: ctx.accounts.vault_auth_a.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

        // Convert to u64 before transferring
        let tokenAtoGive: u64 = tokenAtoGive
            .try_into()
            .map_err(|_| error!(TokenSwapError::CalculationError))?;

        token::transfer(cpi_ctx, tokenAtoGive)?;

        Ok(())
    }

    pub fn swap_a_for_b(ctx: Context<TokenSwap>, amountOfTokenA: u64) -> Result<()> {
        let token_a_quantity = ctx.accounts.vault_token_a_account.amount;
        let token_b_quantity = ctx.accounts.vault_token_b_account.amount;

        let (x) = amm_calculation(token_a_quantity, token_b_quantity)?;

        let tokenBtoSend = (x / ((token_a_quantity as u128) + (amountOfTokenA as u128)))
                            .try_into()
                            .map_err(|_| error!(TokenSwapError::CalculationError))?;

        let tokenBtoGive = (token_b_quantity as u128)
            .checked_sub(tokenBtoSend)
            .ok_or(error!(TokenSwapError::CalculationError))?;

        require!(
            tokenBtoGive <= token_b_quantity as u128,
            TokenSwapError::InsufficientTokenB
        );

        // Transfer Token A from user to Token Vault
        deposit_to_vault_token_a(
            &ctx.accounts.user.to_account_info(),
            &ctx.accounts.user_token_account_for_token_a,
            &ctx.accounts.vault_token_a_account,
            &ctx.accounts.token_program,
            amountOfTokenA,
        )?;

        let mint_b_key = ctx.accounts.mint_b.key(); // prevent temp drop

        let seeds = &[
            b"vault_auth_b",
            mint_b_key.as_ref(),
            &[ctx.bumps.vault_auth_b],
        ];

        let signer = &[&seeds[..]];

        // Transfer Token B from TokenVault to user
        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_token_b_account.to_account_info(),
            to: ctx.accounts.user_token_account_for_token_b.to_account_info(),
            authority: ctx.accounts.vault_auth_b.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

        // Convert to u64 before transferring
        let tokenBtoGive: u64 = tokenBtoGive
            .try_into()
            .map_err(|_| error!(TokenSwapError::CalculationError))?;

        token::transfer(cpi_ctx, tokenBtoGive)?;

        Ok(())
    }

    pub fn withdraw_from_vault(ctx: Context<WithdrawFromVault>, amountOfTokenA: u64, amountOfTokenB: u64) -> Result<()> {
        let token_quantity_a = ctx.accounts.vault_token_a_account.amount;
        let token_quantity_b = ctx.accounts.vault_token_b_account.amount;

        require!(
            amountOfTokenA <= token_quantity_a,
            TokenSwapError::InsufficientTokenA
        );

        require!(
            amountOfTokenB <= token_quantity_b,
            TokenSwapError::InsufficientTokenB
        );

        let mint_key_a = ctx.accounts.mint_a.key();
        let mint_key_b = ctx.accounts.mint_b.key();

        let seeds = &[
            b"vault_auth_a",
            mint_key_a.as_ref(),
            &[ctx.bumps.vault_auth_a],
        ];

        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer{
            from: ctx.accounts.vault_token_a_account.to_account_info(),
            to: ctx.accounts.user_token_account_a.to_account_info(),
            authority: ctx.accounts.vault_auth_a.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amountOfTokenA)?;

        let state = &mut ctx.accounts.user_vault_state_a;
        state.amount = state.amount.checked_sub(amountOfTokenA).ok_or(TokenSwapError::InsufficientTokenA)?;


        let seeds2 = &[
            b"vault_auth_b",
            mint_key_b.as_ref(),
            &[ctx.bumps.vault_auth_b],
        ];

        let signer2 = &[&seeds2[..]];

        let cpi_accounts2 = Transfer {
            from: ctx.accounts.vault_token_b_account.to_account_info(),
            to: ctx.accounts.user_token_account_b.to_account_info(),
            authority: ctx.accounts.vault_auth_b.to_account_info(),
        };

        let cpi_program2 = ctx.accounts.token_program.to_account_info();

        let cpi_ctx2 = CpiContext::new_with_signer(cpi_program2, cpi_accounts2, signer2);
        token::transfer(cpi_ctx2, amountOfTokenB)?;

        let state2 = &mut ctx.accounts.user_vault_state_b;
        state2.amount = state2.amount.checked_sub(amountOfTokenB).ok_or(TokenSwapError::InsufficientTokenB)?;

        Ok(())
    }
}

fn amm_calculation(token_a_quantity: u64, token_b_quantity: u64) -> Result<(u128)> {
    let token_a_128 = token_a_quantity as u128;
    let token_b_128 = token_b_quantity as u128;

    let x = token_a_128
        .checked_mul(token_b_128)
        .ok_or_else(|| error!(TokenSwapError::CalculationError))?;

    Ok(x)
}

// This function deposits the Token A from user to the token vault
fn deposit_to_vault_token_a<'info>(
    user: &AccountInfo<'info>,
    user_token_account_for_token_a: &Account<'info, TokenAccount>,
    vault_token_a_account: &Account<'info, TokenAccount>,
    token_program: &Program<'info, Token>,
    amount: u64,
) -> Result<()> {
    let cpi_accounts = Transfer {
        from: user_token_account_for_token_a.to_account_info(),
        to: vault_token_a_account.to_account_info(),
        authority: user.clone(),
    };

    let cpi_program = token_program.to_account_info();

    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, amount)?;

    Ok(())
}

// This function deposits the Token B from user to the token vault
fn deposit_to_vault_token_b<'info>(
    user: &AccountInfo<'info>,
    user_token_account_for_token_b: &Account<'info, TokenAccount>,
    vault_token_b_account: &Account<'info, TokenAccount>,
    token_program: &Program<'info, Token>,
    amount: u64,
) -> Result<()> {
    let cpi_accounts = Transfer {
        from: user_token_account_for_token_b.to_account_info(),
        to: vault_token_b_account.to_account_info(),
        authority: user.to_account_info(),
    };

    let cpi_program = token_program.to_account_info();

    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, amount)?;

    Ok(())
}

#[derive(Accounts)]
#[instruction()]
pub struct InitializeVaultTokenA<'info> {
    #[account(
        init_if_needed,
        seeds = [b"vaultTokenA", mint.key().as_ref()],
        bump,
        payer = payer,
        token::mint = mint,
        token::authority = vault_auth
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    /// CHECK: PDA will be the authority for the vault PDA
    #[account{
        seeds = [b"vault_auth_a", mint.key().as_ref()],
        bump
    }]
    pub vault_auth: AccountInfo<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}


#[derive(Accounts)]
pub struct DepositToVaultTokenA<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"vaultTokenA", mint.key().as_ref()],
        bump
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 32 + 32 + 8,
        seeds = [
            b"user_vault_token_a",
            user.key().as_ref(),
            mint.key().as_ref()
        ],
        bump
    )]
    pub user_vault_state_a: Account<'info, UserVaultState>,

    pub mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction()]
pub struct InitializeVaultTokenB<'info> {
    #[account(
        init_if_needed,
        seeds = [b"vaultTokenB", mint.key().as_ref()],
        bump,
        payer = payer,
        token::mint = mint,
        token::authority = vault_auth
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    /// CHECK: PDA will be the authority for the vault PDAs
    #[account{
        seeds = [b"vault_auth_b", mint.key().as_ref()],
        bump
    }]
    pub vault_auth: AccountInfo<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct DepositToVaultTokenB<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"vaultTokenB", mint.key().as_ref()],
        bump
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 32 + 32 + 8,
        seeds = [
            b"user_vault_token_b",
            user.key().as_ref(),
            mint.key().as_ref()
        ],
        bump
    )]
    pub user_vault_state_b: Account<'info, UserVaultState>,

    pub mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TokenSwap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub user_token_account_for_token_a: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_account_for_token_b: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"vaultTokenA", mint_a.key().as_ref()],
        bump
    )]
    pub vault_token_a_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"vaultTokenB", mint_b.key().as_ref()],
        bump
    )]
    pub vault_token_b_account: Account<'info, TokenAccount>,

    /// CHECK: This is just a signer PDA, no data
    #[account(
        seeds = [b"vault_auth_a", mint_a.key().as_ref()],
        bump
    )]
    pub vault_auth_a: AccountInfo<'info>,

    /// CHECK: This is just a signer PDA, no data
    #[account(
        seeds = [b"vault_auth_b", mint_b.key().as_ref()],
        bump
    )]
    pub vault_auth_b: AccountInfo<'info>,

    pub mint_a: Account<'info, Mint>,

    pub mint_b: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct WithdrawFromVault<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub user_token_account_a: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_account_b: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [
            b"user_vault_token_a",
            user.key().as_ref(),
            mint_a.key().as_ref(),
        ],
        bump,
    )]
    pub user_vault_state_a: Account<'info, UserVaultState>,

    #[account(
        mut,
        seeds = [
            b"user_vault_token_b",
            user.key().as_ref(),
            mint_b.key().as_ref(),
        ],
        bump,
    )]
    pub user_vault_state_b: Account<'info, UserVaultState>,

    #[account(
        mut,
        seeds = [b"vaultTokenA", mint_a.key().as_ref()],
        bump
    )]
    pub vault_token_a_account: Account<'info, TokenAccount>,

    /// CHECK: This is just a signer PDA, no data
    #[account(
        seeds = [b"vault_auth_a", mint_a.key().as_ref()],
        bump
    )]
    pub vault_auth_a: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"vaultTokenB", mint_b.key().as_ref()],
        bump
    )]
    pub vault_token_b_account: Account<'info, TokenAccount>,

    /// CHECK: This is just a signer PDA, no data
    #[account(
        seeds = [b"vault_auth_b", mint_b.key().as_ref()],
        bump
    )]
    pub vault_auth_b: AccountInfo<'info>,

    pub mint_a: Account<'info, Mint>,

    pub mint_b: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
}

#[account]
pub struct UserVaultState {
    pub owner: Pubkey,
    pub amount: u64,
}

#[error_code]
pub enum TokenSwapError {
    #[msg("Insufficient amount of token A in the liquidity pool")]
    InsufficientTokenA,

    #[msg("Insufficient amount of token B in the liquidity pool")]
    InsufficientTokenB,

    #[msg("Multiplication overflow in calculation error")]
    CalculationError
}