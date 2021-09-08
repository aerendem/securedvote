use std::fmt::{ self, Debug, Formatter };
use super::*;
use std::collections::HashSet;

pub struct Candidate {
    //pub index: u32, //Index of ballot
    pub name: String,
    pub candidate_id: u32,
    pub vote_count: u32
}

pub struct Candidates {
    pub candidates: Vec<Candidate>,
}

impl Debug for Candidate {
    fn fmt (&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "created candidate with name {}, id {}, with vote count of {}",
            //&self.index,
            &self.name,
            &self.candidate_id,
            &self.vote_count
        )
    }
}

impl Candidate {
    pub fn new (/*index: u32,*/ name: String, candidate_id: u32, vote_count: u32) -> Self {
        Candidate {
            //index,  
            name,
            candidate_id, 
            vote_count
        }

        //Candidates.candidates.insert(1, &Self)
    }

    pub fn write_candidate_vote(&mut self) {
        println!("adayin oy sayisi {}", &self.vote_count);
    }

    pub fn vote (&mut self, candidate_id: u32) {
        self.vote_count += 1;
    }
}