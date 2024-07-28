use super::method::{DropFn, Method, MethodTrait, RoutingPolicy};
use crate::{
    connection::Connection,
    connection_handler::ServerContext,
    error::{GrpcServerError, GrpcServerResult},
};
use karlsen_grpc_core::{
    ops::KarlsendPayloadOps,
    protowire::{KarlsendRequest, KarlsendResponse},
};
use std::fmt::Debug;
use std::{collections::HashMap, sync::Arc};

pub type KarlsendMethod = Method<ServerContext, Connection, KarlsendRequest, KarlsendResponse>;
pub type DynKarlsendMethod =
    Arc<dyn MethodTrait<ServerContext, Connection, KarlsendRequest, KarlsendResponse>>;
pub type KarlsendDropFn = DropFn<KarlsendRequest, KarlsendResponse>;
pub type KarlsendRoutingPolicy = RoutingPolicy<KarlsendRequest, KarlsendResponse>;

/// An interface providing methods implementations and a fallback "not implemented" method
/// actually returning a message with a "not implemented" error.
///
/// The interface can provide a method clone for every [`KarlsendPayloadOps`] variant for later
/// processing of related requests.
///
/// It is also possible to directly let the interface itself process a request by invoking
/// the `call()` method.
pub struct Interface {
    server_ctx: ServerContext,
    methods: HashMap<KarlsendPayloadOps, DynKarlsendMethod>,
    method_not_implemented: DynKarlsendMethod,
}

impl Interface {
    pub fn new(server_ctx: ServerContext) -> Self {
        let method_not_implemented =
            Arc::new(Method::new(|_, _, karlsend_request: KarlsendRequest| {
                Box::pin(async move {
                    match karlsend_request.payload {
                        Some(ref request) => {
                            Ok(KarlsendResponse {
                                id: karlsend_request.id,
                                payload: Some(KarlsendPayloadOps::from(request).to_error_response(
                                    GrpcServerError::MethodNotImplemented.into(),
                                )),
                            })
                        }
                        None => Err(GrpcServerError::InvalidRequestPayload),
                    }
                })
            }));
        Self {
            server_ctx,
            methods: Default::default(),
            method_not_implemented,
        }
    }

    pub fn method(&mut self, op: KarlsendPayloadOps, method: KarlsendMethod) {
        let method: DynKarlsendMethod = Arc::new(method);
        if self.methods.insert(op, method).is_some() {
            panic!("RPC method {op:?} is declared multiple times")
        }
    }

    pub fn replace_method(&mut self, op: KarlsendPayloadOps, method: KarlsendMethod) {
        let method: DynKarlsendMethod = Arc::new(method);
        let _ = self.methods.insert(op, method);
    }

    pub fn set_method_properties(
        &mut self,
        op: KarlsendPayloadOps,
        tasks: usize,
        queue_size: usize,
        routing_policy: KarlsendRoutingPolicy,
    ) {
        self.methods.entry(op).and_modify(|x| {
            let method: Method<ServerContext, Connection, KarlsendRequest, KarlsendResponse> =
                Method::with_properties(x.method_fn(), tasks, queue_size, routing_policy);
            let method: Arc<
                dyn MethodTrait<ServerContext, Connection, KarlsendRequest, KarlsendResponse>,
            > = Arc::new(method);
            *x = method;
        });
    }

    pub async fn call(
        &self,
        op: &KarlsendPayloadOps,
        connection: Connection,
        request: KarlsendRequest,
    ) -> GrpcServerResult<KarlsendResponse> {
        self.methods
            .get(op)
            .unwrap_or(&self.method_not_implemented)
            .call(self.server_ctx.clone(), connection, request)
            .await
    }

    pub fn get_method(&self, op: &KarlsendPayloadOps) -> DynKarlsendMethod {
        self.methods
            .get(op)
            .unwrap_or(&self.method_not_implemented)
            .clone()
    }
}

impl Debug for Interface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Interface").finish()
    }
}
