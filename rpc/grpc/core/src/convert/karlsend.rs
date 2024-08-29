use crate::protowire::{karlsend_request, KarlsendRequest, KarlsendResponse};

impl From<karlsend_request::Payload> for KarlsendRequest {
    fn from(item: karlsend_request::Payload) -> Self {
        KarlsendRequest {
            id: 0,
            payload: Some(item),
        }
    }
}

impl AsRef<KarlsendRequest> for KarlsendRequest {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsRef<KarlsendResponse> for KarlsendResponse {
    fn as_ref(&self) -> &Self {
        self
    }
}

pub mod karlsend_request_convert {
    use crate::protowire::*;
    use karlsen_rpc_core::{RpcError, RpcResult};

    impl_into_karlsend_request!(Shutdown);
    impl_into_karlsend_request!(SubmitBlock);
    impl_into_karlsend_request!(GetBlockTemplate);
    impl_into_karlsend_request!(GetBlock);
    impl_into_karlsend_request!(GetInfo);

    impl_into_karlsend_request!(GetCurrentNetwork);
    impl_into_karlsend_request!(GetPeerAddresses);
    impl_into_karlsend_request!(GetSink);
    impl_into_karlsend_request!(GetMempoolEntry);
    impl_into_karlsend_request!(GetMempoolEntries);
    impl_into_karlsend_request!(GetConnectedPeerInfo);
    impl_into_karlsend_request!(AddPeer);
    impl_into_karlsend_request!(SubmitTransaction);
    impl_into_karlsend_request!(GetSubnetwork);
    impl_into_karlsend_request!(GetVirtualChainFromBlock);
    impl_into_karlsend_request!(GetBlocks);
    impl_into_karlsend_request!(GetBlockCount);
    impl_into_karlsend_request!(GetBlockDagInfo);
    impl_into_karlsend_request!(ResolveFinalityConflict);
    impl_into_karlsend_request!(GetHeaders);
    impl_into_karlsend_request!(GetUtxosByAddresses);
    impl_into_karlsend_request!(GetBalanceByAddress);
    impl_into_karlsend_request!(GetBalancesByAddresses);
    impl_into_karlsend_request!(GetSinkBlueScore);
    impl_into_karlsend_request!(Ban);
    impl_into_karlsend_request!(Unban);
    impl_into_karlsend_request!(EstimateNetworkHashesPerSecond);
    impl_into_karlsend_request!(GetMempoolEntriesByAddresses);
    impl_into_karlsend_request!(GetCoinSupply);
    impl_into_karlsend_request!(Ping);
    impl_into_karlsend_request!(GetMetrics);
    impl_into_karlsend_request!(GetServerInfo);
    impl_into_karlsend_request!(GetSyncStatus);
    impl_into_karlsend_request!(GetDaaScoreTimestampEstimate);

    impl_into_karlsend_request!(NotifyBlockAdded);
    impl_into_karlsend_request!(NotifyNewBlockTemplate);
    impl_into_karlsend_request!(NotifyUtxosChanged);
    impl_into_karlsend_request!(NotifyPruningPointUtxoSetOverride);
    impl_into_karlsend_request!(NotifyFinalityConflict);
    impl_into_karlsend_request!(NotifyVirtualDaaScoreChanged);
    impl_into_karlsend_request!(NotifyVirtualChainChanged);
    impl_into_karlsend_request!(NotifySinkBlueScoreChanged);

    macro_rules! impl_into_karlsend_request {
        ($name:tt) => {
            paste::paste! {
                impl_into_karlsend_request_ex!(karlsen_rpc_core::[<$name Request>],[<$name RequestMessage>],[<$name Request>]);
            }
        };
    }

    use impl_into_karlsend_request;

