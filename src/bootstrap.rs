use std::future;

use tokio::sync::mpsc::{self, Sender};

use crate::{nix::Cli, parser::Line, worker};

pub async fn run<R, E: std::fmt::Debug, NC>(input: R, nix_cli: NC)
where
    R: Iterator<Item = Result<String, E>>,
    NC: Cli + Clone + Send + Sync + 'static,
{
    let (tx, rx) = mpsc::channel::<Line>(1000);
    let (signal_tx, signal_rx) = mpsc::channel::<()>(10);

    let worker = worker::spawn(rx, signal_rx, move |data| {
        let nix_cli = nix_cli.clone();

        match data {
            Line::Info(_) => Box::pin(future::ready(())),
            Line::Copied(_, path) => Box::pin(async move {
                nix_cli.copy_store_path(&path).await.unwrap();
            }),
            Line::Built(_, drv_file) => Box::pin(async move {
                nix_cli.copy_drv_output(&drv_file).await.unwrap();
            }),
        }
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
    use std::sync::Arc;

    use async_trait::async_trait;
    use tokio::sync::{mpsc, Mutex};

    use super::{process_stdin, run};
    use crate::{nix::Cli, parser::Line, DrvFile, StorePath};

    #[derive(Debug)]
    struct Error;

    #[derive(Debug, PartialEq)]
    enum NixCliCall {
        CopyStorePath(StorePath),
        CopyDrvOurput(DrvFile),
    }

    #[derive(Clone)]
    struct MockNixCli {
        calls: Arc<Mutex<Vec<NixCliCall>>>,
    }

    impl MockNixCli {
        fn new() -> Self {
            Self {
                calls: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl Cli for MockNixCli {
        async fn copy_store_path(&self, path: &StorePath) -> anyhow::Result<()> {
            let mut calls = self.calls.lock().await;
            calls.push(NixCliCall::CopyStorePath((*path).clone()));

            Ok(())
        }
        async fn copy_drv_output(&self, drv: &DrvFile) -> anyhow::Result<()> {
            let mut calls = self.calls.lock().await;
            calls.push(NixCliCall::CopyDrvOurput((*drv).clone()));

            Ok(())
        }
    }

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

    #[tokio::test]
    async fn test_process_stdin_processes_all_lines() {
        let (tx, mut rx) = mpsc::channel(100);

        let input: Vec<Result<String, Error>> =
            vec![Ok(String::from("one")), Ok(String::from("two"))];

        let result = process_stdin(input.into_iter(), tx).await;
        assert!(result.is_ok());

        assert_eq!(rx.recv().await, Some(Line::Info(String::from("one"))));
        assert_eq!(rx.recv().await, Some(Line::Info(String::from("two"))));
        assert_eq!(rx.recv().await, None);
    }

    #[tokio::test]
    async fn test_run_parses_stdin_and_copies_store_paths() {
        let input: Vec<Result<String, Error>> =
            vec![
                Ok(String::from("/nix/store/y0id07hk69wfhr14mpjq22fv2v27nsnk-zstd-1.5.2-dev")), 
                Ok(String::from("copying path '/nix/store/vnwdak3n1w2jjil119j65k8mw1z23p84-glibc-2.35-224' from 'https://cache.nixos.org'...")),
                Ok(String::from("building '/nix/store/kwd8mkkl1sv3n5z9jf8447gr9g299pmp-nix-cache-copy-0.1.0.drv'..."))
            ];

        let nix_cli = MockNixCli::new();
        let calls = Arc::clone(&nix_cli.calls);
        run(input.into_iter(), nix_cli).await;

        let calls = calls.lock().await;

        assert_eq!(
            *calls,
            vec![
                NixCliCall::CopyStorePath(StorePath::from(String::from(
                    "/nix/store/vnwdak3n1w2jjil119j65k8mw1z23p84-glibc-2.35-224"
                ))),
                NixCliCall::CopyDrvOurput(DrvFile::from(String::from(
                    "/nix/store/kwd8mkkl1sv3n5z9jf8447gr9g299pmp-nix-cache-copy-0.1.0.drv"
                ))),
            ]
        );
    }
}
