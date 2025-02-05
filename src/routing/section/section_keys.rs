// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::routing::error::{Error, Result};
use std::collections::{HashMap, VecDeque};

/// All the key material needed to sign or combine signature for our section key.
#[derive(custom_debug::Debug)]
pub(crate) struct SectionKeyShare {
    /// Public key set to verify threshold signatures and combine shares.
    pub(crate) public_key_set: bls::PublicKeySet,
    /// Index of the owner of this key share within the set of all section elders.
    pub(crate) index: usize,
    /// Secret Key share.
    #[debug(skip)]
    pub(crate) secret_key_share: bls::SecretKeyShare,
}

/// Struct that holds the current section keys and helps with new key generation.
#[derive(Debug)]
pub(crate) struct SectionKeysProvider {
    /// A cache for current and previous section BLS keys.
    cache: MiniKeyCache,
    /// The new keys to use when section update completes.
    // TODO: evict outdated keys.
    // TODO: alternatively, store the pending keys in DkgVoter instead. That way the outdated ones
    //       would get dropped when the DKG session itself gets dropped which we already have
    //       implemented.
    pending: HashMap<bls::PublicKey, SectionKeyShare>,
}

impl SectionKeysProvider {
    pub(crate) fn new(cache_size: u8, current: Option<SectionKeyShare>) -> Self {
        let mut provider = Self {
            pending: HashMap::new(),
            cache: MiniKeyCache::with_capacity(cache_size as usize),
        };
        if let Some(share) = current {
            let public_key = share.public_key_set.public_key();
            provider.insert_dkg_outcome(share);
            provider.finalise_dkg(&public_key);
        }
        provider
    }

    pub(crate) fn key_share(&self) -> Result<&SectionKeyShare> {
        self.cache.get_most_recent()
    }

    pub(crate) fn sign_with(
        &self,
        data: &[u8],
        public_key: &bls::PublicKey,
    ) -> Result<(usize, bls::SignatureShare)> {
        self.cache.sign_with(data, public_key)
    }

    pub(crate) fn has_key_share(&self) -> bool {
        self.cache.has_key_share()
    }

    pub(crate) fn insert_dkg_outcome(&mut self, share: SectionKeyShare) {
        let public_key = share.public_key_set.public_key();
        let _ = self.pending.insert(public_key, share);
    }

    pub(crate) fn finalise_dkg(&mut self, public_key: &bls::PublicKey) {
        if let Some(share) = self.pending.remove(public_key) {
            if let Some(evicted) = self.cache.add(public_key, share) {
                trace!("evicted old key from cache: {:?}", evicted);
            }
            trace!("finalised DKG: {:?}", public_key);
        }
    }
}

/// Implementation of super simple cache, for no more than a handfull of items.
#[derive(Debug)]
struct MiniKeyCache {
    list: VecDeque<(bls::PublicKey, SectionKeyShare)>,
}

impl MiniKeyCache {
    /// Constructor for capacity based `KeyCache`.
    fn with_capacity(capacity: usize) -> MiniKeyCache {
        MiniKeyCache {
            list: VecDeque::with_capacity(capacity),
        }
    }

    /// Returns true if a key share exists.
    fn has_key_share(&self) -> bool {
        !self.list.is_empty()
    }

    /// Returns the most recently added key.
    fn get_most_recent(&self) -> Result<&SectionKeyShare> {
        if let Some((_, share)) = self.list.back() {
            return Ok(share);
        }
        Err(Error::MissingSecretKeyShare)
    }

    /// Uses the secret key from cache, corresponding to
    /// the provided public key.
    fn sign_with(
        &self,
        data: &[u8],
        public_key: &bls::PublicKey,
    ) -> Result<(usize, bls::SignatureShare)> {
        for (cached_public, section_key_share) in &self.list {
            if public_key == cached_public {
                return Ok((
                    section_key_share.index,
                    section_key_share.secret_key_share.sign(data),
                ));
            }
        }
        Err(Error::MissingSecretKeyShare)
    }

    /// Adds a new key to the cache, and removes + returns the oldest
    /// key if cache size is exceeded.
    fn add(
        &mut self,
        public_key: &bls::PublicKey,
        section_key_share: SectionKeyShare,
    ) -> Option<bls::PublicKey> {
        for (cached_public, _) in &self.list {
            if public_key == cached_public {
                return None;
            }
        }

        let mut evicted = None;
        if self.list.capacity() == self.list.len() {
            if let Some((cached_public, _)) = self.list.pop_front() {
                evicted = Some(cached_public);
            }
        }

        self.list.push_back((*public_key, section_key_share));

        evicted
    }
}
