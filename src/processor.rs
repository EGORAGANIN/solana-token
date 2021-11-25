use borsh::BorshDeserialize;
use solana_program::account_info::{AccountInfo, next_account_info};
use solana_program::entrypoint::ProgramResult;
use solana_program::{msg, system_instruction};
use solana_program::program::invoke;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use crate::error::TransferError;
use crate::instruction::TokenInstruction;


pub struct Processor;

impl Processor {

    pub fn process(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        input: &[u8],
    ) -> ProgramResult {
        msg!("input {:?}", input);
        let instr = TokenInstruction::try_from_slice(input)?;
        match instr {
            TokenInstruction::TransferLamports { amount } => Self::transfer_lamports(accounts, amount),
            TokenInstruction::TransferSplToken { amount } => Self::transfer_spl_token(accounts, amount),
            TokenInstruction::ApproveSplToken { amount } => Self::approve_spl_token(accounts, amount),
        }
    }

    fn transfer_lamports(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        let acc_iter = &mut accounts.iter();
        let from_acc = next_account_info(acc_iter)?;
        let to_acc = next_account_info(acc_iter)?;
        msg!("Transfer lamports from={:?}, to={:?}, amount={}", from_acc.key, to_acc.key, amount);

        if !from_acc.is_signer {
            return Err(ProgramError::MissingRequiredSignature)
        }
        if !from_acc.is_writable {
            return Err(TransferError::AccountNonWritable.into())
        }
        if !to_acc.is_writable {
            return Err(TransferError::AccountNonWritable.into())
        }

        let transfer_instr = system_instruction::transfer(
            from_acc.key,
            to_acc.key,
            amount,
        );
        invoke(
            &transfer_instr,
            &[from_acc.clone(), to_acc.clone()],
        )?;

        msg!("Transfer lamports from={:?}, to={:?}, amount={} done", from_acc.key, to_acc.key, amount);
        Ok(())
    }

    fn transfer_spl_token(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        let acc_iter = &mut accounts.iter();
        let owner_acc = next_account_info(acc_iter)?;
        let from_spl_token_acc = next_account_info(acc_iter)?;
        let to_spl_token_acc = next_account_info(acc_iter)?;
        let spl_token_acc = next_account_info(acc_iter)?;
        msg!(
            "Transfer spl token from={:?}, to={:?}, amount={}",
            from_spl_token_acc.key, to_spl_token_acc.key, amount
        );

        if !owner_acc.is_signer {
            return Err(ProgramError::MissingRequiredSignature)
        }
        if !from_spl_token_acc.is_writable {
            return Err(TransferError::AccountNonWritable.into())
        }
        if !to_spl_token_acc.is_writable {
            return Err(TransferError::AccountNonWritable.into())
        }

        let transfer_instr = spl_token::instruction::transfer(
            spl_token_acc.key,
            from_spl_token_acc.key,
            to_spl_token_acc.key,
            owner_acc.key,
            &[&owner_acc.key],
            amount,
        )?;
        invoke(&transfer_instr,
               &[
                   owner_acc.clone(),
                   from_spl_token_acc.clone(),
                   to_spl_token_acc.clone(),
                   spl_token_acc.clone()
               ],
        )?;

        msg!(
            "Transfer spl token from={:?}, to={:?}, amount={} done",
            from_spl_token_acc.key, to_spl_token_acc.key, amount
        );
        Ok(())
    }

    fn approve_spl_token(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
        let acc_iter = &mut accounts.iter();
        let owner_acc = next_account_info(acc_iter)?;
        let from_spl_token_acc = next_account_info(acc_iter)?;
        let to_spl_token_acc = next_account_info(acc_iter)?;
        let spl_token_acc = next_account_info(acc_iter)?;
        msg!(
            "Approve spl token from={:?}, to={:?}, amount={}",
            from_spl_token_acc.key,
            to_spl_token_acc.key,
            amount
        );

        if !owner_acc.is_signer {
            return Err(ProgramError::MissingRequiredSignature)
        }
        if !from_spl_token_acc.is_writable {
            return Err(TransferError::AccountNonWritable.into())
        }
        if !to_spl_token_acc.is_writable {
            return Err(TransferError::AccountNonWritable.into())
        }

        let approve_instr = spl_token::instruction::approve(
            spl_token_acc.key,
            from_spl_token_acc.key,
            to_spl_token_acc.key,
            owner_acc.key,
            &[owner_acc.key],
            amount,
        )?;
        invoke(
            &approve_instr,
            &[
                owner_acc.clone(),
                from_spl_token_acc.clone(),
                to_spl_token_acc.clone(),
                spl_token_acc.clone()
            ],
        )?;

        msg!(
            "Approve spl token from={:?}, to={:?}, amount={} done",
            from_spl_token_acc.key,
            to_spl_token_acc.key,
            amount
        );
        Ok(())
    }
}