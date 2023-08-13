use crate::{thread_pool::ThreadPool, Result};
use std::sync::Arc;
use rayon;

#[derive(Clone)]
pub struct RayonThreadPool {
    raypool: Arc<rayon::ThreadPool>,
}

impl ThreadPool for RayonThreadPool {
    fn new(thread: usize) -> Result<Self>
    where
        Self: Sized,
    {
        let raypool = Arc::new(rayon::ThreadPoolBuilder::new()
            .num_threads(thread.try_into().unwrap())
            .build()
            .unwrap());
        Ok(RayonThreadPool { raypool})
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.raypool.spawn(job);
    }
}
