use super::*;
use std::collections::HashSet;

#[derive(Debug)]
pub enum BlockValidationErr {
    MismatchedIndex,
    InvalidHash,
    AchronologicalTimestamp,
    MismatchedPreviousHash,
    InvalidGenesisBlockFormat,
    InvalidInput,
    InsufficientInputValue,
}

pub struct Ballotchain {
    pub blocks: Vec<Ballot>,
}

impl Ballotchain {
    pub fn new () -> Self {
        Ballotchain {
            blocks: vec![],
        }
    }

    pub fn update_with_block (&mut self, ballot: Ballot) -> Result<(), BlockValidationErr> {
        let i = self.blocks.len();

        if ballot.index != i as u32 {
            return Err(BlockValidationErr::MismatchedIndex);
        } else if !ballot::check_difficulty(&ballot.hash(), ballot.difficulty) {
            return Err(BlockValidationErr::InvalidHash);
        } else if i != 0 {
            // Not genesis ballot
            let prev_block = &self.blocks[i - 1];
            if ballot.timestamp <= prev_block.timestamp {
                return Err(BlockValidationErr::AchronologicalTimestamp);
            } else if ballot.prev_block_hash != prev_block.hash {
                return Err(BlockValidationErr::MismatchedPreviousHash);
            }
        } else {
            // Genesis ballot
            if ballot.prev_block_hash != vec![0; 32] {
                return Err(BlockValidationErr::InvalidGenesisBlockFormat);
            }
        }

        self.blocks.push(ballot);

        Ok(())
    }
}
