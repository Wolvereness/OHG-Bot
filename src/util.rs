use serenity::model::prelude::*;
use std::fmt::{Display, Formatter};
use serenity::framework::standard::CommandResult;
use crate::models::{RoleAssociation, Shim};
use wither::mongodb::Database;
use wither::bson::doc;
use wither::Model;
use futures::TryStreamExt;

pub async fn get_role_associations(db: &Database, channel: ChannelId, guild: GuildId) -> CommandResult<Vec<RoleAssociation>> {
    RoleAssociation::find(
        db,
        Some(doc!{
                "$or": [
                    { "channel": doc!{ "$eq": &Shim::from(channel) } },
                    { "server": doc!{ "$eq": &Shim::from(guild) } },
                ],
            }),
        None,
    )
        .await?
        .try_collect()
        .await
        .map_err(Into::into)
}

#[derive(Debug, Clone, Copy)]
pub struct Mentionable(MentionableImpl);

#[derive(Debug, Clone, Copy)]
enum MentionableImpl {
    Channel(ChannelId),
    User(UserId),
    Role(RoleId),
}

impl From<MentionableImpl> for Mentionable {
    #[inline(always)]
    fn from(value: MentionableImpl) -> Self {
        Mentionable(value)
    }
}

impl From<&'_ Channel> for Mentionable {
    #[inline(always)]
    fn from(value: &Channel) -> Self {
        MentionableImpl::Channel(value.id()).into()
    }
}

impl From<ChannelId> for Mentionable {
    #[inline(always)]
    fn from(value: ChannelId) -> Self {
        MentionableImpl::Channel(value).into()
    }
}

impl From<&'_ ChannelCategory> for Mentionable {
    #[inline(always)]
    fn from(value: &'_ ChannelCategory) -> Self {
        MentionableImpl::Channel(value.id).into()
    }
}

impl From<&'_ GuildChannel> for Mentionable {
    #[inline(always)]
    fn from(value: &'_ GuildChannel) -> Self {
        MentionableImpl::Channel(value.id).into()
    }
}

impl From<&'_ PrivateChannel> for Mentionable {
    #[inline(always)]
    fn from(value: &'_ PrivateChannel) -> Self {
        MentionableImpl::Channel(value.id).into()
    }
}

impl From<&'_ CurrentUser> for Mentionable {
    #[inline(always)]
    fn from(value: &'_ CurrentUser) -> Self {
        MentionableImpl::User(value.id).into()
    }
}

impl From<&'_ Member> for Mentionable {
    #[inline(always)]
    fn from(value: &'_ Member) -> Self {
        MentionableImpl::User(value.user.id).into()
    }
}

impl From<UserId> for Mentionable {
    #[inline(always)]
    fn from(value: UserId) -> Self {
        MentionableImpl::User(value).into()
    }
}

impl From<&'_ User> for Mentionable {
    #[inline(always)]
    fn from(value: &'_ User) -> Self {
        MentionableImpl::User(value.id).into()
    }
}

impl From<RoleId> for Mentionable {
    #[inline(always)]
    fn from(value: RoleId) -> Self {
        MentionableImpl::Role(value).into()
    }
}

impl From<&'_ Role> for Mentionable {
    #[inline(always)]
    fn from(value: &'_ Role) -> Self {
        MentionableImpl::Role(value.id).into()
    }
}

impl Display for Mentionable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            MentionableImpl::Channel(id) =>
                f.write_fmt(format_args!("<#{}>", id.0)),
            MentionableImpl::User(id) =>
                f.write_fmt(format_args!("<@{}>", id.0)),
            MentionableImpl::Role(id) =>
                f.write_fmt(format_args!("<@&{}>", id.0)),
        }
    }
}
