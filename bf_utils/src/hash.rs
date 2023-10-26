use sha1::{Digest, Sha1};

pub type Sha1Digest = [u8; 20];

pub fn sha1_digest<D: AsRef<[u8]>>(data: D) -> Sha1Digest {
    Sha1::new().chain_update(data).finalize().into()
}
