use super::*;

pub struct Candidates {
    pub candidates: Vec<Candidate>,
}

impl Candidates {
    pub fn new() -> Self {
        Candidates {
            candidates: vec![],
        }
    }

    pub fn add_candidate(&mut self, candidate: Candidate) {
        self.candidates.push(candidate);
    }

}
