use async_channel::Sender;
use karlsen_notify::notifier::Notify;
use karlsen_rpc_core::Notification;

#[derive(Debug)]
pub struct ChannelNotify {
    sender: Sender<Notification>,
}

impl ChannelNotify {
    pub fn new(sender: Sender<Notification>) -> Self {
        Self { sender }
    }
}

impl Notify<Notification> for ChannelNotify {
    fn notify(&self, notification: Notification) -> karlsen_notify::error::Result<()> {
        self.sender.try_send(notification)?;
        Ok(())
    }
}
