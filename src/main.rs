use libp2p::{
    core::upgrade, gossipsub::{subscription_filter::{AllowAllSubscriptionFilter, TopicSubscriptionFilter},
    Gossipsub, GossipsubConfig, IdentTopic, IdentityTransform, MessageAuthenticity, Sha256Topic, Topic},
    identity, kad::{store::MemoryStore, Kademlia, KademliaConfig}, mplex, noise::{Keypair, NoiseConfig, X25519Spec},
    swarm::{Swarm, SwarmBuilder}, tcp::TokioTcpConfig, Multiaddr, PeerId, Transport 
};
use log::info;
use news::{EventType, News, NewsMode};
use once_cell::sync::Lazy;
use tokio::{io::AsyncBufReadExt, sync::mpsc};

mod network;
mod news;

pub static KEYS: Lazy<identity::Keypair> = Lazy::new(|| identity::Keypair::generate_ed25519());
pub static PEER_ID: Lazy<PeerId> = Lazy::new(|| PeerId::from(KEYS.public()));

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    info!("Peer Id: {:?}", PEER_ID);

    let auth_keys = Keypair::<X25519Spec>::new()
        .into_authentic(&KEYS)
        .expect("can create auth keys");

    //create transport 
    let transport = TokioTcpConfig::new()
        .upgrade(upgrade::Version::V1)
        .authenticate(NoiseConfig::xx(auth_keys).into_authenticated())
        .multiplex(mplex::MplexConfig::new())
        .boxed();

      
    let mut network_behaviour = network::NewsNetworkBehaviour{
        gossipsub: Gossipsub::new(MessageAuthenticity::Signed(KEYS.clone()) , GossipsubConfig::default()).unwrap(),
        kademlia: Kademlia::with_config(PEER_ID.clone(), MemoryStore::new(PEER_ID.clone()), KademliaConfig::default())
    };
    
    //subscribe to topic
    let topic = IdentTopic::new("news");
    network_behaviour.gossipsub.subscribe(&topic);

    let mut swarm = SwarmBuilder::new(transport, network_behaviour, PEER_ID.clone())
        .executor(Box::new(|fut| {
            tokio::spawn(fut);
        }))
        .build();

    let listen_addr: Multiaddr = "/ip4/0.0.0.0/tcp/0".parse().unwrap();
    Swarm::listen_on(&mut swarm, listen_addr).expect("Swarm is started");

    //Handling the upcoming request
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();

    loop{
        let evt = {
            tokio::select! {
                line = stdin.next_line() => Some(EventType::Input(line.expect("Can get line").expect("can read line"))),
                event = swarm.next() => {
                    info!("Unhandled Swarm Event: {:?}", event);
                    None
                }
                response = response_receiver.recv() => Some(EventType::Response(response.expect("response exists"))),
            }
        };
    }
}
