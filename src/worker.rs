use std::{future::Future, pin::Pin};

use tokio::{sync::mpsc::Receiver, task::JoinHandle};

/// Spawns a worker task that reads from the given Receiver and applies `f` to the received value until a close signal
pub fn spawn<T, F>(mut rx: Receiver<T>, mut close_signal: Receiver<()>, f: F) -> JoinHandle<()>
where
    T: 'static + std::fmt::Debug + Send,
    F: Fn(T) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync + 'static,
{
    tokio::spawn(async move {
        while let Some(data) = rx.recv().await {
            f(data).await;

            // Check if we can close the receiver
            if close_signal.try_recv().is_ok() {
                println!("closing the receiver, waiting for the queue to be drained");
                rx.close();
            }
        }
    })
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use tokio::sync::mpsc;

    use super::spawn;

    #[tokio::test]
    async fn test_span_completes_all_tasks() {
        let (tx, rx) = mpsc::channel::<String>(1000);
        let (result_tx, mut result_rx) = mpsc::channel::<String>(1000);
        let (close_signal_tx, close_signal_rx) = mpsc::channel::<()>(10);

        tx.send(String::from("one")).await.unwrap();
        tx.send(String::from("two")).await.unwrap();
        close_signal_tx.send(()).await.unwrap();

        let worker = spawn(rx, close_signal_rx, move |msg| {
            let rtx = result_tx.clone();
            Box::pin(async move {
                tokio::time::sleep(Duration::from_millis(200)).await;
                rtx.send(msg).await.unwrap();
            })
        });

        // ensure worker completes successfully
        let worker_result = worker.await;
        assert!(worker_result.is_ok());

        // ensure worker completed all tasks
        let mut result = Vec::new();
        while let Some(data) = result_rx.recv().await {
            result.push(data);
        }

        assert_eq!(result, vec![String::from("one"), String::from("two")]);
    }
}
