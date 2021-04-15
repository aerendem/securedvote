extern crate timer;

//use std::sync::mpsc::channel; --stuff for lifetime 
use blockchainlib::*;
use std::io;
//const lifetime: u128 = 10000; //constant to declare the lifetime of dapp 
//let voterId: u32; //voterId
fn main () {
    //Have to create a voterId here and change it if necessary after connecting with other nodes

    //difficulty of hash
    let difficulty = 0x000fffffffffffffffffffffffffffff;

    let mut genesis_block = Ballot::new(0, now(), vec![0; 32], 0,362, difficulty);

    genesis_block.vote(0);

    println!("Voted(mined) genesis ballot {:?}", &genesis_block);

    let mut last_hash = genesis_block.hash.clone();

    let mut ballotchain = Ballotchain::new();

    ballotchain.update_with_block(genesis_block).expect("Failed to add genesis ballot");

    let mut ballot = Ballot::new(1, now(), last_hash, 0,362, difficulty);
    //just simple run of vote mechanic to "mine" a ballot and putting 0 as candidateId
    ballot.vote(0);

    println!("Voted(mined) with ballot {:?}", &ballot);

    last_hash = ballot.hash.clone(); //to be assigned to new ballot

    ballotchain.update_with_block(ballot).expect("Failed to add ballot");

    
    let mut input = String::new();
    println!("Would you like to vote ?");
    
    match io::stdin().read_line(&mut input) {
        Ok(n) => {
            let mut ballot = Ballot::new(2, now(), last_hash, 0,365, difficulty);
            ballot.vote(0);
            println!("Voted(mined) with ballot {:?}", &ballot);
        }
        Err(error) => println!("error: {}", error),
    }
    //Lifetime
    /* let timer = timer::Timer::new();
    let (tx, rx) = channel();

    timer.schedule_with_delay(chrono::Duration::seconds(3), move || {
        tx.send(()).unwrap();
    });

    rx.recv().unwrap();

    std::process::exit() //exiting right here */
}
