//! What this adapter says it is, and the kit that serves its contract.

use std::sync::Arc;

use lemonfiber_adapter_kit::{Kit, Readiness};
use lemonfiber_contract::adapter::{About, Release};
use lemonfiber_contract::capabilities::quality;

use crate::configarr::Configarr;

/// Where Configarr's configuration directory is, in the adapter's container, which the
/// adapter shares.
pub const ADDRESS: &str = "/configarr";

/// The upstream this adapter fronts, as its project names itself.
pub const UPSTREAM: &str = "Configarr";

include!(concat!(env!("OUT_DIR"), "/fronted.rs"));

/// What this adapter says it is: the contract it speaks and the release it supports.
#[must_use]
pub fn about() -> About {
    About {
        speaks: vec![lemonfiber_contract::spoken(
            quality::sync::CAPABILITY,
            quality::sync::MAJOR,
        )],
        upstream: UPSTREAM.to_owned(),
        releases: vec![Release {
            version: RELEASE.to_owned(),
            digest: DIGEST.to_owned(),
        }],
    }
}

/// The contract, served from `configarr` under `key`, ready while Configarr's configuration
/// directory is there to write.
#[must_use]
pub fn kit(key: String, configarr: Arc<Configarr>) -> Kit {
    let asked = Arc::clone(&configarr);
    let ready: Readiness = Arc::new(move || {
        let asked = Arc::clone(&asked);
        Box::pin(async move { asked.answers().await })
    });
    Kit::new(key, about())
        .ready_when(ready)
        .serving(quality::sync::served(configarr))
}

#[cfg(test)]
mod tests;
