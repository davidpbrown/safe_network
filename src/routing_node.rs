// Copyright 2015 MaidSafe.net limited
// This MaidSafe Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
// By contributing code to the MaidSafe Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement, version 1.0, found in the root
// directory of this project at LICENSE, COPYING and CONTRIBUTOR respectively and also
// available at: http://www.maidsafe.net/licenses
// Unless required by applicable law or agreed to in writing, the MaidSafe Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS
// OF ANY KIND, either express or implied.
// See the Licences for the specific language governing permissions and limitations relating to
// use of the MaidSafe
// Software.

use sodiumoxide;
use crust;
use std::sync::{Arc, mpsc, Mutex};
//use sodiumoxide::crypto;
use std::sync::mpsc::{Receiver};
use facade::*;
use super::*;
use std::net::{SocketAddr};
use std::str::FromStr;


use routing_table::RoutingTable;
use types::DhtId;
use message_header::MessageHeader;
use messages::get_data::GetData;
use messages::get_data_response::GetDataResponse;
use messages::put_data::PutData;
use messages::put_data_response::PutDataResponse;
use messages::connect::ConnectRequest;
use messages::connect_response::ConnectResponse;
use messages::find_group::FindGroup;
use messages::find_group_response::FindGroupResponse;


type ConnectionManager = crust::ConnectionManager<DhtId>;
type Event             = crust::Event<DhtId>;
type Bytes             = Vec<u8>;
/// DHT node
pub struct RoutingNode<F: Facade> {
    facade: Arc<Mutex<F>>,
    pmid: types::Pmid,
    own_id: types::DhtId,
    event_input: Receiver<Event>,
    connections: ConnectionManager,
    routing_table: RoutingTable,
    accepting_on: Option<u16>
}

impl<F> RoutingNode<F> where F: Facade {
    pub fn new(id: DhtId, my_facade: F) -> RoutingNode<F> {
        sodiumoxide::init(); // enable shared global (i.e. safe to mutlithread now)
        // let key_pair = crypto::sign::gen_keypair();
        // let encrypt_key_pair = crypto::asymmetricbox::gen_keypair();
        let (event_output, event_input) = mpsc::channel();
        let pmid = types::Pmid::new();
        let own_id = id; //DhtId(pmid.get_name());  FIXME (prakash) ?????

        let cm = crust::ConnectionManager::new(own_id.clone(), event_output);

        let accepting_on = cm.start_accepting().ok();

        RoutingNode { facade: Arc::new(Mutex::new(my_facade)),
                      pmid : pmid,
                      own_id : own_id.clone(),
                      event_input: event_input,
                      connections: cm,
                      routing_table : RoutingTable::new(own_id),
                      accepting_on: accepting_on
                    }
    }

    pub fn accepting_on(&self) -> Option<SocketAddr> {
        self.accepting_on.and_then(|port| {
            SocketAddr::from_str(&format!("127.0.0.1:{}", port)).ok()
        })
    }

    /// Retreive something from the network (non mutating) - Direct call
    pub fn get(&self, type_id: u64, name: types::DhtId) { unimplemented!()}

    /// Add something to the network, will always go via ClientManager group
    pub fn put(&self, name: types::DhtId, content: Vec<u8>) { unimplemented!() }

    /// Mutate something on the network (you must prove ownership) - Direct call
    pub fn post(&self, name: types::DhtId, content: Vec<u8>) { unimplemented!() }

    //fn get_facade(&'a mut self) -> &'a Facade {
    //    self.facade
    //}

    pub fn add_bootstrap(&self, endpoint: SocketAddr) {
        let _ = self.connections.connect(endpoint);
    }
    
    pub fn run(&mut self) {
        loop {
            let event = self.event_input.recv();

            if event.is_err() { return; }

            match event.unwrap() {
                crust::Event::NewMessage(id, bytes) => {
                    self.message_received(id, bytes);
                },
                crust::Event::NewConnection(id) => {
                    self.handle_new_connection(id);
                },
                crust::Event::LostConnection(id) => {
                    self.handle_lost_connection(id);
                }
            }
        }
    }
    
    fn next_endpoint_pair(&self)->(types::EndPoint, types::EndPoint) {
      unimplemented!();  // FIXME (Peter)
    }

    fn handle_new_connection(&mut self, peer_id: types::DhtId) {
        if false {  // if unexpected connection, its likely
            //add to non_routing_list;
        } else {
            // let peer_node_info = NodeInfo {};
            // if !(self.routing_table.add_node(&peer_id)) {
            //     self.connection.drop_node(peer_id);
            // }
        }
        // handle_curn
    }

    fn handle_lost_connection(&mut self, peer_id: types::DhtId) {
        self.routing_table.drop_node(&peer_id);
        // remove from the non routing list
        // handle_curn
    }

