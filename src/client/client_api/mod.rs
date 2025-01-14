// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod blob_apis;
mod commands;
mod data;
mod queries;
mod register_apis;

pub use self::blob_apis::BlobAddress;
use crate::client::{connections::Session, errors::Error, Config};
use crate::messaging::data::CmdError;
use crate::types::{Keypair, PublicKey};

use rand::rngs::OsRng;
use std::collections::BTreeSet;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::{
    sync::{mpsc::Receiver, RwLock},
    time::Duration,
};
use tracing::{debug, info};

/// Client object
#[derive(Clone, Debug)]
pub struct Client {
    keypair: Keypair,
    incoming_errors: Arc<RwLock<Receiver<CmdError>>>,
    session: Session,
    pub(crate) query_timeout: Duration,
}

/// Easily manage connections to/from The Safe Network with the client and its APIs.
/// Use a random client for read-only or one-time operations.
/// Supply an existing, SecretKey which holds a SafeCoin balance to be able to perform
/// write operations.
impl Client {
    /// Create a Safe Network client instance. Either for an existing SecretKey (in which case) the client will attempt
    /// to retrieve the history of the key's balance in order to be ready for any token operations. Or if no SecreteKey
    /// is passed, a random keypair will be used, which provides a client that can only perform Read operations (at
    /// least until the client's SecretKey receives some token).
    ///
    /// # Examples
    ///
    /// TODO: update once data types are crdt compliant
    ///
    pub async fn new(
        config: Config,
        bootstrap_nodes: BTreeSet<SocketAddr>,
        optional_keypair: Option<Keypair>,
    ) -> Result<Self, Error> {
        let mut rng = OsRng;

        let keypair = match optional_keypair {
            Some(id) => {
                info!("Client started for specific pk: {:?}", id.public_key());
                id
            }
            None => {
                let keypair = Keypair::new_ed25519(&mut rng);
                info!(
                    "Client started for new randomly created pk: {:?}",
                    keypair.public_key()
                );
                keypair
            }
        };

        // Incoming error notifiers
        let (err_sender, err_receiver) = tokio::sync::mpsc::channel::<CmdError>(10);

        let client_pk = keypair.public_key();

        // Bootstrap to the network, connecting to a section based
        // on a public key of our choice.
        debug!(
            "Bootstrapping to the network, genesis key: {} ...",
            hex::encode(config.genesis_key.to_bytes())
        );
        // Create a session with the network
        let session = Session::attempt_bootstrap(
            client_pk,
            config.genesis_key,
            config.qp2p,
            bootstrap_nodes.clone(),
            config.local_addr,
            err_sender,
        )
        .await?;

        let client = Self {
            keypair,
            session,
            incoming_errors: Arc::new(RwLock::new(err_receiver)),
            query_timeout: config.query_timeout,
        };

        Ok(client)
    }

    /// Return the client's keypair.
    ///
    /// Useful for retrieving the PublicKey or KeyPair in the event you need to _sign_ something
    ///
    /// # Examples
    ///
    /// TODO: update once data types are crdt compliant
    ///
    pub fn keypair(&self) -> Keypair {
        self.keypair.clone()
    }

    /// Return the client's PublicKey.
    ///
    /// # Examples
    ///
    /// TODO: update once data types are crdt compliant
    ///
    pub fn public_key(&self) -> PublicKey {
        self.keypair().public_key()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::utils::test_utils::{create_test_client, create_test_client_with};
    use crate::types::utils::random_bytes;
    use crate::url::Scope;
    use eyre::Result;
    use std::{
        collections::HashSet,
        net::{IpAddr, Ipv4Addr, SocketAddr},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn client_creation() -> Result<()> {
        let _client = create_test_client(None).await?;

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    #[ignore]
    async fn client_nonsense_bootstrap_fails() -> Result<()> {
        let mut nonsense_bootstrap = HashSet::new();
        let _ = nonsense_bootstrap.insert(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            3033,
        ));
        //let setup = create_test_client_with(None, Some(nonsense_bootstrap)).await;
        //assert!(setup.is_err());
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn client_creation_with_existing_keypair() -> Result<()> {
        let mut rng = OsRng;
        let full_id = Keypair::new_ed25519(&mut rng);
        let pk = full_id.public_key();

        let client = create_test_client_with(Some(full_id), None).await?;
        assert_eq!(pk, client.public_key());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn long_lived_connection_survives() -> Result<()> {
        let client = create_test_client(None).await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(40)).await;
        let data = random_bytes(self_encryption::MIN_ENCRYPTABLE_BYTES);
        let _ = client.write_to_network(data, Scope::Public).await?;
        Ok(())
    }

    // Send is an important trait that assures futures can be run in a
    // multithreaded context. If a future depends on a non-Send future, directly
    // or indirectly, the future itself becomes non-Send and so on. Thus, it can
    // happen that high-level API functions will become non-Send by accident.
    #[test]
    fn client_is_send() {
        fn require_send<T: Send>(_t: T) {}
        require_send(create_test_client(None));
    }
}