    macro_rules! impl_into_karlsend_request_ex {
        // ($($core_struct:ident)::+, $($protowire_struct:ident)::+, $($variant:ident)::+) => {
        ($core_struct:path, $protowire_struct:ident, $variant:ident) => {
            // ----------------------------------------------------------------------------
            // rpc_core to protowire
            // ----------------------------------------------------------------------------

            impl From<&$core_struct> for karlsend_request::Payload {
                fn from(item: &$core_struct) -> Self {
                    Self::$variant(item.into())
                }
            }

            impl From<&$core_struct> for KarlsendRequest {
                fn from(item: &$core_struct) -> Self {
                    Self {
                        id: 0,
                        payload: Some(item.into()),
                    }
                }
            }

            impl From<$core_struct> for karlsend_request::Payload {
                fn from(item: $core_struct) -> Self {
                    Self::$variant((&item).into())
                }
            }

            impl From<$core_struct> for KarlsendRequest {
                fn from(item: $core_struct) -> Self {
                    Self {
                        id: 0,
                        payload: Some((&item).into()),
                    }
                }
            }

            // ----------------------------------------------------------------------------
            // protowire to rpc_core
            // ----------------------------------------------------------------------------

            impl TryFrom<&karlsend_request::Payload> for $core_struct {
                type Error = RpcError;
                fn try_from(item: &karlsend_request::Payload) -> RpcResult<Self> {
                    if let karlsend_request::Payload::$variant(request) = item {
                        request.try_into()
                    } else {
                        Err(RpcError::MissingRpcFieldError(
                            "Payload".to_string(),
                            stringify!($variant).to_string(),
                        ))
                    }
                }
            }

            impl TryFrom<&KarlsendRequest> for $core_struct {
                type Error = RpcError;
                fn try_from(item: &KarlsendRequest) -> RpcResult<Self> {
                    item.payload
                        .as_ref()
                        .ok_or(RpcError::MissingRpcFieldError(
                            "KarlsenRequest".to_string(),
                            "Payload".to_string(),
                        ))?
                        .try_into()
                }
            }

            impl From<$protowire_struct> for KarlsendRequest {
                fn from(item: $protowire_struct) -> Self {
                    Self {
                        id: 0,
                        payload: Some(karlsend_request::Payload::$variant(item)),
                    }
                }
            }

            impl From<$protowire_struct> for karlsend_request::Payload {
                fn from(item: $protowire_struct) -> Self {
                    karlsend_request::Payload::$variant(item)
                }
            }
        };
    }
    use impl_into_karlsend_request_ex;
}

pub mod karlsend_response_convert {
    use crate::protowire::*;
    use karlsen_rpc_core::{RpcError, RpcResult};

    impl_into_karlsend_response!(Shutdown);
    impl_into_karlsend_response!(SubmitBlock);
    impl_into_karlsend_response!(GetBlockTemplate);
    impl_into_karlsend_response!(GetBlock);
    impl_into_karlsend_response!(GetInfo);
    impl_into_karlsend_response!(GetCurrentNetwork);

    impl_into_karlsend_response!(GetPeerAddresses);
    impl_into_karlsend_response!(GetSink);
    impl_into_karlsend_response!(GetMempoolEntry);
    impl_into_karlsend_response!(GetMempoolEntries);
    impl_into_karlsend_response!(GetConnectedPeerInfo);
    impl_into_karlsend_response!(AddPeer);
    impl_into_karlsend_response!(SubmitTransaction);
    impl_into_karlsend_response!(GetSubnetwork);
    impl_into_karlsend_response!(GetVirtualChainFromBlock);
    impl_into_karlsend_response!(GetBlocks);
    impl_into_karlsend_response!(GetBlockCount);
    impl_into_karlsend_response!(GetBlockDagInfo);
    impl_into_karlsend_response!(ResolveFinalityConflict);
    impl_into_karlsend_response!(GetHeaders);
    impl_into_karlsend_response!(GetUtxosByAddresses);
    impl_into_karlsend_response!(GetBalanceByAddress);
    impl_into_karlsend_response!(GetBalancesByAddresses);
    impl_into_karlsend_response!(GetSinkBlueScore);
    impl_into_karlsend_response!(Ban);
    impl_into_karlsend_response!(Unban);
    impl_into_karlsend_response!(EstimateNetworkHashesPerSecond);
    impl_into_karlsend_response!(GetMempoolEntriesByAddresses);
    impl_into_karlsend_response!(GetCoinSupply);
    impl_into_karlsend_response!(Ping);
    impl_into_karlsend_response!(GetMetrics);
    impl_into_karlsend_response!(GetServerInfo);
    impl_into_karlsend_response!(GetSyncStatus);
    impl_into_karlsend_response!(GetDaaScoreTimestampEstimate);

    impl_into_karlsend_notify_response!(NotifyBlockAdded);
    impl_into_karlsend_notify_response!(NotifyNewBlockTemplate);
    impl_into_karlsend_notify_response!(NotifyUtxosChanged);
    impl_into_karlsend_notify_response!(NotifyPruningPointUtxoSetOverride);
    impl_into_karlsend_notify_response!(NotifyFinalityConflict);
    impl_into_karlsend_notify_response!(NotifyVirtualDaaScoreChanged);
    impl_into_karlsend_notify_response!(NotifyVirtualChainChanged);
    impl_into_karlsend_notify_response!(NotifySinkBlueScoreChanged);

    impl_into_karlsend_notify_response!(NotifyUtxosChanged, StopNotifyingUtxosChanged);
    impl_into_karlsend_notify_response!(
        NotifyPruningPointUtxoSetOverride,
        StopNotifyingPruningPointUtxoSetOverride
    );

    macro_rules! impl_into_karlsend_response {
        ($name:tt) => {
            paste::paste! {
                impl_into_karlsend_response_ex!(karlsen_rpc_core::[<$name Response>],[<$name ResponseMessage>],[<$name Response>]);
            }
        };
        ($core_name:tt, $protowire_name:tt) => {
            paste::paste! {
                impl_into_karlsend_response_base!(karlsen_rpc_core::[<$core_name Response>],[<$protowire_name ResponseMessage>],[<$protowire_name Response>]);
            }
        };
    }
    use impl_into_karlsend_response;

