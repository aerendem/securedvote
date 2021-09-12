use super::*;
use async_std::{io, task};
use env_logger::{Builder, Env};
use futures::prelude::*;
use libp2p::gossipsub::MessageId;
use libp2p::gossipsub::{
    GossipsubEvent, GossipsubMessage, IdentTopic as Topic, MessageAuthenticity, ValidationMode,
};
use libp2p::swarm::NetworkBehaviour;
use libp2p::{gossipsub, identity, swarm::SwarmEvent, PeerId};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use std::vec;
use std::{
    convert::TryInto,
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
        /* } else if !ballot::check_difficulty(&ballot.hash(), ballot.difficulty) {
        return Err(BlockValidationErr::InvalidHash); */
        } else if i != 0 {
            // Not genesis ballot
            let prev_block = &self.ballots[i - 1];
            if ballot.timestamp <= prev_block.timestamp {
                return Err(BlockValidationErr::AchronologicalTimestamp);
            } else if ballot.prev_block_hash != prev_block.hash && ballot.prev_block_hash == vec![0;32] {
                return Err(BlockValidationErr::MismatchedPreviousHash);
            }
        } else {
            // Genesis ballot
            if ballot.prev_block_hash != vec![0;32]
            {
                return Err(BlockValidationErr::InvalidGenesisBlockFormat);
            }
        }

        self.ballots.push(ballot);

        Ok(())
    }

    pub fn init_network(&mut self /* , last_hash: &Vec<u8> */) -> Result<(), Box<dyn Error>> {
        task::block_on(async {
            let difficulty = 0x000fffffffffffffffffffffffffffff;

            let predefined_hash: Vec<u8> = vec![0;32];
            let mut genesis_block = Ballot::new(0, now(), predefined_hash, 0, 362, difficulty);

            genesis_block.vote(0);
            println!("genesis_block {:?}", genesis_block.hash.clone());
            println!("Voted(mined) genesis ballot {:?}", &genesis_block);
            let mut last_hash = genesis_block.hash.clone();
            //let mut last_hash: Vec<u8> = genesis_block.hash.clone().hash();
            //let mut last_hash: Vec<u8> = Hashable.hash(genesis_block.hash.clone()).clone();

            self.update_with_block(genesis_block)
                .expect("Failed to add genesis ballot");

            Builder::from_env(Env::default().default_filter_or("info")).init();

            // Create a random PeerId
            let local_key = identity::Keypair::generate_ed25519();
            let duplicate_local_key: identity::Keypair = identity::Keypair::generate_ed25519();
            let local_peer_id = PeerId::from(local_key.public());
            println!("Local peer id: {:?}", local_peer_id);

            // Set up an encrypted TCP Transport over the Mplex and Yamux protocols
            let transport = libp2p::development_transport(local_key.clone()).await?;

            // Create a Gossipsub topic
            let topic = Topic::new("test-net");

            // Create a Swarm to manage peers and events
            let mut swarm = {
                // To content-address message, we can take the hash of message and use it as an ID.
                let message_id_fn = |message: &GossipsubMessage| {
                    let mut s = DefaultHasher::new();
                    message.data.hash(&mut s);
                    MessageId::from(s.finish().to_string())
                };

                // Set a custom gossipsub
                let gossipsub_config = gossipsub::GossipsubConfigBuilder::default()
                    .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
                    .validation_mode(ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
                    .message_id_fn(message_id_fn) // content-address messages. No two messages of the
                    // same content will be propagated.
                    .build()
                    .expect("Valid config");
                // build a gossipsub network behaviour
                let mut gossipsub: gossipsub::Gossipsub = gossipsub::Gossipsub::new(
                    MessageAuthenticity::Signed(local_key),
                    gossipsub_config,
                )
                .expect("Correct configuration");

                // subscribes to our topic
                gossipsub.subscribe(&topic).unwrap();

                // add an explicit peer if one was provided
                if let Some(explicit) = std::env::args().nth(2) {
                    let explicit = explicit.clone();
                    match explicit.parse() {
                        Ok(id) => gossipsub.add_explicit_peer(&id),
                        Err(err) => println!("Failed to parse explicit peer id: {:?}", err),
                    }
                }

                // build the swarm
                libp2p::Swarm::new(transport, gossipsub, local_peer_id)
            };

            // Listen on all interfaces and whatever port the OS assigns
            swarm
                .listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap())
                .unwrap();

            // Reach out to another node if specified
            if let Some(to_dial) = std::env::args().nth(1) {
                let dialing = to_dial.clone();
                match to_dial.parse() {
                    Ok(to_dial) => match swarm.dial_addr(to_dial) {
                        Ok(_) => {
                            println!("Dialed {:?}", dialing);
                            let mut entry_message = String::new();
                            entry_message = "ILK_BAGLANTI".to_string();


                            let message_id_fn = |message: &GossipsubMessage| {
                                let mut s = DefaultHasher::new();
                                message.data.hash(&mut s);
                                MessageId::from(s.finish().to_string())
                            };
                            let gossipsub_config = gossipsub::GossipsubConfigBuilder::default()
                            .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
                            .validation_mode(ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
                            .message_id_fn(message_id_fn) // content-address messages. No two messages of the
                            // same content will be propagated.
                            .build()
                            .expect("Valid config");
                            let test_topic = Topic::new("ILK_BAGLANTI-net");
                            let mut gossipsub: gossipsub::Gossipsub = gossipsub::Gossipsub::new(
                                MessageAuthenticity::Signed(duplicate_local_key),
                                gossipsub_config,
                            )
                            .expect("Correct configuration");
                            gossipsub.subscribe(&test_topic).unwrap();

                            swarm
                                .behaviour_mut()
                                .publish(topic.clone(), entry_message.as_bytes());
                        }
                        
                        Err(e) => println!("Dial {:?} failed: {:?}", dialing, e),
                    },
                    Err(err) => println!("Failed to parse address to dial: {:?}", err),
                }
            }

            // Read full lines from stdin
            let mut stdin = io::BufReader::new(io::stdin()).lines();

            // Kick it off
            task::block_on(future::poll_fn(move |cx: &mut Context<'_>| {
                loop {
                    match stdin.try_poll_next_unpin(cx)? {
                        Poll::Ready(Some(line)) => {
                            let mut line = &line.clone();
                            let mut args = line.split_whitespace();
                            match args.next() {
                                Some("OY_GOSTER") => {
                                    let string_of_slice: String = args.next().unwrap().to_string();
                                    println!("{:?}", string_of_slice);
                                    let key: u32 = string_of_slice.parse::<u32>().unwrap();
                                    println!("{:?}", key);

                                    let mut vote_count: u32 = 0;
                                    for ballot in self.ballots.iter() {
                                        if ballot.voted_candidate_id == key {
                                            vote_count += 1;
                                        }
                                    }
                                    println!("{:?}", vote_count);
                                }
                                Some("OY_VER") => {
                                    let key = args.next().unwrap();
                                    let new_index: u32 = self.ballots.len().try_into().unwrap();
                                    println!("key {:?}", key);
                                    let mut ballot: Ballot = Ballot::new(
                                        new_index,
                                        now(),
                                        last_hash.clone(),
                                        0,
                                        123,
                                        difficulty,
                                    );

                                    //just simple run of vote mechanic to "mine" a ballot and putting 0 as candidateId
                                    ballot.vote(key.parse::<u32>().unwrap());
                                    last_hash = ballot.hash.clone();
                                    println!("Voted(mined) with ballot {:?}", &ballot);

                                    let mut clonedBallot = ballot.clone();

                                    //UPDATE LAST HASH HERE

                                    self.update_with_block(ballot)
                                        .expect("Failed to add ballot");

                                    let json = serde_json::to_string(&clonedBallot)
                                        .expect("can jsonify request");
                                    swarm
                                        .behaviour_mut()
                                        .publish(topic.clone(), json.as_bytes());
                                }
                                /* Some("ILK_BAGLANTI") => {
                                    let message_id_fn = |message: &GossipsubMessage| {
                                        let mut s = DefaultHasher::new();
                                        message.data.hash(&mut s);
                                        MessageId::from(s.finish().to_string())
                                    };
                                    let gossipsub_config = gossipsub::GossipsubConfigBuilder::default()
                                    .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
                                    .validation_mode(ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
                                    .message_id_fn(message_id_fn) // content-address messages. No two messages of the
                                    // same content will be propagated.
                                    .build()
                                    .expect("Valid config");
                                    let duplicate_local_key: identity::Keypair = identity::Keypair::generate_ed25519();
                                    let test_topic = Topic::new("ILK_BAGLANTI-net");
                                    let mut gossipsub: gossipsub::Gossipsub = gossipsub::Gossipsub::new(
                                        MessageAuthenticity::Signed(duplicate_local_key),
                                        gossipsub_config,
                                    )
                                    .expect("Correct configuration");
                                    gossipsub.subscribe(&test_topic).unwrap();
                                    
                                    let json = serde_json::to_string(&self.ballots)
                                        .expect("can jsonify request");
                                    swarm
                                        .behaviour_mut()
                                        .publish(topic.clone(), json.as_bytes());

                                    
                                } */
                                None => eprintln!("olmadı"),
                                _ => {
                                    eprintln!("OY_GOSTER ya da OY_VER komutu beklenildi ");
                                }
                            }
                        }
                        Poll::Ready(None) => panic!("Stdin closed"),
                        Poll::Pending => break,
                    }
                    {
                        println!("_");
                    }
                }

                loop {
                    match swarm.poll_next_unpin(cx) {
                        Poll::Ready(Some(event)) => match event {
                            SwarmEvent::Behaviour(GossipsubEvent::Message {
                                propagation_source: peer_id,
                                message_id: id,
                                message,
                            }) => {
                                println!(
                                    "Got message: {} with id: {} from peer: {:?}",
                                    String::from_utf8_lossy(&message.data),
                                    id,
                                    peer_id
                                );

                                let received_ballot: Ballot =
                                    serde_json::from_str(&String::from_utf8_lossy(&message.data))
                                        .unwrap();
                                self.update_with_block(received_ballot)
                                    .expect("Failed to add ballot");
                            }

                            SwarmEvent::NewListenAddr { address, .. } => {
                                println!("Listening on {:?}", address);
                            }
                            _ => {}
                        },
                        Poll::Ready(None) | Poll::Pending => break,
                    }
                }

                Poll::Pending
            }))
        })
    }
}
