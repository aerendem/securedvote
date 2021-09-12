extern crate timer;

use async_std::task;
use ballotchainlib::*;

fn main() {
    //Have to create a voterId here and change it if necessary after connecting with other nodes
    //building ui
    // //difficulty of hash
    /* let difficulty = 0x000fffffffffffffffffffffffffffff;

    let mut genesis_block = Ballot::new(0, now(), vec![0; 32], 0, 362, difficulty);

    genesis_block.vote(0);

    println!("Voted(mined) genesis ballot {:?}", &genesis_block);

    

    let mut last_hash: Vec<u8> = genesis_block.hash.clone();
    
    ballotchain
        .update_with_block(genesis_block)
        .expect("Failed to add genesis ballot");
 */
    let mut ballotchain = Ballotchain::new();
    task::block_on(async {
        Ballotchain::init_network(&mut ballotchain);
    });
}