    macro_rules! impl_into_karlsend_response_base {
        ($core_struct:path, $protowire_struct:ident, $variant:ident) => {
            // ----------------------------------------------------------------------------
            // rpc_core to protowire
            // ----------------------------------------------------------------------------

            impl From<RpcResult<$core_struct>> for $protowire_struct {
                fn from(item: RpcResult<$core_struct>) -> Self {
                    item.as_ref().map_err(|x| (*x).clone()).into()
                }
            }

            impl From<RpcError> for $protowire_struct {
                fn from(item: RpcError) -> Self {
                    let x: RpcResult<&$core_struct> = Err(item);
                    x.into()
                }
            }

            impl From<$protowire_struct> for karlsend_response::Payload {
                fn from(item: $protowire_struct) -> Self {
                    karlsend_response::Payload::$variant(item)
                }
            }

            impl From<$protowire_struct> for KarlsendResponse {
                fn from(item: $protowire_struct) -> Self {
                    Self {
                        id: 0,
                        payload: Some(karlsend_response::Payload::$variant(item)),
                    }
                }
            }
        };
    }
    use impl_into_karlsend_response_base;

    macro_rules! impl_into_karlsend_response_ex {
        ($core_struct:path, $protowire_struct:ident, $variant:ident) => {
            // ----------------------------------------------------------------------------
            // rpc_core to protowire
            // ----------------------------------------------------------------------------

            impl From<RpcResult<&$core_struct>> for karlsend_response::Payload {
                fn from(item: RpcResult<&$core_struct>) -> Self {
                    karlsend_response::Payload::$variant(item.into())
                }
            }

            impl From<RpcResult<&$core_struct>> for KarlsendResponse {
                fn from(item: RpcResult<&$core_struct>) -> Self {
                    Self {
                        id: 0,
                        payload: Some(item.into()),
                    }
                }
            }

            impl From<RpcResult<$core_struct>> for karlsend_response::Payload {
                fn from(item: RpcResult<$core_struct>) -> Self {
                    karlsend_response::Payload::$variant(item.into())
                }
            }

            impl From<RpcResult<$core_struct>> for KarlsendResponse {
                fn from(item: RpcResult<$core_struct>) -> Self {
                    Self {
                        id: 0,
                        payload: Some(item.into()),
                    }
                }
            }

            impl_into_karlsend_response_base!($core_struct, $protowire_struct, $variant);

            // ----------------------------------------------------------------------------
            // protowire to rpc_core
            // ----------------------------------------------------------------------------

            impl TryFrom<&karlsend_response::Payload> for $core_struct {
                type Error = RpcError;
                fn try_from(item: &karlsend_response::Payload) -> RpcResult<Self> {
                    if let karlsend_response::Payload::$variant(response) = item {
                        response.try_into()
                    } else {
                        Err(RpcError::MissingRpcFieldError(
                            "Payload".to_string(),
                            stringify!($variant).to_string(),
                        ))
                    }
                }
            }

            impl TryFrom<&KarlsendResponse> for $core_struct {
                type Error = RpcError;
                fn try_from(item: &KarlsendResponse) -> RpcResult<Self> {
                    item.payload
                        .as_ref()
                        .ok_or(RpcError::MissingRpcFieldError(
                            "KarlsenResponse".to_string(),
                            "Payload".to_string(),
                        ))?
                        .try_into()
                }
            }
        };
    }
    use impl_into_karlsend_response_ex;

    macro_rules! impl_into_karlsend_notify_response {
        ($name:tt) => {
            impl_into_karlsend_response!($name);

            paste::paste! {
                impl_into_karlsend_notify_response_ex!(karlsen_rpc_core::[<$name Response>],[<$name ResponseMessage>]);
            }
        };
        ($core_name:tt, $protowire_name:tt) => {
            impl_into_karlsend_response!($core_name, $protowire_name);

            paste::paste! {
                impl_into_karlsend_notify_response_ex!(karlsen_rpc_core::[<$core_name Response>],[<$protowire_name ResponseMessage>]);
            }
        };
    }
    use impl_into_karlsend_notify_response;

    macro_rules! impl_into_karlsend_notify_response_ex {
        ($($core_struct:ident)::+, $protowire_struct:ident) => {
            // ----------------------------------------------------------------------------
            // rpc_core to protowire
            // ----------------------------------------------------------------------------

            impl<T> From<Result<(), T>> for $protowire_struct
            where
                T: Into<RpcError>,
            {
                fn from(item: Result<(), T>) -> Self {
                    item
                        .map(|_| $($core_struct)::+{})
                        .map_err(|err| err.into()).into()
                }
            }

        };
    }
    use impl_into_karlsend_notify_response_ex;
}
