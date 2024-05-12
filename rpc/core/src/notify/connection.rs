use crate::Notification;

pub type ChannelConnection = karlsen_notify::connection::ChannelConnection<Notification>;
pub use karlsen_notify::connection::ChannelType;
