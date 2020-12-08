// Import from `core` instead of from `std` since we are in no-std mode
use core::result::Result;

// Import heap related library from `alloc`
// https://doc.rust-lang.org/alloc/index.html
use alloc::{collections::btree_set::BTreeSet, vec::Vec};

use blake2b_rs::Blake2bBuilder;

// Import CKB syscalls and structures
// https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/index.html
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes::Bytes, prelude::*},
    error::SysError,
    high_level::{
        load_cell_lock_hash, load_cell_type_hash, load_input, load_script, load_script_hash,
        QueryIter,
    },
    syscalls::load_cell_data,
};

/// Error
#[repr(i8)]
pub enum Error {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    // Add customized errors here...
    InvalidArgument,
    RequireGovernanceMode,
    InvalidNft,
}

impl From<SysError> for Error {
    fn from(err: SysError) -> Self {
        use SysError::*;
        match err {
            IndexOutOfBound => Self::IndexOutOfBound,
            ItemMissing => Self::ItemMissing,
            LengthNotEnough(_) => Self::LengthNotEnough,
            Encoding => Self::Encoding,
            Unknown(err_code) => panic!("unexpected sys error {}", err_code),
        }
    }
}

pub fn validate() -> Result<(), Error> {
    // We will need to extract governance lock from current running script
    let script = load_script()?;
    let args: Bytes = script.args().unpack();
    if args.len() < 32 {
        return Err(Error::InvalidArgument);
    }
    let mut governance_lock_hash = [0u8; 32];
    governance_lock_hash.copy_from_slice(&args[0..32]);

    let mut input_lock_hashes = QueryIter::new(load_cell_lock_hash, Source::Input);
    let governance_mode = input_lock_hashes.any(|lock_hash| lock_hash == governance_lock_hash);

    // To detect if an NFT is newly generated, we will need to first gather
    // NFTs in input cells.
    let nft_data_loader = |index, source| {
        let mut buf = [0u8; 32];
        match load_cell_data(&mut buf[..], 0, index, source) {
            Ok(length) => {
                if length < 32 {
                    Err(SysError::Encoding)
                } else {
                    Ok(buf)
                }
            }
            Err(SysError::LengthNotEnough(_)) => Ok(buf),
            Err(err) => Err(err),
        }
    };
    let consumed_nfts: BTreeSet<[u8; 32]> =
        QueryIter::new(nft_data_loader, Source::GroupInput).collect();

    // In NFT generation, we will need to calculate a hash that includes the output
    // index of the NFT cells. Let's first loop through all output cells to find
    // the indices for all NFTs of the current type.
    let script_hash = load_script_hash()?;
    let output_nft_indices: Vec<usize> = QueryIter::new(
        |index, source| match load_cell_type_hash(index, source) {
            Ok(Some(hash)) => Ok((Some(hash), index)),
            Ok(None) => Ok((None, index)),
            Err(err) => Err(err),
        },
        Source::Output,
    )
    .filter_map(|(current_script_hash, index)| {
        if current_script_hash
            .map(|s| s == script_hash)
            .unwrap_or(false)
        {
            Some(index)
        } else {
            None
        }
    })
    .collect();

    // Now we can loop through each output NFT and validate them:
    // 1. If an NFT is found in consumed_nfts, this will be a transfer operation,
    // no further work is needed.
    // 2. If an NFT is not found in consumed_nfts, first, we need to ensure the
    // script is in governance_mode, since NFT generation is only enabled in
    // governance mode; second, we will validate that the NFT ID is exactly the
    // blake2b hash of the first input of current transaction, and the current
    // output index.
    let first_input = load_input(0, Source::Input)?;
    for nft_index in output_nft_indices {
        let nft_id = nft_data_loader(nft_index, Source::Output)?;
        if !consumed_nfts.contains(&nft_id) {
            if !governance_mode {
                return Err(Error::RequireGovernanceMode);
            }
            let mut blake2b = Blake2bBuilder::new(32)
                .personal(b"ckb-default-hash")
                .build();
            blake2b.update(first_input.as_slice());
            blake2b.update(&(nft_index as u64).to_le_bytes());
            let mut hash = [0u8; 32];
            blake2b.finalize(&mut hash[..]);
            if hash != nft_id {
                return Err(Error::InvalidNft);
            }
        }
    }

    Ok(())
}
