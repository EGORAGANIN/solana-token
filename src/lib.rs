pub mod instruction;
pub mod processor;
pub mod error;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

solana_program::declare_id!("4ieTTSrJzX1GbW9susJJpLE3bv6kuWqguCjrzYr3jUJ1");