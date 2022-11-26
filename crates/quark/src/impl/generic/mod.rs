//! Database agnostic implementations.

pub mod admin {
    pub mod migrations;
}

pub mod media {
    pub mod attachment;
    pub mod emoji;
}

pub mod channels {
    pub mod channel;
    pub mod channel_invite;
    pub mod channel_unread;
    pub mod message;
    pub mod webhook;
}

pub mod servers {
    pub mod server;
    pub mod server_ban;
    pub mod server_member;
}

pub mod users {
    pub mod bot;
    pub mod user;
    pub mod user_settings;
}
