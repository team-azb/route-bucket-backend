use async_trait::async_trait;
use futures::future::BoxFuture;

pub use route::{CallRouteRepository, RouteRepository};
use route_bucket_utils::ApplicationResult;

#[cfg(feature = "mocking")]
pub use route::MockRouteRepository;

pub(crate) mod route;

#[async_trait]
pub trait Connection: Sync {
    async fn begin_transaction(&self) -> ApplicationResult<()>;
    async fn commit_transaction(&self) -> ApplicationResult<()>;
    async fn rollback_transaction(&self) -> ApplicationResult<()>;

    // Fellow rustaceans helped me to make this work.
    // https://users.rust-lang.org/t/lifetime-may-not-live-long-enough-for-an-async-closure/62489
    // https://users.rust-lang.org/t/how-to-use-self-inside-an-async-closure/62540
    async fn transaction<'a, T, F>(&'a self, f: F) -> ApplicationResult<T>
    where
        T: Send,
        F: FnOnce(&'a Self) -> BoxFuture<'a, ApplicationResult<T>> + Send,
    {
        self.begin_transaction().await?;
        let result = f(self).await;

        if result.is_ok() {
            self.commit_transaction().await?;
        } else {
            self.rollback_transaction().await?;
        }
        result
    }
}

#[async_trait]
pub trait Repository: Send + Sync {
    type Connection: Connection + Send;

    async fn get_connection(&self) -> ApplicationResult<Self::Connection>;

    async fn begin_transaction(&self) -> ApplicationResult<Self::Connection> {
        let conn = self.get_connection().await?;
        conn.begin_transaction().await?;
        Ok(conn)
    }
}

// NOTE: Connectionにautomockをつけようとすると、transactionのライフタイムでバグるっぽいので、自前で作成
//     : ライフタイムパラメータのついたgenericsはそもそも言語使用的に捌けないっぽい（ほんまか）
// 参考　： https://github.com/asomers/mockall/issues/299#issuecomment-873669518
#[cfg(feature = "mocking")]
#[cfg_attr(feature = "mocking", derive(Clone, Debug, PartialEq))]
pub struct MockConnection {}

#[cfg(feature = "mocking")]
#[async_trait]
impl Connection for MockConnection {
    async fn begin_transaction(&self) -> ApplicationResult<()> {
        Ok(())
    }
    async fn commit_transaction(&self) -> ApplicationResult<()> {
        Ok(())
    }
    async fn rollback_transaction(&self) -> ApplicationResult<()> {
        Ok(())
    }
}
