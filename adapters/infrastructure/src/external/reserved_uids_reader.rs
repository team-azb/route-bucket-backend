use std::fs::read_to_string;

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use derivative::Derivative;
use once_cell::sync::Lazy;
use route_bucket_domain::{external::ReservedUserIdCheckerApi, model::user::UserId};
use route_bucket_utils::{ApplicationError, ApplicationResult};
use tokio::sync::RwLock;

const TEXT_PATH: &str = "adapters/infrastructure/resources/reserved_uids.txt";
static UPDATE_INTERVAL: Lazy<Duration> = Lazy::new(|| Duration::days(1));

#[derive(Derivative)]
#[derivative(Default)]
struct InnerReservedUidsReader {
    reserved_ids: Vec<UserId>,
    #[derivative(Default(value = "chrono::MIN_DATETIME"))]
    next_update_time: DateTime<Utc>,
}

impl InnerReservedUidsReader {
    fn new() -> ApplicationResult<Self> {
        let mut reader: Self = Default::default();
        reader.update()?;
        Ok(reader)
    }

    fn update(&mut self) -> ApplicationResult<()> {
        self.reserved_ids = read_to_string(TEXT_PATH)
            .map_err(|err| {
                ApplicationError::ExternalError(format!(
                    "Failed to read from {}! ({})",
                    TEXT_PATH, err
                ))
            })?
            .split('\n')
            .map(String::from)
            .map(UserId::from)
            .collect();
        self.next_update_time = Utc::now() + *UPDATE_INTERVAL;
        Ok(())
    }

    fn contains(&self, id: &UserId) -> bool {
        self.reserved_ids.contains(id)
    }
}

pub struct ReservedUidsReader(RwLock<InnerReservedUidsReader>);

impl ReservedUidsReader {
    pub fn new() -> ApplicationResult<Self> {
        InnerReservedUidsReader::new().map(RwLock::new).map(Self)
    }
}

#[async_trait]
impl ReservedUserIdCheckerApi for ReservedUidsReader {
    async fn check_if_reserved(&self, id: &UserId) -> ApplicationResult<bool> {
        // Write lock scope
        {
            let mut inner_reader = self.0.write().await;
            if inner_reader.next_update_time <= Utc::now() {
                inner_reader.update()?;
            }
        }

        Ok(self.0.read().await.contains(id))
    }
}
