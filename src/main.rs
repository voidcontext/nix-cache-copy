use std::{
    io::{self, BufRead},
    time::Duration,
};
use tokio::sync::mpsc::{self, Sender};

use crate::parser::Line;

mod parser;
mod worker;

#[tokio::main]
async fn main() {
    let stdin = io::stdin();

    run_program(stdin.lock().lines()).await;
}

async fn run_program<R, E: std::fmt::Debug>(input: R)
where
    R: Iterator<Item = Result<String, E>>,
{
    let (tx, rx) = mpsc::channel::<Line>(1000);
    let (signal_tx, signal_rx) = mpsc::channel::<()>(10);

    let worker = worker::spawn(rx, signal_rx, |data| {
        Box::pin(async move {
            println!("processing line: {data:?}");
            tokio::time::sleep(Duration::from_millis(50)).await;
            println!("processing line done: {data:?}");
        })
    });

    process_stdin(input, tx).await.unwrap();

    // send a "close" signal to stop the receiver since we don't intend to send any more messages
    signal_tx.send(()).await.unwrap();

    // the worker will stop once the last message was processed
    worker.await.unwrap();
}

// So, the idea here is that nix build will echo the derivations that are being copied and being
// built. We assume that the build/copy is completed once we receive the next line or the end of
// the input
async fn process_stdin<R, E: std::fmt::Debug>(input: R, tx: Sender<Line>) -> Result<(), E>
where
    R: Iterator<Item = Result<String, E>>,
{
    let mut prev: Option<Line> = None;
    for line in input {
        let line = line?;
        // echo back the line on the stdout
        println!("{line}");

        // parse and send to the worker
        let line = Line::parse(&line);

        if let Some(prev_line) = prev {
            tx.send(prev_line).await.unwrap();
        }

        prev = Some(line);
    }

    if let Some(prev_line) = prev {
        tx.send(prev_line).await.unwrap();
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use tokio::sync::mpsc;

    use crate::{parser::Line, process_stdin};

    #[derive(Debug)]
    struct Error;

    #[tokio::test]
    async fn test_process_stdin_delays_sending_1() {
        let (tx, mut rx) = mpsc::channel(100);

        let input = vec![Ok(String::from("one")), Err(Error)];

        let result = process_stdin(input.into_iter(), tx).await;

        assert!(result.is_err());
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_process_stdin_delays_sending_2() {
        let (tx, mut rx) = mpsc::channel(100);

        let input = vec![Ok(String::from("one")), Ok(String::from("two")), Err(Error)];

        let result = process_stdin(input.into_iter(), tx).await;
        assert!(result.is_err());

        assert_eq!(rx.recv().await, Some(Line::Info(String::from("one"))));
        assert!(rx.try_recv().is_err());
    }
}
