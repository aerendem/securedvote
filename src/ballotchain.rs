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
    pub kademlia: Kademlia<MemoryStore>,
}

impl Ballotchain {
    pub fn new() -> Self {
        let mut kademlia;
        Ballotchain {
            ballots: vec![],
            pending_votes: vec![],
            kademlia,
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
    
    pub fn handle_input_line(kademlia: &mut Kademlia<MemoryStore>, line: String) {
        let mut args = line.split(' ');

        match args.next() {
            Some("GET") => {
                let key = {
                    match args.next() {
                        Some(key) => Key::new(&key),
                        None => {
                            eprintln!("Expected key");
                            return;
                        }
                    }
                };
                kademlia.get_record(&key, Quorum::One);
            }
            Some("GET_PROVIDERS") => {
                let key = {
                    match args.next() {
                        Some(key) => Key::new(&key),
                        None => {
                            eprintln!("Expected key");
                            return;
                        }
                    }
                };
                kademlia.get_providers(key);
            }
            Some("PUT") => {
                let key = {
                    match args.next() {
                        Some(key) => Key::new(&key),
                        None => {
                            eprintln!("Expected key");
                            return;
                        }
                    }
                };
                let value = {
                    match args.next() {
                        Some(value) => value.as_bytes().to_vec(),
                        None => {
                            eprintln!("Expected value");
                            return;
                        }
                    }
                };
                let record = Record {
                    key,
                    value,
                    publisher: None,
                    expires: None,
                };
                kademlia
                    .put_record(record, Quorum::One)
                    .expect("Failed to store record locally.");
            }
            Some("PUT_PROVIDER") => {
                let key = {
                    match args.next() {
                        Some(key) => Key::new(&key),
                        None => {
                            eprintln!("Expected key");
                            return;
                        }
                    }
                };

                kademlia
                    .start_providing(key)
                    .expect("Failed to start providing key");
            }
            _ => {
                eprintln!("expected GET, GET_PROVIDERS, PUT or PUT_PROVIDER");
            }
        }
    }
    pub fn init_network(&mut self) -> Result<(), Box<dyn Error>> {
        task::block_on(async {
            env_logger::init();

            // Create a random key for ourselves.
            let local_key = identity::Keypair::generate_ed25519();
            let local_peer_id = PeerId::from(local_key.public());
            let _connectionCount: usize;
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
            let mut kademlia = self.kademlia;
            // Create a swarm to manage peers and events.
            let mut swarm = {
                // Create a Kademlia behaviour.
                let store = MemoryStore::new(local_peer_id.clone());
                kademlia = Kademlia::new(local_peer_id.clone(), store);
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
                            Ballotchain::handle_input_line(&mut swarm.behaviour_mut().kademlia, line)
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
    
    pub fn put_vote_for_candidate(candidateId: i32) {
        let &(mut kademlia) = self.kademlia;
        let mut record = &kademlia.get_record(&candidateId, Quorum::One);
        record.value += 1;
        kademlia
            .put_record(record, Quorum::One)
            .expect("Failed to store record locally.");
    }
}
