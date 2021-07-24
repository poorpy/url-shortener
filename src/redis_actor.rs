use actix::{Actor, Context, Handler, Message, ResponseFuture};
use redis::{aio::MultiplexedConnection, Client, RedisError};

pub struct RedisActor {
    conn: MultiplexedConnection,
}

impl RedisActor {
    pub async fn new(redis_url: &'static str) -> Result<Self, RedisError> {
        let client = Client::open(redis_url)?;
        let conn = client.get_multiplexed_async_connection().await?;
        Ok(RedisActor { conn })
    }
}

#[derive(Message, Debug)]
#[rtype(result = "Result<Option<String>, RedisError>")]
pub struct PutCommand {
    pub key: String,
    pub value: String,
}

impl Actor for RedisActor {
    type Context = Context<Self>;
}

impl Handler<PutCommand> for RedisActor {
    type Result = ResponseFuture<Result<Option<String>, RedisError>>;

    fn handle(&mut self, command: PutCommand, _: &mut Self::Context) -> Self::Result {
        let mut conn = self.conn.clone();
        let cmd: Box<redis::Cmd> = Box::new({
            let mut cmd = redis::cmd("SET");
                cmd.arg(command.key).arg(command.value);
                cmd
        });
        let future = async move { cmd.query_async(&mut conn).await };
        Box::pin(future)
    }
}
