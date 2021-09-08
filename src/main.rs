extern crate timer;

use async_std::task;
use ballotchainlib::*;
use std::io;
use std::thread;
use winit::event_loop;

fn main() {
    //Have to create a voterId here and change it if necessary after connecting with other nodes
    //building ui
    // //difficulty of hash
    let difficulty = 0x000fffffffffffffffffffffffffffff;

    let mut genesis_block = Ballot::new(0, now(), vec![0; 32], 0, 362, difficulty);

    genesis_block.vote(0);

    println!("Voted(mined) genesis ballot {:?}", &genesis_block);

    let mut last_hash = genesis_block.hash.clone();

    let mut ballotchain = Ballotchain::new();

    ballotchain
        .update_with_block(genesis_block)
        .expect("Failed to add genesis ballot");

    /* println!("Would you like to vote ?");
    let mut adayatakan = Candidate::new(String::from("Atakan"), 1, 31);

    Candidate::write_candidate_vote(&mut adayatakan); */

    

    task::block_on(async {
         //Ballotchain::init_network();
         let appp = SecVApp::default();
         let native_options = eframe::NativeOptions::default();
         eframe::run_native(Box::new(appp), native_options);

         SecVApp::change_last_hash(last_hash);
    });

    task::block_on(async {
        Ballotchain::init_network(&ballotchain);
    });

}
