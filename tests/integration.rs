#![cfg(feature = "test-bpf")]

use solana_program::hash::Hash;
use solana_program::program_option::COption;
use solana_program::system_instruction;
use solana_program_test::{processor, ProgramTest, ProgramTestContext};
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use token::instruction::TokenInstruction;
use token::entrypoint::process_instruction;
use token::id;
use solana_program::pubkey::Pubkey;
use spl_token::solana_program::program_pack::Pack;
use spl_token::state::{Account, Mint};

struct Env {
    ctx: ProgramTestContext,
    from: Keypair,
    to: Keypair,
}

impl Env {
    const DEPOSIT_AMOUNT: u64 = 5_000_000_000;

    async fn new() -> Env {
        let transfer_program = ProgramTest::new("token", id(), processor!(process_instruction));
        let mut ctx = transfer_program.start_with_context().await;

        let from = Keypair::new();
        let to = Keypair::new();

        let from_deposit_instr = system_instruction::transfer(
            &ctx.payer.pubkey(),
            &from.pubkey(),
            Env::DEPOSIT_AMOUNT,
        );
        let to_deposit_instr = system_instruction::transfer(
            &ctx.payer.pubkey(),
            &to.pubkey(),
            Env::DEPOSIT_AMOUNT,
        );
        let deposit_tx = Transaction::new_signed_with_payer(
            &[from_deposit_instr, to_deposit_instr],
            Some(&ctx.payer.pubkey()),
            &[&ctx.payer],
            ctx.last_blockhash,
        );
        ctx.banks_client.process_transaction(deposit_tx).await.unwrap();

        Env { ctx, from, to }
    }
}

#[tokio::test]
async fn transfer_lamports() {
    let env = Env::new().await;
    let from = env.from;
    let to = env.to;
    let mut ctx = env.ctx;

    let transfer_amount = 1_111_111;
    let from_balance_before_transfer = ctx
        .banks_client
        .get_balance(from.pubkey())
        .await
        .unwrap();
    let to_balance_before_transfer = ctx
        .banks_client
        .get_balance(to.pubkey())
        .await
        .unwrap();
    let (fee_calculator, _, _) = ctx.banks_client.get_fees().await.unwrap();

    let transfer_instr = TokenInstruction::transfer_lamports(
        from.pubkey(),
        to.pubkey(),
        transfer_amount,
    );
    let transfer_tx = Transaction::new_signed_with_payer(
        &[transfer_instr],
        Some(&from.pubkey()),
        &[&from],
        ctx.last_blockhash,
    );
    let transfer_tx_fee = fee_calculator.calculate_fee(transfer_tx.message());
    ctx.banks_client.process_transaction(transfer_tx).await.unwrap();

    let to_balance_after_transfer = ctx
        .banks_client
        .get_balance(to.pubkey())
        .await
        .unwrap();
    assert_eq!(to_balance_after_transfer - to_balance_before_transfer, transfer_amount);

    let from_balance_after_transfer = ctx
        .banks_client
        .get_balance(from.pubkey())
        .await
        .unwrap();
    assert_eq!(from_balance_before_transfer, from_balance_after_transfer + transfer_amount + transfer_tx_fee);
}

