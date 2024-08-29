use super::error::Result;
use core::fmt::Debug;
use karlsen_grpc_core::{
    ops::KarlsendPayloadOps,
    protowire::{KarlsendRequest, KarlsendResponse},
};
use std::{sync::Arc, time::Duration};
use tokio::sync::oneshot;

pub(crate) mod id;
pub(crate) mod matcher;
pub(crate) mod queue;

pub(crate) trait Resolver: Send + Sync + Debug {
    fn register_request(
        &self,
        op: KarlsendPayloadOps,
        request: &KarlsendRequest,
    ) -> KarlsendResponseReceiver;
    fn handle_response(&self, response: KarlsendResponse);
    fn remove_expired_requests(&self, timeout: Duration);
}

pub(crate) type DynResolver = Arc<dyn Resolver>;

pub(crate) type KarlsendResponseSender = oneshot::Sender<Result<KarlsendResponse>>;
pub(crate) type KarlsendResponseReceiver = oneshot::Receiver<Result<KarlsendResponse>>;
