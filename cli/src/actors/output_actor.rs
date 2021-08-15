use anyhow::Result;
use async_trait::async_trait;
use log::error;
use serde::Serialize;
use std::io::Write;
use std::marker::PhantomData;
use std::time::Duration;
use xactor::{message, Actor, Context, Handler, Message};

#[message]
#[derive(Clone)]
struct Flush;

pub struct OutputActor<W: Write, T: Serialize + Message<Result = ()>> {
    csv_writer: csv::Writer<W>,
    _phantom: PhantomData<T>,
}

impl<W: Write, T: Serialize + Message<Result = ()>> OutputActor<W, T> {
    pub fn new(writer: W) -> Self {
        OutputActor {
            csv_writer: csv::Writer::from_writer(writer),
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<W: Write + Send + 'static, T: Serialize + Message<Result = ()> + Send + 'static> Actor
    for OutputActor<W, T>
{
    async fn started(&mut self, ctx: &mut Context<Self>) -> Result<()> {
        ctx.send_interval(Flush, Duration::from_secs(15));
        ctx.subscribe::<T>().await?;
        Ok(())
    }
}

#[async_trait]
impl<W: Write + Send + 'static, T: Serialize + Message<Result = ()> + Send + 'static> Handler<Flush>
    for OutputActor<W, T>
{
    async fn handle(&mut self, _ctx: &mut Context<Self>, _msg: Flush) -> () {
        if let Err(e) = self.csv_writer.flush() {
            error!("Failed to flush writer: {:?}", e);
        }
    }
}

#[async_trait]
impl<W: Write + Send + 'static, T: Serialize + Send + Message<Result = ()> + 'static> Handler<T>
    for OutputActor<W, T>
{
    async fn handle(&mut self, ctx: &mut Context<Self>, msg: T) -> () {
        if let Err(e) = self.csv_writer.serialize(&msg) {
            error!(
                "Failed to serialize data for msg: {:?}. Retrying in 5 seconds",
                e
            );
            ctx.send_later(msg, Duration::from_secs(5));
        };
    }
}

#[cfg(test)]
mod tests {
    use super::{Flush, OutputActor};
    use serde::Serialize;
    use std::io::{Result, Write};
    use std::sync::{Arc, Mutex};
    use xactor::{message, Actor};

    struct MockWriter {
        buf: Arc<Mutex<Vec<u8>>>,
        flush: Arc<Mutex<i32>>,
    }

    impl Write for MockWriter {
        fn write(&mut self, buf: &[u8]) -> Result<usize> {
            let mut data = self.buf.lock().unwrap();
            data.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> Result<()> {
            let mut data = self.flush.lock().unwrap();
            *data += 1;
            Ok(())
        }
    }

    impl MockWriter {
        fn new(buf: Arc<Mutex<Vec<u8>>>, flush: Arc<Mutex<i32>>) -> Self {
            MockWriter { buf, flush }
        }
    }

    #[message]
    #[derive(Serialize)]
    struct MockSerializable {
        test: String,
        other_test: i32,
    }

    #[async_std::test]
    async fn output_actor_emits_csv_record_with_headers_on_initial_message() {
        let buffer = Arc::new(Mutex::new(vec![]));
        let flush = Arc::new(Mutex::new(0));
        let mock_writer: MockWriter = MockWriter::new(buffer.clone(), flush);
        let output_actor = OutputActor::new(mock_writer);

        let mock_serializable = MockSerializable {
            test: "test".to_owned(),
            other_test: 12,
        };

        let mut addr = output_actor.start().await.unwrap();
        addr.call(mock_serializable).await.unwrap();
        addr.stop(None).unwrap();
        addr.wait_for_stop().await;
        assert!(buffer
            .lock()
            .unwrap()
            .starts_with(b"test,other_test\ntest,12"));
    }

    #[async_std::test]
    async fn output_actor_flushes_on_flush_message() {
        let buffer = Arc::new(Mutex::new(vec![]));
        let flush = Arc::new(Mutex::new(0));
        let mock_writer: MockWriter = MockWriter::new(buffer.clone(), flush.clone());
        let output_actor: OutputActor<_, MockSerializable> = OutputActor::new(mock_writer);

        let mut addr = output_actor.start().await.unwrap();
        addr.call(Flush).await.unwrap();
        addr.stop(None).unwrap();
        addr.wait_for_stop().await;
        let flush_count = flush.lock().unwrap();
        assert!(*flush_count > 0);
    }

    #[async_std::test]
    async fn output_actor_doesnt_emit_new_header_per_message() {
        let buffer = Arc::new(Mutex::new(vec![]));
        let flush = Arc::new(Mutex::new(0));
        let mock_writer: MockWriter = MockWriter::new(buffer.clone(), flush);
        let output_actor = OutputActor::new(mock_writer);

        let mock_serializable_one = MockSerializable {
            test: "test".to_owned(),
            other_test: 12,
        };
        let mock_serializable_two = MockSerializable {
            test: "test2".to_owned(),
            other_test: 13,
        };

        let mut addr = output_actor.start().await.unwrap();
        addr.call(mock_serializable_one).await.unwrap();
        addr.call(mock_serializable_two).await.unwrap();

        addr.stop(None).unwrap();
        addr.wait_for_stop().await;
        assert!(buffer
            .lock()
            .unwrap()
            .starts_with(b"test,other_test\ntest,12\ntest2,13"));
    }
}