    fn message_received(&mut self, peer_id: types::DhtId, serialised_message: Bytes) {
      // Parse
      // filter check
      // add to filter
      // add to cache
      // cache check / response
      // SendSwarmOrParallel
      // handle relay request/response
      // switch message type
      unimplemented!();
    }

    fn handle_connect(&self, connect_request: ConnectRequest, original_header: MessageHeader) {
        if !(self.routing_table.check_node(&connect_request.requester_id)) {
           return;
        }
        let (receiver_local, receiver_external) = self.next_endpoint_pair();
        let own_public_pmid = types::PublicPmid::generate_random();  // FIXME (Ben)
        let connect_response = ConnectResponse {
                                requester_local: connect_request.local,
                                requester_external: connect_request.external,
                                receiver_local: receiver_local,
                                receiver_external: receiver_external,
                                requester_id: connect_request.requester_id,
                                receiver_id: self.own_id.clone(),
                                receiver_fob: own_public_pmid};
        debug_assert!(connect_request.receiver_id == self.own_id);
        // Serialise message

        // if (bootstrap_node_) {
        // SendToBootstrapNode(message);
        // }
        // SendSwarmOrParallel();
        // if (original_header.ReplyToAddress())
        // SendToNonRoutingNode((*original_header.ReplyToAddress()).data, message);


        // Add connection
        // AddNodeAccept
    }

    fn handle_connect_response(&self, connect_response: ConnectResponse) {
        if !(self.routing_table.check_node(&connect_response.receiver_id)) {
           return;
        }
        // AddNode
        // self.connections.connect();
    }

    fn handle_find_group(find_group: FindGroup, original_header: MessageHeader) {
        unimplemented!();
    }

    fn handle_find_group_response(find_group_response: FindGroupResponse, original_header: MessageHeader) {
        unimplemented!();
    }

    fn handle_get_data(get_data: GetData, original_header: MessageHeader) {
        unimplemented!();
    }

    fn handle_get_data_response(get_data_response: GetDataResponse, original_header: MessageHeader) {
        // need to call facade handle_get_response
        unimplemented!();
    }

    // // for clients, below methods are required
    fn handle_put_data(put_data: PutData, original_header: MessageHeader) {
        // need to call facade handle_get_response
        unimplemented!();
    }

    fn handle_put_data_response(put_data_response: PutDataResponse, original_header: MessageHeader) {
        // need to call facade handle_put_response
        unimplemented!();
    }
}

#[cfg(test)]
mod test {
    use routing_node::{RoutingNode};
    use facade::{Facade};
    use types::{Authority, DhtId, DestinationAddress};
    use super::super::{Action, RoutingError};
    use std::thread;
    use std::net::{SocketAddr};

    struct NullFacade;

    impl Facade for NullFacade {
      fn handle_get(&mut self, type_id: u64, our_authority: Authority, from_authority: Authority,from_address: DhtId , data: Vec<u8>)->Result<Action, RoutingError> { Err(RoutingError::Success) }
      fn handle_put(&mut self, our_authority: Authority, from_authority: Authority,
                    from_address: DhtId, dest_address: DestinationAddress, data: Vec<u8>)->Result<Action, RoutingError> { Err(RoutingError::Success) }
      fn handle_post(&mut self, our_authority: Authority, from_authority: Authority, from_address: DhtId, data: Vec<u8>)->Result<Action, RoutingError> { Err(RoutingError::Success) }
      fn handle_get_response(&mut self, from_address: DhtId , response: Result<Vec<u8>, RoutingError>) { }
      fn handle_put_response(&mut self, from_authority: Authority,from_address: DhtId , response: Result<Vec<u8>, RoutingError>) { }
      fn handle_post_response(&mut self, from_authority: Authority,from_address: DhtId , response: Result<Vec<u8>, RoutingError>) { }
      fn add_node(&mut self, node: DhtId) {}
      fn drop_node(&mut self, node: DhtId) {}
    }

    #[test]
    fn test_routing_node() {
        let f1 = NullFacade;
        let f2 = NullFacade;
        let n1 = RoutingNode::new(DhtId::generate_random(), f1);
        let n2 = RoutingNode::new(DhtId::generate_random(), f2);

        let n1_ep = n1.accepting_on().unwrap();
        let n2_ep = n2.accepting_on().unwrap();

        fn run_node(n: RoutingNode<NullFacade>, my_ep: SocketAddr, his_ep: SocketAddr)
            -> thread::JoinHandle
        {
            thread::spawn(move || {
                let mut n = n;
                if my_ep.port() < his_ep.port() {
                    n.add_bootstrap(his_ep);
                }
                n.run();
            })
        }

        let t1 = run_node(n1, n1_ep.clone(), n2_ep.clone());
        let t2 = run_node(n2, n2_ep.clone(), n1_ep.clone());

        assert!(t1.join().is_ok());
        assert!(t2.join().is_ok());
    }
}
