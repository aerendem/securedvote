use std::fmt::{ self, Debug, Formatter };
use super::*;

//Candidate struct
pub struct Candidate {
    pub name: String,
    pub candidate_id: u32,
    pub vote_count: u32
}

impl Debug for Candidate {
    
    //Candidate format function to display
    fn fmt (&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Adayın ismi {}, idsi {} ve sahip olduğu oy sayısı {}",
            &self.name,
            &self.candidate_id,
            &self.vote_count
        )
    }
}

impl Candidate {
    pub fn new (name: String, candidate_id: u32, vote_count: u32) -> Self {
        Candidate {
            name,
            candidate_id, 
            vote_count
        }
    }

    pub fn output_candidate_vote(&mut self) {
        println!("adayin oy sayisi {}", &self.vote_count);
    }
}