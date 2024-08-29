use crate::pb::karlsend_message::Payload as KarlsendMessagePayload;

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum KarlsendMessagePayloadType {
    Addresses = 0,
    Block,
    Transaction,
    BlockLocator,
    RequestAddresses,
    RequestRelayBlocks,
    RequestTransactions,
    IbdBlock,
    InvRelayBlock,
    InvTransactions,
    Ping,
    Pong,
    Verack,
    Version,
    TransactionNotFound,
    Reject,
    PruningPointUtxoSetChunk,
    RequestIbdBlocks,
    UnexpectedPruningPoint,
    IbdBlockLocator,
    IbdBlockLocatorHighestHash,
    RequestNextPruningPointUtxoSetChunk,
    DonePruningPointUtxoSetChunks,
    IbdBlockLocatorHighestHashNotFound,
    BlockWithTrustedData,
    DoneBlocksWithTrustedData,
    RequestPruningPointAndItsAnticone,
    BlockHeaders,
    RequestNextHeaders,
    DoneHeaders,
    RequestPruningPointUtxoSet,
    RequestHeaders,
    RequestBlockLocator,
    PruningPoints,
    RequestPruningPointProof,
    PruningPointProof,
    Ready,
    BlockWithTrustedDataV4,
    TrustedData,
    RequestIbdChainBlockLocator,
    IbdChainBlockLocator,
    RequestAntipast,
    RequestNextPruningPointAndItsAnticoneBlocks,
}

impl From<&KarlsendMessagePayload> for KarlsendMessagePayloadType {
    fn from(payload: &KarlsendMessagePayload) -> Self {
        match payload {
            KarlsendMessagePayload::Addresses(_) => KarlsendMessagePayloadType::Addresses,
            KarlsendMessagePayload::Block(_) => KarlsendMessagePayloadType::Block,
            KarlsendMessagePayload::Transaction(_) => KarlsendMessagePayloadType::Transaction,
            KarlsendMessagePayload::BlockLocator(_) => KarlsendMessagePayloadType::BlockLocator,
            KarlsendMessagePayload::RequestAddresses(_) => {
                KarlsendMessagePayloadType::RequestAddresses
            }
            KarlsendMessagePayload::RequestRelayBlocks(_) => {
                KarlsendMessagePayloadType::RequestRelayBlocks
            }
            KarlsendMessagePayload::RequestTransactions(_) => {
                KarlsendMessagePayloadType::RequestTransactions
            }
            KarlsendMessagePayload::IbdBlock(_) => KarlsendMessagePayloadType::IbdBlock,
            KarlsendMessagePayload::InvRelayBlock(_) => KarlsendMessagePayloadType::InvRelayBlock,
            KarlsendMessagePayload::InvTransactions(_) => {
                KarlsendMessagePayloadType::InvTransactions
            }
            KarlsendMessagePayload::Ping(_) => KarlsendMessagePayloadType::Ping,
            KarlsendMessagePayload::Pong(_) => KarlsendMessagePayloadType::Pong,
            KarlsendMessagePayload::Verack(_) => KarlsendMessagePayloadType::Verack,
            KarlsendMessagePayload::Version(_) => KarlsendMessagePayloadType::Version,
            KarlsendMessagePayload::TransactionNotFound(_) => {
                KarlsendMessagePayloadType::TransactionNotFound
            }
            KarlsendMessagePayload::Reject(_) => KarlsendMessagePayloadType::Reject,
            KarlsendMessagePayload::PruningPointUtxoSetChunk(_) => {
                KarlsendMessagePayloadType::PruningPointUtxoSetChunk
            }
            KarlsendMessagePayload::RequestIbdBlocks(_) => {
                KarlsendMessagePayloadType::RequestIbdBlocks
            }
            KarlsendMessagePayload::UnexpectedPruningPoint(_) => {
                KarlsendMessagePayloadType::UnexpectedPruningPoint
            }
            KarlsendMessagePayload::IbdBlockLocator(_) => {
                KarlsendMessagePayloadType::IbdBlockLocator
            }
            KarlsendMessagePayload::IbdBlockLocatorHighestHash(_) => {
                KarlsendMessagePayloadType::IbdBlockLocatorHighestHash
            }
            KarlsendMessagePayload::RequestNextPruningPointUtxoSetChunk(_) => {
                KarlsendMessagePayloadType::RequestNextPruningPointUtxoSetChunk
            }
            KarlsendMessagePayload::DonePruningPointUtxoSetChunks(_) => {
                KarlsendMessagePayloadType::DonePruningPointUtxoSetChunks
            }
            KarlsendMessagePayload::IbdBlockLocatorHighestHashNotFound(_) => {
                KarlsendMessagePayloadType::IbdBlockLocatorHighestHashNotFound
            }
            KarlsendMessagePayload::BlockWithTrustedData(_) => {
                KarlsendMessagePayloadType::BlockWithTrustedData
            }
            KarlsendMessagePayload::DoneBlocksWithTrustedData(_) => {
                KarlsendMessagePayloadType::DoneBlocksWithTrustedData
            }
            KarlsendMessagePayload::RequestPruningPointAndItsAnticone(_) => {
                KarlsendMessagePayloadType::RequestPruningPointAndItsAnticone
            }
            KarlsendMessagePayload::BlockHeaders(_) => KarlsendMessagePayloadType::BlockHeaders,
            KarlsendMessagePayload::RequestNextHeaders(_) => {
                KarlsendMessagePayloadType::RequestNextHeaders
            }
            KarlsendMessagePayload::DoneHeaders(_) => KarlsendMessagePayloadType::DoneHeaders,
            KarlsendMessagePayload::RequestPruningPointUtxoSet(_) => {
                KarlsendMessagePayloadType::RequestPruningPointUtxoSet
            }
            KarlsendMessagePayload::RequestHeaders(_) => KarlsendMessagePayloadType::RequestHeaders,
            KarlsendMessagePayload::RequestBlockLocator(_) => {
                KarlsendMessagePayloadType::RequestBlockLocator
            }
            KarlsendMessagePayload::PruningPoints(_) => KarlsendMessagePayloadType::PruningPoints,
            KarlsendMessagePayload::RequestPruningPointProof(_) => {
                KarlsendMessagePayloadType::RequestPruningPointProof
            }
            KarlsendMessagePayload::PruningPointProof(_) => {
                KarlsendMessagePayloadType::PruningPointProof
            }
            KarlsendMessagePayload::Ready(_) => KarlsendMessagePayloadType::Ready,
            KarlsendMessagePayload::BlockWithTrustedDataV4(_) => {
                KarlsendMessagePayloadType::BlockWithTrustedDataV4
            }
            KarlsendMessagePayload::TrustedData(_) => KarlsendMessagePayloadType::TrustedData,
            KarlsendMessagePayload::RequestIbdChainBlockLocator(_) => {
                KarlsendMessagePayloadType::RequestIbdChainBlockLocator
            }
            KarlsendMessagePayload::IbdChainBlockLocator(_) => {
                KarlsendMessagePayloadType::IbdChainBlockLocator
            }
            KarlsendMessagePayload::RequestAntipast(_) => {
                KarlsendMessagePayloadType::RequestAntipast
            }
            KarlsendMessagePayload::RequestNextPruningPointAndItsAnticoneBlocks(_) => {
                KarlsendMessagePayloadType::RequestNextPruningPointAndItsAnticoneBlocks
            }
        }
    }
}
