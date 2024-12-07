use std::{future::Future, pin::Pin};

use futures::{stream::FuturesUnordered, FutureExt};

#[derive(Debug)]
pub enum OperationOutcome {
    CreationSuccess,
    CreationFailure,
    DestructionSuccess,
    DestructionFailure,
    ScanSuccess,
    ScanFailure,
}

#[derive(Debug, Clone, Copy)]
pub enum OperationType {
    Creation,
    Destruction,
    Scan,
}

type Bag = FuturesUnordered<Pin<Box<dyn Future<Output = (i32, OperationOutcome)>>>>;

pub fn launch_threads_boxdyn(bag: &mut Bag) {
    for i in 0..100 {
        let handle = tokio::task::spawn_blocking(move || {
            println!("Thread {i} is running");
            std::thread::sleep(std::time::Duration::from_secs(1));
            i
        });
        let operation_type = OperationType::Scan;
        let future = async move {
            let Ok(result) = handle.await else {
                return (i, OperationOutcome::CreationFailure);
            };
            match operation_type {
                OperationType::Creation => {
                    if result < 5 {
                        (i, OperationOutcome::CreationSuccess)
                    } else {
                        (i, OperationOutcome::CreationFailure)
                    }
                }
                OperationType::Destruction => {
                    if result < 9 {
                        (i, OperationOutcome::DestructionSuccess)
                    } else {
                        (i, OperationOutcome::DestructionFailure)
                    }
                }
                OperationType::Scan => {
                    if result < 1 {
                        (i, OperationOutcome::ScanSuccess)
                    } else {
                        (i, OperationOutcome::ScanFailure)
                    }
                }
            }
        }
        .boxed();
        bag.push(future);
    }
}

#[pin_project::pin_project]
pub struct Operation {
    #[pin]
    join_handle: tokio::task::JoinHandle<i32>,
    sensor_id: i32,
    operation_type: OperationType,
}

impl Future for Operation {
    type Output = (i32, OperationOutcome);

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        let Ok(return_code) = futures::ready!(this.join_handle.poll(cx)) else {
            return std::task::Poll::Ready((*this.sensor_id, OperationOutcome::CreationFailure));
        };
        let event = match this.operation_type {
            OperationType::Creation => {
                if return_code < 5 {
                    OperationOutcome::CreationSuccess
                } else {
                    OperationOutcome::CreationFailure
                }
            }
            OperationType::Destruction => {
                if return_code < 9 {
                    OperationOutcome::DestructionSuccess
                } else {
                    OperationOutcome::DestructionFailure
                }
            }
            OperationType::Scan => {
                if return_code < 1 {
                    OperationOutcome::ScanSuccess
                } else {
                    OperationOutcome::ScanFailure
                }
            }
        };
        std::task::Poll::Ready((*this.sensor_id, event))
    }
}

pub fn launch_threads_noboxdyn(bag: &mut FuturesUnordered<Operation>) {
    for i in 0..100 {
        let join_handle = tokio::task::spawn_blocking(move || {
            println!("Thread {i} is running");
            std::thread::sleep(std::time::Duration::from_secs(1));
            i
        });
        let operation = Operation {
            sensor_id: i,
            join_handle,
            operation_type: OperationType::Scan,
        };
        bag.push(operation);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use futures::stream::{FuturesUnordered, StreamExt};

    #[tokio::test]
    async fn test_launch_threads() {
        let mut bag = FuturesUnordered::new();

        launch_threads_boxdyn(&mut bag);
        while let Some((sensor_id, event)) = bag.next().await {
            println!("{:?}", (sensor_id, event));
        }
    }

    #[tokio::test]
    async fn test_launch_threads_noboxdyn() {
        let mut bag = FuturesUnordered::new();

        launch_threads_noboxdyn(&mut bag);
        while let Some((sensor_id, event)) = bag.next().await {
            println!("{:?}", (sensor_id, event));
        }
    }
}
