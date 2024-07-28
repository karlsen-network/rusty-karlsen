use karlsen_notify::{scope::Scope, subscription::Command};

use crate::protowire::{
    karlsend_request, karlsend_response, KarlsendRequest, KarlsendResponse,
    NotifyBlockAddedRequestMessage, NotifyFinalityConflictRequestMessage,
    NotifyNewBlockTemplateRequestMessage, NotifyPruningPointUtxoSetOverrideRequestMessage,
    NotifySinkBlueScoreChangedRequestMessage, NotifyUtxosChangedRequestMessage,
    NotifyVirtualChainChangedRequestMessage, NotifyVirtualDaaScoreChangedRequestMessage,
};

impl KarlsendRequest {
    pub fn from_notification_type(scope: &Scope, command: Command) -> Self {
        KarlsendRequest {
            id: 0,
            payload: Some(karlsend_request::Payload::from_notification_type(
                scope, command,
            )),
        }
    }

    pub fn is_subscription(&self) -> bool {
        self.payload.as_ref().is_some_and(|x| x.is_subscription())
    }
}

impl karlsend_request::Payload {
    pub fn from_notification_type(scope: &Scope, command: Command) -> Self {
        match scope {
            Scope::BlockAdded(_) => {
                karlsend_request::Payload::NotifyBlockAddedRequest(NotifyBlockAddedRequestMessage {
                    command: command.into(),
                })
            }
            Scope::NewBlockTemplate(_) => karlsend_request::Payload::NotifyNewBlockTemplateRequest(
                NotifyNewBlockTemplateRequestMessage {
                    command: command.into(),
                },
            ),

            Scope::VirtualChainChanged(ref scope) => {
                karlsend_request::Payload::NotifyVirtualChainChangedRequest(
                    NotifyVirtualChainChangedRequestMessage {
                        command: command.into(),
                        include_accepted_transaction_ids: scope.include_accepted_transaction_ids,
                    },
                )
            }
            Scope::FinalityConflict(_) => karlsend_request::Payload::NotifyFinalityConflictRequest(
                NotifyFinalityConflictRequestMessage {
                    command: command.into(),
                },
            ),
            Scope::FinalityConflictResolved(_) => {
                karlsend_request::Payload::NotifyFinalityConflictRequest(
                    NotifyFinalityConflictRequestMessage {
                        command: command.into(),
                    },
                )
            }
            Scope::UtxosChanged(ref scope) => karlsend_request::Payload::NotifyUtxosChangedRequest(
                NotifyUtxosChangedRequestMessage {
                    addresses: scope
                        .addresses
                        .iter()
                        .map(|x| x.into())
                        .collect::<Vec<String>>(),
                    command: command.into(),
                },
            ),
            Scope::SinkBlueScoreChanged(_) => {
                karlsend_request::Payload::NotifySinkBlueScoreChangedRequest(
                    NotifySinkBlueScoreChangedRequestMessage {
                        command: command.into(),
                    },
                )
            }
            Scope::VirtualDaaScoreChanged(_) => {
                karlsend_request::Payload::NotifyVirtualDaaScoreChangedRequest(
                    NotifyVirtualDaaScoreChangedRequestMessage {
                        command: command.into(),
                    },
                )
            }
            Scope::PruningPointUtxoSetOverride(_) => {
                karlsend_request::Payload::NotifyPruningPointUtxoSetOverrideRequest(
                    NotifyPruningPointUtxoSetOverrideRequestMessage {
                        command: command.into(),
                    },
                )
            }
        }
    }

    pub fn is_subscription(&self) -> bool {
        use crate::protowire::karlsend_request::Payload;
        matches!(
            self,
            Payload::NotifyBlockAddedRequest(_)
                | Payload::NotifyVirtualChainChangedRequest(_)
                | Payload::NotifyFinalityConflictRequest(_)
                | Payload::NotifyUtxosChangedRequest(_)
                | Payload::NotifySinkBlueScoreChangedRequest(_)
                | Payload::NotifyVirtualDaaScoreChangedRequest(_)
                | Payload::NotifyPruningPointUtxoSetOverrideRequest(_)
                | Payload::NotifyNewBlockTemplateRequest(_)
                | Payload::StopNotifyingUtxosChangedRequest(_)
                | Payload::StopNotifyingPruningPointUtxoSetOverrideRequest(_)
        )
    }
}

impl KarlsendResponse {
    pub fn is_notification(&self) -> bool {
        match self.payload {
            Some(ref payload) => payload.is_notification(),
            None => false,
        }
    }
}

#[allow(clippy::match_like_matches_macro)]
impl karlsend_response::Payload {
    pub fn is_notification(&self) -> bool {
        use crate::protowire::karlsend_response::Payload;
        match self {
            Payload::BlockAddedNotification(_) => true,
            Payload::VirtualChainChangedNotification(_) => true,
            Payload::FinalityConflictNotification(_) => true,
            Payload::FinalityConflictResolvedNotification(_) => true,
            Payload::UtxosChangedNotification(_) => true,
            Payload::SinkBlueScoreChangedNotification(_) => true,
            Payload::VirtualDaaScoreChangedNotification(_) => true,
            Payload::PruningPointUtxoSetOverrideNotification(_) => true,
            Payload::NewBlockTemplateNotification(_) => true,
            _ => false,
        }
    }
}
