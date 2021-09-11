use super::*;
use async_std::{io, task};
use futures::prelude::*;
use libp2p::kad::handler::KademliaHandler;
use libp2p::kad::record::store::MemoryStore;
use libp2p::kad::{
    record::Key, AddProviderOk, Kademlia, KademliaEvent, PeerRecord, PutRecordOk, QueryResult,
    Quorum, Record,
};
use libp2p::{
    development_transport, identity,
    mdns::{Mdns, MdnsConfig, MdnsEvent},
    swarm::{NetworkBehaviourEventProcess, SwarmEvent},
    NetworkBehaviour, PeerId, Swarm,
};
use pretty_env_logger::*;
use std::any::Any;
use std::collections::HashSet;
use std::{
    error::Error,
    task::{Context, Poll},
};

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
    ///Stores all the ballots(blocks) which are accepted as votes
    pub ballots: Vec<Ballot>,

    ///Stores votes which are not accepted yet
    pub pending_votes: Vec<Ballot>,

    //put reference to network manager here
    //pub kademlia: Kademlia<MemoryStore>,
}

impl Ballotchain {
    pub fn new() -> Self {
        Ballotchain {
            ballots: vec![],
            pending_votes: vec![],
            //kademlia,
        }
    }

    pub fn update_with_block(&mut self, ballot: Ballot) -> Result<(), BlockValidationErr> {
        let i = self.ballots.len();

        if ballot.index != i as u32 {
            return Err(BlockValidationErr::MismatchedIndex);
        } else if !ballot::check_difficulty(&ballot.hash(), ballot.difficulty) {
            return Err(BlockValidationErr::InvalidHash);
        } else if i != 0 {
            // Not genesis ballot
            let prev_block = &self.ballots[i - 1];
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

        self.ballots.push(ballot);

        Ok(())
    }
    
    pub fn handle_input_line(&mut self, kademlia: &mut Kademlia<MemoryStore>, line: String, last_hash: &Vec<u8>) {
        let mut lineS = &line.clone();
        let mut lineK = &line.clone();
        let mut args = lineS.split(' ');
        println!("{}", lineS);
        println!("{}", lineK);
        let mut second_args = lineK.split(' ');
        second_args.next();

        println!("{}", lineS);
        println!("{}", lineK);
        let mut aKey;// = second_args.take(0).collect::<Vec<_>>();
        
        let mut newKey;//: u32 = aKey.pop().unwrap().parse().unwrap();
        
        let mut value = args.next();

        let difficulty = 0x000fffffffffffffffffffffffffffff;
        match args.next() {
            Some("OY_GOSTER") => {
                let key = {
                    match args.next() {
                        Some(key) => Key::new(&key),
                        None => {
                            eprintln!("Expected key");
                            return;
                        }
                    }
                };
                second_args.next();
                aKey = second_args.take(0).collect::<Vec<_>>();
                newKey = aKey.pop().unwrap().parse().unwrap();
                println!("newKey {:?}", newKey);
                kademlia.get_record(&key, Quorum::One);
            }
            Some("OY_VER") => {
                let key = {
                    match args.next() {
                        Some(key) => Key::new(&key),
                        None => {
                            eprintln!("Expected key");
                            return;
                        }
                    }
                };
                second_args.next();
                aKey = second_args.take(0).collect::<Vec<_>>();
                newKey = aKey.pop().unwrap().parse().unwrap();
                println!("newKey {:?}", newKey);
                let mut ballot = Ballot::new(1, now(), (&last_hash).to_vec(), 0 ,362, difficulty);
                //just simple run of vote mechanic to "mine" a ballot and putting 0 as candidateId
                ballot.vote(newKey);
                let value = ballot.hash.clone();
                println!("Voted(mined) with ballot {:?}", &ballot);
                let record = Record {
                    key,
                    value: ballot.hash.clone() ,
                    publisher: None,
                    expires: None,
                };

                //last_hash = &last_hash.clone(); //to be assigned to new ballot
                self.update_with_block(ballot).expect("Failed to add ballot");
               
                kademlia
                    .put_record(record, Quorum::One)
                    .expect("Failed to store record locally.");
            }
            _ => {
                eprintln!("OY_GOSTER ya da OY_VER komutu beklenildi ");
            }
        }
    }
   
    pub fn init_network(&mut self,  last_hash: &Vec<u8>) -> Result<(), Box<dyn Error>> {
        task::block_on(async {
            env_logger::init();

            // Create a random key for ourselves.
            let local_key = identity::Keypair::generate_ed25519();
            let local_peer_id = PeerId::from(local_key.public());
            // Set up a an encrypted DNS-enabled TCP Transport over the Mplex protocol.

            // this is std::fs, which blocks
            let transport = development_transport(local_key).await?;

            // We create a custom network behaviour that combines Kademlia and mDNS.
            #[derive(NetworkBehaviour)]
            struct MyBehaviour {
                kademlia: Kademlia<MemoryStore>,
                mdns: Mdns,
            }

            impl NetworkBehaviourEventProcess<MdnsEvent> for MyBehaviour {
                // Called when `mdns` produces an event.
                fn inject_event(&mut self, event: MdnsEvent) {
                    if let MdnsEvent::Discovered(list) = event {
                        for (peer_id, multiaddr) in list {
                            self.kademlia.add_address(&peer_id, multiaddr);
                        }
                    }
                }
            }

            impl NetworkBehaviourEventProcess<KademliaEvent> for MyBehaviour {
                // Called when `kademlia` produces an event.
                fn inject_event(&mut self, message: KademliaEvent) {
                    match message {
                        KademliaEvent::OutboundQueryCompleted { result, .. } => match result {
                            QueryResult::GetProviders(Ok(ok)) => {
                                for peer in ok.providers {
                                    println!(
                                        "Peer {:?} provides key {:?}",
                                        peer,
                                        std::str::from_utf8(ok.key.as_ref()).unwrap()
                                    );
                                }
                            }
                            QueryResult::GetProviders(Err(err)) => {
                                eprintln!("Failed to get providers: {:?}", err);
                            }
                            QueryResult::GetRecord(Ok(ok)) => {
                                for PeerRecord {
                                    record: Record { key, value, .. },
                                    ..
                                } in ok.records
                                {
                                    println!(
                                        "Got record {:?} {:?}",
                                        std::str::from_utf8(key.as_ref()).unwrap(),
                                        std::str::from_utf8(&value).unwrap(),
                                    );
                                }
                            }
                            QueryResult::GetRecord(Err(err)) => {
                                eprintln!("Failed to get record: {:?}", err);
                            }
                            QueryResult::PutRecord(Ok(PutRecordOk { key })) => {
                                //Create ballot here
                                println!(
                                    "Successfully put record {:?}",
                                    std::str::from_utf8(key.as_ref()).unwrap()
                                );
                            }
                            QueryResult::PutRecord(Err(err)) => {
                                eprintln!("Failed to put record: {:?}", err);
                            }
                            QueryResult::StartProviding(Ok(AddProviderOk { key })) => {
                                println!(
                                    "Successfully put provider record {:?}",
                                    std::str::from_utf8(key.as_ref()).unwrap()
                                );
                            }
                            QueryResult::StartProviding(Err(err)) => {
                                eprintln!("Failed to put provider record: {:?}", err);
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
           
            // Create a swarm to manage peers and events.
            let mut swarm = {
                // Create a Kademlia behaviour.
                let store = MemoryStore::new(local_peer_id.clone());
                let kademlia = Kademlia::new(local_peer_id.clone(), store);
                let mdns = task::block_on(Mdns::new(MdnsConfig::default()))?;
                let behaviour = MyBehaviour { kademlia, mdns };
                Swarm::new(transport, behaviour, local_peer_id)
            };

            // Read full lines from stdin
            let mut stdin = io::BufReader::new(io::stdin()).lines();

            // Listen on all interfaces and whatever port the OS assigns.

            swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

            // Kick it off.c
            //Add us to the count of discovered peers to distributed key/value stuff
            // Kick it off.
            task::block_on(future::poll_fn(move |cx: &mut Context<'_>| {
                loop {
                    match stdin.try_poll_next_unpin(cx)? {
                        Poll::Ready(Some(line)) => {
                            Ballotchain::handle_input_line(self, &mut swarm.behaviour_mut().kademlia, line, last_hash)
                        }
                        Poll::Ready(None) => panic!("Stdin closed"),
                        Poll::Pending => break,
                    }
                }
                loop {
                    match swarm.poll_next_unpin(cx) {
                        Poll::Ready(Some(event)) => {
                            if let SwarmEvent::NewListenAddr { address, .. } = event {
                                println!("Listening on {:?}", address);
                            }
                        }
                        Poll::Ready(None) => return Poll::Ready(Ok(())),
                        Poll::Pending => break,
                    }
                }
                Poll::Pending
            }))
        })
    }
    
    pub fn publish_ballot(ballot: Ballot) -> Result<()> {
      
    }
}
