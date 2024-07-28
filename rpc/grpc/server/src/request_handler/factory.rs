use std::sync::Arc;

use super::{
    handler::RequestHandler,
    handler_trait::Handler,
    interface::{Interface, KarlsendMethod, KarlsendRoutingPolicy},
    method::Method,
};
use crate::{
    connection::{Connection, IncomingRoute},
    connection_handler::ServerContext,
    error::GrpcServerError,
};
use karlsen_grpc_core::protowire::{karlsend_request::Payload, *};
use karlsen_grpc_core::{
    ops::KarlsendPayloadOps, protowire::NotifyFinalityConflictResponseMessage,
};
use karlsen_notify::{scope::FinalityConflictResolvedScope, subscriber::SubscriptionManager};
use karlsen_rpc_core::{SubmitBlockRejectReason, SubmitBlockReport, SubmitBlockResponse};
use karlsen_rpc_macros::build_grpc_server_interface;

pub struct Factory {}

impl Factory {
    pub fn new_handler(
        rpc_op: KarlsendPayloadOps,
        incoming_route: IncomingRoute,
        server_context: ServerContext,
        interface: &Interface,
        connection: Connection,
    ) -> Box<dyn Handler> {
        Box::new(RequestHandler::new(
            rpc_op,
            incoming_route,
            server_context,
            interface,
            connection,
        ))
    }

    pub fn new_interface(server_ctx: ServerContext, network_bps: u64) -> Interface {
        // The array as last argument in the macro call below must exactly match the full set of
        // KarlsendPayloadOps variants.
        let mut interface = build_grpc_server_interface!(
            server_ctx.clone(),
            ServerContext,
            Connection,
            KarlsendRequest,
            KarlsendResponse,
            KarlsendPayloadOps,
            [
                SubmitBlock,
                GetBlockTemplate,
                GetCurrentNetwork,
                GetBlock,
                GetBlocks,
                GetInfo,
                Shutdown,
                GetPeerAddresses,
                GetSink,
                GetMempoolEntry,
                GetMempoolEntries,
                GetConnectedPeerInfo,
                AddPeer,
                SubmitTransaction,
                GetSubnetwork,
                GetVirtualChainFromBlock,
                GetBlockCount,
                GetBlockDagInfo,
                ResolveFinalityConflict,
                GetHeaders,
                GetUtxosByAddresses,
                GetBalanceByAddress,
                GetBalancesByAddresses,
                GetSinkBlueScore,
                Ban,
                Unban,
                EstimateNetworkHashesPerSecond,
                GetMempoolEntriesByAddresses,
                GetCoinSupply,
                Ping,
                GetMetrics,
                GetServerInfo,
                GetSyncStatus,
                GetDaaScoreTimestampEstimate,
                NotifyBlockAdded,
                NotifyNewBlockTemplate,
                NotifyFinalityConflict,
                NotifyUtxosChanged,
                NotifySinkBlueScoreChanged,
                NotifyPruningPointUtxoSetOverride,
                NotifyVirtualDaaScoreChanged,
                NotifyVirtualChainChanged,
                StopNotifyingUtxosChanged,
                StopNotifyingPruningPointUtxoSetOverride,
            ]
        );

        // Manually reimplementing the NotifyFinalityConflictRequest method so subscription
        // gets mirrored to FinalityConflictResolved notifications as well.
        let method: KarlsendMethod = Method::new(
            |server_ctx: ServerContext, connection: Connection, request: KarlsendRequest| {
                Box::pin(async move {
                    let mut response: KarlsendResponse = match request.payload {
                        Some(Payload::NotifyFinalityConflictRequest(ref request)) => {
                            match karlsen_rpc_core::NotifyFinalityConflictRequest::try_from(request)
                            {
                                Ok(request) => {
                                    let listener_id = connection.get_or_register_listener_id()?;
                                    let command = request.command;
                                    let result = server_ctx
                                        .notifier
                                        .clone()
                                        .execute_subscribe_command(
                                            listener_id,
                                            request.into(),
                                            command,
                                        )
                                        .await
                                        .and(
                                            server_ctx
                                                .notifier
                                                .clone()
                                                .execute_subscribe_command(
                                                    listener_id,
                                                    FinalityConflictResolvedScope::default().into(),
                                                    command,
                                                )
                                                .await,
                                        );
                                    NotifyFinalityConflictResponseMessage::from(result).into()
                                }
                                Err(err) => NotifyFinalityConflictResponseMessage::from(err).into(),
                            }
                        }
                        _ => {
                            return Err(GrpcServerError::InvalidRequestPayload);
                        }
                    };
                    response.id = request.id;
                    Ok(response)
                })
            },
        );
        interface.replace_method(KarlsendPayloadOps::NotifyFinalityConflict, method);

        // Methods with special properties
        let network_bps = network_bps as usize;
        interface.set_method_properties(
            KarlsendPayloadOps::SubmitBlock,
            network_bps,
            10.max(network_bps * 2),
            KarlsendRoutingPolicy::DropIfFull(Arc::new(Box::new(|_: &KarlsendRequest| {
                Ok(Ok(SubmitBlockResponse {
                    report: SubmitBlockReport::Reject(SubmitBlockRejectReason::RouteIsFull),
                })
                .into())
            }))),
        );

        interface
    }
}
