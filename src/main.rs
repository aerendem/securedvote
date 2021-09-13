extern crate timer;

use async_std::task;
use ballotchainlib::{Candidate, ballotchain::Ballotchain, candidates::Candidates};

fn main() {
    //Creating ballotchain(blockhain)
    let mut ballotchain = Ballotchain::new();

    //Creating a candidate list to hold all candidates
    let mut candidatelist: Candidates = Candidates::new();

    let birinci_aday = Candidate::new("Mehmet Seçilenleroğlu".to_string(), 1, 0);
    
    let ikinci_aday = Candidate::new("Ahmet Seçilmez".to_string(), 2, 0);

    let ucuncu_aday = Candidate::new("Veli Katılmamış".to_string(), 3, 0);

    candidatelist.add_candidate(birinci_aday);
    candidatelist.add_candidate(ikinci_aday);

    task::block_on(async {
        Ballotchain::init_network(&mut ballotchain, &mut candidatelist);
    });
}
