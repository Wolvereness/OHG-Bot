use std::future::Future;

use once_cell::sync::OnceCell;
use wither::{
    mongodb::Database,
    Model,
    bson::{
        doc,
        oid::ObjectId,
    },
};
use serde::{
    Serialize,
    Deserialize,
};
use crate::Error;

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LazyDB<T> {
    id: ObjectId,
    #[serde(skip, default = "OnceCell::new")]
    contents: OnceCell<T>,
}

impl<T: Model + Send + Sync> LazyDB<T> {
    pub async fn make(value: T, db: &Database) -> Result<Self, Error> {
        let mut value = match Self::try_from(value) {
            Ok(value) => return Ok(value),
            Err(value) => value,
        };
        value.save(db, None).await?;
        Self::try_from(value)
            .map_err(|_| "Id not saved".into())
    }

    pub fn take<'d>(
        self,
        db: &'d Database,
    ) -> Result<
        T,
        impl 'd + Future<Output=Result<T, Error>>
    > {
        let LazyDB {
            id,
            contents,
        } = self;
        if let Some(contents) = contents.into_inner() {
            return Ok(contents);
        }

        Err(async move {
            match T::find_one(
                db,
                Some(doc! {
                    "_id": &id,
                }),
                None,
            ).await {
                Ok(Some(v)) =>
                    Ok(v),
                Err(e) =>
                    Err(e.into()),
                _ =>
                    Err("Value missing".into()),
            }
        })
    }

    pub fn get<'d: 'f, 's: 'f, 'f>(
        &'s self,
        db: &'d Database,
    ) -> Result<
        &'s T,
        impl 'f + Future<Output=Result<&'s T, Error>>,
    > {
        if let Some(contents) = self.contents.get() {
            return Ok(contents)
        }

        Err(async move {
            match T::find_one(
                db,
                Some(doc! {
                    "_id": &self.id,
                }),
                None,
            ).await {
                Ok(Some(v)) =>
                    Ok(self.contents.get_or_init(|| v)),
                Err(e) =>
                    Err(e.into()),
                _ =>
                    Err("Value missing".into()),
            }
        })
    }

    pub fn get_mut<'d: 'f, 's: 'f, 'f>(
        &'s mut self,
        db: &'d Database,
    ) -> Result<
        &'s mut T,
        impl 'f + Future<Output=Result<&'s mut T, Error>>,
    > {
        if self.contents.get().is_some() {
            if let Some(contents) = self.contents.get_mut() {
                Ok(contents)
            } else {
                unreachable!()
            }
        } else {
            Err(async move {
                let value = T::find_one(
                    db,
                    Some(doc! {
                        "_id": &self.id,
                    }),
                    None,
                ).await;
                match value {
                    Ok(Some(v)) => {
                        self.contents.get_or_init(|| v);
                        Ok(self.contents.get_mut().unwrap())
                    },
                    Err(e) =>
                        Err(e.into()),
                    _ =>
                        Err("Value missing".into()),
                }
            })
        }
    }

    pub async fn save_inner(&mut self, db: &Database) -> Result<(), Error> {
        let id = ObjectId::new();
        self.id = id.clone();
        let inner = match self.get_mut(db) {
            Ok(inner) => inner,
            Err(future) => future.await?,
        };
        inner.set_id(id);
        inner.save(db, None).await?;
        Ok(())
    }

    #[inline(always)]
    pub fn inner(self) -> Option<T> {
        self.contents.into_inner()
    }

    #[inline(always)]
    pub fn inner_ref(&self) -> Option<&T> {
        self.contents.get()
    }

    #[inline(always)]
    pub fn inner_mut(&mut self) -> Option<&mut T> {
        self.contents.get_mut()
    }
}

impl<T: Model> LazyDB<T> {
    pub fn try_from(value: T) -> Result<Self, T> {
        if let Some(id) = value.id() {
            Ok(LazyDB {
                id,
                contents: OnceCell::from(value),
            })
        } else {
            Err(value)
        }
    }
}

impl<T: Clone> Clone for LazyDB<T> {
    fn clone(&self) -> Self {
        LazyDB {
            id: self.id.clone(),
            contents: self
                .contents
                .get()
                .cloned()
                .map(OnceCell::from)
                .unwrap_or_default()
        }
    }
}

impl<T> LazyDB<T> {
    pub fn shallow_clone(&self) -> Self {
        LazyDB {
            id: self.id.clone(),
            contents: OnceCell::new(),
        }
    }
}
