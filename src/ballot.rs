use std::fmt::{ self, Debug, Formatter };
use super::*;

pub struct Ballot {
    pub index: u32, //Index of ballot
    pub timestamp: u128, //The time when ballot created
    pub hash: Hash, //Hashing is the process of taking the input string of any length and turning it into cryptographic fixed output
    pub prev_block_hash: Hash, //hash of previous ballot that currrent ballot chained onto
    pub nonce: u64, //it is a number added to a hashed—or encrypted—block
    pub voted_candidate_id: u32, //Using 32-bit integer to hold integers more then 0
    pub voter_id: u32, //the id of voter, this field will be checked in voting process aswell
    pub difficulty: u128, //difficulty of hash
}

impl Debug for Ballot {
    fn fmt (&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Vote Ballot[{}]: {} created at: {} voted for: {} voted by: {} nonce: {}",
            &self.index,
            &hex::encode(&self.hash),
            &self.timestamp,
            &self.voted_candidate_id,
            &self.voter_id,
            &self.nonce,
        )
    }
}

impl Ballot {
    pub fn new (index: u32, timestamp: u128, prev_block_hash: Hash, voted_candidate_id: u32, voter_id: u32, difficulty: u128) -> Self {
        Ballot {
            index,  
            timestamp, 
            hash: vec![0; 32],
            prev_block_hash,
            nonce: 0,
            voted_candidate_id,
            voter_id,
            difficulty,
        }
    }

    pub fn vote (&mut self, voted_candidate_id: u32) {
        for nonce_attempt in 0..(u64::max_value()) {
            self.nonce = nonce_attempt;
            let hash = self.hash();
            if check_difficulty(&hash, self.difficulty) {
                self.hash = hash;
                self.voted_candidate_id = voted_candidate_id;
                return;
            }
        }
    }

    pub fn candidate_check(candidateId: u32)  {
        
    }

    pub fn voted_check(voterId: u32) { 
        
    }
}

impl Hashable for Ballot {
    fn bytes (&self) -> Vec<u8> {
        let mut bytes = vec![];

        bytes.extend(&u32_bytes(&self.index));
        bytes.extend(&u128_bytes(&self.timestamp));
        bytes.extend(&self.prev_block_hash);
        bytes.extend(&u64_bytes(&self.nonce));
        bytes.extend(&u32_bytes(&self.voted_candidate_id));
        bytes.extend(&u128_bytes(&self.difficulty));

        bytes
    }
}

pub fn check_difficulty (hash: &Hash, difficulty: u128) -> bool {
    difficulty > difficulty_bytes_as_u128(&hash)
}