#[tokio::test]
async fn transfer_spl_token() {
    let mut env = Env::new().await;
    let mint_env = MintEnv::new(&mut env).await;
    let from = env.from;
    let transfer_amount = MintEnv::MINT_AMOUNT;
    let mut ctx = env.ctx;

    let from_spl_token_acc_before_transfer: Account = ctx.banks_client
        .get_packed_account_data(mint_env.from_spl_token.pubkey())
        .await
        .unwrap();
    let to_spl_token_acc_before_transfer: Account = ctx.banks_client
        .get_packed_account_data(mint_env.to_spl_token.pubkey())
        .await
        .unwrap();

    let transfer_spl_token_instr = TokenInstruction::transfer_spl_token(
        from.pubkey(),
        mint_env.from_spl_token.pubkey(),
        mint_env.to_spl_token.pubkey(),
        transfer_amount,
    );
    let transfer_spl_token_tx = Transaction::new_signed_with_payer(
        &[transfer_spl_token_instr],
        Some(&from.pubkey()),
        &[&from],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(transfer_spl_token_tx).await.unwrap();

    let from_spl_token_acc_after_transfer: Account = ctx.banks_client
        .get_packed_account_data(mint_env.from_spl_token.pubkey())
        .await
        .unwrap();
    let to_spl_token_acc_after_transfer: Account = ctx.banks_client
        .get_packed_account_data(mint_env.to_spl_token.pubkey())
        .await
        .unwrap();

    assert_eq!(from_spl_token_acc_before_transfer.mint, mint_env.minter.pubkey());
    assert_eq!(from_spl_token_acc_before_transfer.amount,
               from_spl_token_acc_after_transfer.amount + transfer_amount);
    assert_eq!(to_spl_token_acc_before_transfer.mint, mint_env.minter.pubkey());
    assert_eq!(to_spl_token_acc_before_transfer.amount,
               to_spl_token_acc_after_transfer.amount - transfer_amount);
}

#[tokio::test]
async fn approve_spl_token() {
    let mut env = Env::new().await;
    let mint_env = MintEnv::new(&mut env).await;
    let from = env.from;
    let transfer_amount = MintEnv::MINT_AMOUNT;
    let mut ctx = env.ctx;
    let from_spl_token = mint_env.from_spl_token;
    let to_spl_token = mint_env.to_spl_token;

    let from_spl_token_acc_before_transfer: Account = ctx.banks_client
        .get_packed_account_data(from_spl_token.pubkey())
        .await
        .unwrap();
    let to_spl_token_acc_before_transfer: Account = ctx.banks_client
        .get_packed_account_data(to_spl_token.pubkey())
        .await
        .unwrap();

    let transfer_spl_token_instr = TokenInstruction::approve_spl_token(
        from.pubkey(),
        from_spl_token.pubkey(),
        to_spl_token.pubkey(),
        transfer_amount,
    );
    let transfer_spl_token_tx = Transaction::new_signed_with_payer(
        &[transfer_spl_token_instr],
        Some(&from.pubkey()),
        &[&from],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(transfer_spl_token_tx).await.unwrap();

    let from_spl_token_acc_after_transfer: Account = ctx.banks_client
        .get_packed_account_data(from_spl_token.pubkey())
        .await
        .unwrap();
    let to_spl_token_acc_after_transfer: Account = ctx.banks_client
        .get_packed_account_data(to_spl_token.pubkey())
        .await
        .unwrap();


    assert_eq!(from_spl_token_acc_before_transfer.mint, mint_env.minter.pubkey());
    assert_eq!(from_spl_token_acc_before_transfer.delegate, COption::None);
    assert_eq!(from_spl_token_acc_before_transfer.delegated_amount, 0);
    assert_eq!(from_spl_token_acc_after_transfer.delegate, COption::Some(to_spl_token.pubkey()));
    assert_eq!(from_spl_token_acc_after_transfer.delegated_amount, transfer_amount);

    assert_eq!(to_spl_token_acc_before_transfer.mint, mint_env.minter.pubkey());
    assert_eq!(to_spl_token_acc_before_transfer.delegate, COption::None);
    assert_eq!(to_spl_token_acc_before_transfer.delegated_amount, 0);
    assert_eq!(to_spl_token_acc_after_transfer.delegate, COption::None);
    assert_eq!(to_spl_token_acc_after_transfer.delegated_amount, 0);
}


struct MintEnv {
    minter: Keypair,
    _mint_authority: Keypair,
    _freeze_authority: Keypair,
    from_spl_token: Keypair,
    to_spl_token: Keypair,
    _decimals: u8,
}

impl MintEnv {
    const MINT_AMOUNT: u64 = 26_000;

    async fn new(env: &mut Env) -> MintEnv {
        let minter = Keypair::new();
        let mint_authority = Keypair::new();
        let freeze_authority = Keypair::new();
        let from_spl_token = Keypair::new();
        let to_spl_token = Keypair::new();
        let decimals = 7;

        Self::initialize_mint(env, &minter, &mint_authority, &freeze_authority, decimals).await;
        Self::init_spl_holders_account(env, &minter, &from_spl_token, &to_spl_token).await;
        Self::mint_spl_token(
            &mut env.ctx,
            &env.from,
            &minter,
            &from_spl_token,
            &mint_authority,
            MintEnv::MINT_AMOUNT,
        ).await;

        MintEnv {
            minter,
            _mint_authority: mint_authority,
            _freeze_authority: freeze_authority,
            from_spl_token,
            to_spl_token,
            _decimals: decimals,
        }
    }

    async fn initialize_mint(
        env: &mut Env,
        minter: &Keypair,
        mint_authority: &Keypair,
        freeze_authority: &Keypair,
        decimals: u8,
    ) {
        let ctx = &mut env.ctx;
        let rent = ctx.banks_client.get_rent().await.unwrap();
        let mint_rent_value = rent.minimum_balance(Mint::LEN);
        let from = &env.from;

        let create_mint_storage_acc_instr = system_instruction::create_account(
            &from.pubkey(),
            &minter.pubkey(),
            mint_rent_value,
            Mint::LEN as u64,
            &spl_token::id(),
        );
        let init_mint_instr = spl_token::instruction::initialize_mint(
            &spl_token::id(),
            &minter.pubkey(),
            &mint_authority.pubkey(),
            Some(&freeze_authority.pubkey()),
            decimals,
        ).unwrap();
        let init_mint_tx = Transaction::new_signed_with_payer(
            &[create_mint_storage_acc_instr, init_mint_instr],
            Some(&from.pubkey()),
            &[from, minter],
            ctx.last_blockhash,
        );

        ctx.banks_client.process_transaction(init_mint_tx).await.unwrap();
    }

    async fn init_spl_holders_account(
        env: &mut Env,
        minter: &Keypair,
        from_spl_token: &Keypair,
        to_spl_token: &Keypair,
    ) {
        let ctx = &mut env.ctx;
        let from = &env.from;
        let to = &env.from;

        let rent = ctx.banks_client.get_rent().await.unwrap();
        let acc_rent_value = rent.minimum_balance(Account::LEN);

        let init_from_spl_holder_acc_tx = Self::init_spl_holder_acc_tx(
            from,
            &from_spl_token,
            &minter.pubkey(),
            &from.pubkey(),
            acc_rent_value,
            ctx.last_blockhash,
        );
        let init_to_spl_holder_acc_tx = Self::init_spl_holder_acc_tx(
            to,
            &to_spl_token,
            &minter.pubkey(),
            &to.pubkey(),
            acc_rent_value,
            ctx.last_blockhash,
        );

        ctx.banks_client.process_transactions(
            vec![init_from_spl_holder_acc_tx, init_to_spl_holder_acc_tx]
        ).await.unwrap();
    }

    fn init_spl_holder_acc_tx(
        payer: &Keypair,
        spl_acc: &Keypair,
        minter: &Pubkey,
        owner: &Pubkey,
        acc_rent_value: u64,
        blockhash: Hash,
    ) -> Transaction {
        let create_spl_token_acc_instr = system_instruction::create_account(
            &payer.pubkey(),
            &spl_acc.pubkey(),
            acc_rent_value,
            Account::LEN as u64,
            &spl_token::id(),
        );
        let init_spl_token_acc_instr = spl_token::instruction::initialize_account(
            &spl_token::id(),
            &spl_acc.pubkey(),
            minter,
            owner,
        ).unwrap();
        Transaction::new_signed_with_payer(
            &[create_spl_token_acc_instr, init_spl_token_acc_instr],
            Some(&payer.pubkey()),
            &[payer, spl_acc],
            blockhash,
        )
    }

    async fn mint_spl_token(
        ctx: &mut ProgramTestContext,
        payer: &Keypair,
        minter: &Keypair,
        spl_token_acc: &Keypair,
        mint_authority: &Keypair,
        amount: u64,
    ) {
        let mint_to_instr = spl_token::instruction::mint_to(
            &spl_token::id(),
            &minter.pubkey(),
            &spl_token_acc.pubkey(),
            &mint_authority.pubkey(),
            &[],
            amount,
        ).unwrap();
        let mint_to_tx = Transaction::new_signed_with_payer(
            &[mint_to_instr],
            Some(&payer.pubkey()),
            &[payer, mint_authority],
            ctx.last_blockhash,
        );
        ctx.banks_client.process_transaction(mint_to_tx).await.unwrap();
    }
}

























