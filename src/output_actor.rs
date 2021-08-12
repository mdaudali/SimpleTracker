use std::io::Write;
use xactor::{Message, Handler, Actor, Context};
use serde::Serialize;
use async_trait::async_trait;

pub struct Output<T: Serialize>(T);
impl <T: Serialize> Output<T> {
    pub fn of(serializable: T) -> Self {
        Output(serializable)
    }

    pub fn to_inner(&self) -> &T {
        &self.0
    }
}
impl <T: Serialize + Send + 'static> Message for Output<T> {
    type Result = ();
}

pub struct OutputActor<W: Write>{
    csv_writer: csv::Writer<W>
}

impl <W:Write> OutputActor<W> {
    pub fn of(writer: W) -> Self {
        OutputActor{
            csv_writer: csv::Writer::from_writer(writer)
        }
    }

}

impl <W: Write + Send + 'static> Actor for OutputActor<W> {}

#[async_trait]
impl <T: Serialize + Send + 'static, W: Write + Send + 'static> Handler<Output<T>> for OutputActor<W> {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: Output<T>) -> () {
        self.csv_writer.serialize(msg.to_inner()).unwrap();
        self.csv_writer.flush().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::{Output, OutputActor};
    use std::io::{Result, Write};
    use serde::Serialize;
    use std::sync::{Arc, Mutex};
    use xactor::Actor;

    struct MockWriter {
        buf: Arc<Mutex<Vec<u8>>>
    }
    
    impl Write for MockWriter {
        fn write(&mut self, buf: &[u8]) -> Result<usize> {
            let mut data = self.buf.lock().unwrap();
            data.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> Result<()> {
            Ok(())
        }
    }

    impl MockWriter {
        fn new(buf: Arc<Mutex<Vec<u8>>>) -> Self {
            MockWriter{
                buf
            }
        }
    }

    #[derive(Serialize)]
    struct MockSerializable {
        test: String,
        other_test: i32
    }
    #[async_std::test]
    async fn output_actor_emits_csv_record_with_headers_on_initial_message(){
        let buffer = Arc::new(Mutex::new(vec![]));
        let mock_writer: MockWriter = MockWriter::new(buffer.clone());
        let output_actor = OutputActor::of(mock_writer);
    
        
        let mock_serializable = MockSerializable {
            test: "test".to_owned(),
            other_test: 12,
        };

        let mut addr = output_actor.start().await.unwrap();
        addr.call(Output(mock_serializable)).await.unwrap();
        addr.stop(None).unwrap();
        addr.wait_for_stop().await;
        assert!(buffer.lock().unwrap().starts_with(b"test,other_test\ntest,12"));
    }

    #[async_std::test]
    async fn output_actor_doesnt_emit_new_header_per_message() {
        let buffer = Arc::new(Mutex::new(vec![]));
        let mock_writer: MockWriter = MockWriter::new(buffer.clone());
        let output_actor = OutputActor::of(mock_writer);
    
        
        let mock_serializable_one = MockSerializable {
            test: "test".to_owned(),
            other_test: 12,
        };
        let mock_serializable_two = MockSerializable {
            test: "test2".to_owned(),
            other_test: 13
        };

        let mut addr = output_actor.start().await.unwrap();
        addr.call(Output(mock_serializable_one)).await.unwrap();
        addr.call(Output(mock_serializable_two)).await.unwrap();

        addr.stop(None).unwrap();
        addr.wait_for_stop().await;
        assert!(buffer.lock().unwrap().starts_with(b"test,other_test\ntest,12\ntest2,13"));
    }
}