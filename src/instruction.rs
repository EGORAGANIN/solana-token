use borsh::BorshSerialize;
use borsh::BorshDeserialize;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::pubkey::Pubkey;
use solana_program::system_program;
use crate::id;

#[derive(Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum TokenInstruction {
    /// Transfer native Solana token
    /// 0. [signer, writable] - from
    /// 1. [writable] - to
    /// 2. [] - system program process transfer
    TransferLamports { amount: u64 },

    /// Transfer custom token
    /// 0. [signer] - from user account, authority
    /// 1. [writable] - from SPL token account, PDA
    /// 2. [writable] - to SPL token account, PDA
    /// 3. [] - SPL token program
    TransferSplToken { amount: u64 },

    /// Approve custom token
    /// 0. [signer] - from user account, authority
    /// 1. [writable] - from SPL token account, PDA
    /// 2. [writable] - to SPL token account, PDA
    /// 3. [] - SPL token program
    ApproveSplToken { amount: u64 },
}

impl TokenInstruction {
    pub fn transfer_lamports(from: Pubkey, to: Pubkey, amount: u64) -> Instruction {
        let instr = TokenInstruction::TransferLamports { amount };
        Instruction::new_with_borsh(
            id(),
            &instr,
            vec![
                AccountMeta::new(from, true),
                AccountMeta::new(to, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        )
    }

    pub fn transfer_spl_token(
        from: Pubkey,
        from_spl_token: Pubkey,
        to_spl_token: Pubkey,
        amount: u64
    ) -> Instruction {
        let instr = TokenInstruction::TransferSplToken { amount };
        Instruction::new_with_borsh(
            id(),
            &instr,
            vec![
                AccountMeta::new_readonly(from, true),
                AccountMeta::new(from_spl_token, false),
                AccountMeta::new(to_spl_token, false),
                AccountMeta::new_readonly(spl_token::id(), false)
            ],
        )
    }

    pub fn approve_spl_token(
        from: Pubkey,
        from_spl_token: Pubkey,
        to_spl_token: Pubkey,
        amount: u64
    ) -> Instruction {
        let instr = TokenInstruction::ApproveSplToken { amount };
        Instruction::new_with_borsh(
            id(),
            &instr,
            vec![
                AccountMeta::new_readonly(from, true),
                AccountMeta::new(from_spl_token, false),
                AccountMeta::new(to_spl_token, false),
                AccountMeta::new_readonly(spl_token::id(), false)
            ]
        )
    }
}

#[cfg(test)]
mod transfer_instruction_test {
    use borsh::BorshSerialize;
    use borsh::BorshDeserialize;
    use crate::instruction::TokenInstruction;

    const TRANSFER_LAMPORTS: TokenInstruction = TokenInstruction::TransferLamports { amount: 1_234_567 };
    const BINARY_TRANSFER_LAMPORTS: [u8; 9] = [0, 135, 214, 18, 0, 0, 0, 0, 0];

    const TRANSFER_SLP_TOKEN: TokenInstruction = TokenInstruction::TransferSplToken { amount: 1_111_111 };
    const BINARY_TRANSFER_SLP_TOKEN: [u8; 9] = [1, 71, 244, 16, 0, 0, 0, 0, 0];

    const APPROVE_SLP_TOKEN: TokenInstruction = TokenInstruction::ApproveSplToken { amount: 2_222_222 };
    const BINARY_APPROVE_SLP_TOKEN: [u8; 9] = [2, 142, 232, 33, 0, 0, 0, 0, 0];

    #[test]
    fn when_serialization_transfer_lamports_expect_ok() {
        test_serialization(&TRANSFER_LAMPORTS, &BINARY_TRANSFER_LAMPORTS);
    }

    #[test]
    fn when_deserialization_transfer_lamports_expect_ok() {
        test_deserialization(&TRANSFER_LAMPORTS, &BINARY_TRANSFER_LAMPORTS);
    }

    #[test]
    fn when_serialization_transfer_spl_token_expect_ok() {
        test_serialization(&TRANSFER_SLP_TOKEN, &BINARY_TRANSFER_SLP_TOKEN)
    }

    #[test]
    fn when_deserialization_transfer_spl_token_expect_ok() {
        test_deserialization(&TRANSFER_SLP_TOKEN, &BINARY_TRANSFER_SLP_TOKEN)
    }

    #[test]
    fn when_serialization_approve_spl_token_expect_ok() {
        test_serialization(&APPROVE_SLP_TOKEN, &BINARY_APPROVE_SLP_TOKEN)
    }

    #[test]
    fn when_deserialization_approve_spl_token_expect_ok() {
        test_deserialization(&APPROVE_SLP_TOKEN, &BINARY_APPROVE_SLP_TOKEN)
    }

    fn test_serialization(instr: &TokenInstruction, binary_instr: &[u8]) {
        let serialized_instruction = instr.try_to_vec().unwrap();

        assert_eq!(serialized_instruction, binary_instr);
    }

    fn test_deserialization(instr: &TokenInstruction, binary_instr: &[u8]) {
        let deserialized_instr = TokenInstruction::try_from_slice(binary_instr).unwrap();

        assert_eq!(&deserialized_instr, instr);
    }
}