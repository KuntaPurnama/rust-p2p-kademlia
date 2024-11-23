use std::str::FromStr;

use libp2p::{
    gossipsub::{Gossipsub, GossipsubEvent},
    identity::error,
    kad::{record::Key, store::MemoryStore, Kademlia, KademliaEvent, Record},
    swarm::{NetworkBehaviour, NetworkBehaviourEventProcess},
    NetworkBehaviour, PeerId,
};
use log::info;
use uuid::Uuid;

use crate::news::AddNews;

#[derive(NetworkBehaviour)]
pub struct NewsNetworkBehaviour {
    pub gossipsub: Gossipsub,
    pub kademlia: Kademlia<MemoryStore>,
}

impl NetworkBehaviourEventProcess<GossipsubEvent> for NewsNetworkBehaviour {
    fn inject_event(&mut self, event: GossipsubEvent) {
        match event {
            GossipsubEvent::Message {
                propagation_source,
                message_id,
                message,
            } => {
                info!(
                    "Got message from {:?} with message id : {:?} and message is :{:?}",
                    propagation_source, message_id, message
                );
                if let Ok(req) = serde_json::from_slice::<AddNews>(&message.data) {
                    let id = Uuid::new_v4().to_string();
                    let request = serde_json::to_string(&req.news).unwrap();
                    let record = Record {
                        key: Key::new(&id),
                        value: request.into_bytes(),
                        expires: None,
                        publisher: Option::Some(
                            PeerId::from_str(req.requester_peer_id.as_str()).unwrap(),
                        ),
                    };

                    if let Err(err) = self
                        .kademlia
                        .put_record(record, libp2p::kad::Quorum::Majority)
                    {
                        info!("error when try to save data to kademlia: {:?}", err);
                    }
                }
            }
            GossipsubEvent::Subscribed { peer_id, topic } => {
                info!("Subscribe topic {:?} by {:?}", topic, peer_id);
            }
            GossipsubEvent::Unsubscribed { peer_id, topic } => {
                info!("Unsubscribe topic {:?} by {:?}", topic, peer_id);
            }
        }
    }
}

impl NetworkBehaviourEventProcess<KademliaEvent> for NewsNetworkBehaviour {
    fn inject_event(&mut self, event: KademliaEvent) {
        match event {
            KademliaEvent::RoutingUpdated {
                peer, is_new_peer, ..
            } => {
                if is_new_peer {
                    self.gossipsub.add_explicit_peer(&peer);
                }
            }
            _ => {}
        }
    }
}
