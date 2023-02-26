use serenity::model::prelude::*;
use std::fmt::{Display, Formatter};
use serenity::framework::standard::CommandResult;
use crate::models::{RoleAssociation, Shim};
use wither::{
    mongodb::Database,
    bson::doc,
    Model,
};
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
    Emoji(EmojiId, bool),
}

macro_rules! mention {
    ($i:ident: $t:ty, $e:expr) => {
        impl From<$t> for Mentionable {
            #[inline(always)]
            fn from($i: $t) -> Self {
                $e.into()
            }
        }
    };
}

mention!(value: MentionableImpl, Mentionable(value));
mention!(value: &'_ Channel, MentionableImpl::Channel(value.id()));
mention!(value: ChannelId, MentionableImpl::Channel(value));
mention!(value: &'_ ChannelCategory, MentionableImpl::Channel(value.id));
mention!(value: &'_ GuildChannel, MentionableImpl::Channel(value.id));
mention!(value: &'_ PrivateChannel, MentionableImpl::Channel(value.id));
mention!(value: &'_ CurrentUser, MentionableImpl::User(value.id));
mention!(value: &'_ Member, MentionableImpl::User(value.user.id));
mention!(value: UserId, MentionableImpl::User(value));
mention!(value: &'_ User, MentionableImpl::User(value.id));
mention!(value: RoleId, MentionableImpl::Role(value));
mention!(value: &'_ Role, MentionableImpl::Role(value.id));
mention!(value: EmojiId, MentionableImpl::Emoji(value, false));
mention!(value: (EmojiId, bool), MentionableImpl::Emoji(value.0, value.1));
mention!(value: &'_ Emoji, MentionableImpl::Emoji(value.id, value.animated));

impl Display for Mentionable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            MentionableImpl::Channel(id) =>
                f.write_fmt(format_args!("<#{}>", id.0)),
            MentionableImpl::User(id) =>
                f.write_fmt(format_args!("<@{}>", id.0)),
            MentionableImpl::Role(id) =>
                f.write_fmt(format_args!("<@&{}>", id.0)),
            MentionableImpl::Emoji(id, animated) =>
                f.write_fmt(format_args!("<{}:_:{}>", if animated { "a" } else { "" }, id.0)),
        }
    }
}

pub struct OptionalDisplay<T>(pub Option<T>);

impl<T: Display> Display for OptionalDisplay<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(value) = &self.0 {
            value.fmt(f)
        } else {
            Ok(())
        }
    }
}
