// Workaround for: https://github.com/rust-lang/rust/issues/64450
extern crate builder;

mod common {
    pub use std::collections::{HashMap as Map, HashSet as Set};
    pub use std::convert::{TryFrom, TryInto};
    pub use std::fmt::{Display, Formatter};
    pub use std::fmt::{Error as FmtError, Result as FmtResult};
    pub use std::str::FromStr;

    pub use anyhow::Context as ResultContext;
    pub use anyhow::{bail, format_err, Error, Result};

    pub use chrono::{Datelike, Duration, NaiveDate as Date, TimeZone, Utc};
    pub type DateTime<Tz = Utc> = chrono::DateTime<Tz>;

    pub use futures::{Future, Stream, TryFuture};
    pub use json::json;
    pub use serde::{Deserialize, Deserializer, Serialize, Serializer};
    pub use tokio_compat::FutureExt as TokioCompatFutureExt;
}

pub mod api;

pub mod app;
pub use app::*;

pub mod auth;
pub use auth::*;
