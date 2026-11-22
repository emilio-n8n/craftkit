//! Background jobs for desktop applications.
//!
//! Jobs run on a dedicated thread and report progress through shared state that
//! the UI can poll every frame. Results are returned from
//! [`JobRunner::poll`], so the UI never blocks waiting for work to finish.
//!
//! This is deliberately smaller than a general-purpose thread pool: most
//! desktop applications have a handful of long operations (import, export,
//! render, analysis) and want one visible progress row per operation.

use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};

/// Identifier of a job started by [`JobRunner`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JobId(u64);

impl JobId {
    /// Numeric value, useful for logs.
    #[must_use]
    pub fn value(self) -> u64 {
        self.0
    }
}

impl fmt::Display for JobId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "job-{}", self.0)
    }
}

/// Lifecycle state of a job.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStatus {
    /// Still running.
    Running,
    /// Completed successfully.
    Finished,
    /// Returned an error or panicked.
    Failed,
    /// Cooperative cancellation was requested and observed.
    Cancelled,
}

/// Shared progress and status message for one job.
#[derive(Debug)]
pub struct JobProgress {
    fraction_bits: AtomicU32,
    message: Mutex<String>,
}

impl Default for JobProgress {
    fn default() -> Self {
        Self {
            fraction_bits: AtomicU32::new(0_f32.to_bits()),
            message: Mutex::new(String::new()),
        }
    }
}

impl JobProgress {
    /// Set progress in the `0.0..=1.0` range. Values outside are clamped.
    pub fn set_fraction(&self, fraction: f32) {
        let clamped = fraction.clamp(0.0, 1.0);
        self.fraction_bits
            .store(clamped.to_bits(), Ordering::Relaxed);
    }

    /// Current progress in the `0.0..=1.0` range.
    #[must_use]
    pub fn fraction(&self) -> f32 {
        f32::from_bits(self.fraction_bits.load(Ordering::Relaxed))
    }

    /// Set a short human-readable status message.
    pub fn set_message(&self, message: impl Into<String>) {
        let mut guard = self
            .message
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        *guard = message.into();
    }

    /// Current status message.
    #[must_use]
    pub fn message(&self) -> String {
        self.message
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }
}

/// The worker-side handle passed to a job closure.
#[derive(Debug, Clone)]
pub struct JobContext {
    progress: Arc<JobProgress>,
    cancel: Arc<AtomicBool>,
}

impl JobContext {
    /// Set progress in the `0.0..=1.0` range.
    pub fn set_progress(&self, fraction: f32) {
        self.progress.set_fraction(fraction);
    }

    /// Set progress and a status message together.
    pub fn set_progress_message(&self, fraction: f32, message: impl Into<String>) {
        self.progress.set_fraction(fraction);
        self.progress.set_message(message);
    }

    /// Set a status message without changing progress.
    pub fn set_message(&self, message: impl Into<String>) {
        self.progress.set_message(message);
    }

    /// Whether cancellation has been requested.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }

    /// Return an error when cancellation has been requested.
    ///
    /// # Errors
    ///
    /// Returns [`JobCancelled`] when the job should stop.
    pub fn check_cancelled(&self) -> Result<(), JobCancelled> {
        if self.is_cancelled() {
            Err(JobCancelled)
        } else {
            Ok(())
        }
    }
}

/// Error returned by [`JobContext::check_cancelled`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("job cancelled")]
pub struct JobCancelled;

/// Read-only view of a running job for the UI.
#[derive(Debug, Clone)]
pub struct JobInfo {
    /// Stable job identifier.
    pub id: JobId,
    /// Label chosen by the caller.
    pub label: String,
    progress: Arc<JobProgress>,
    cancel: Arc<AtomicBool>,
}

impl JobInfo {
    /// Progress in the `0.0..=1.0` range.
    #[must_use]
    pub fn fraction(&self) -> f32 {
        self.progress.fraction()
    }

    /// Current status message.
    #[must_use]
    pub fn message(&self) -> String {
        self.progress.message()
    }

    /// Whether cancellation was requested.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }

    /// Request cooperative cancellation.
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

/// Error returned when a job thread cannot be started.
#[derive(Debug, thiserror::Error)]
#[error("could not start background job: {0}")]
pub struct JobSpawnError(pub String);

/// Owns running jobs and delivers their results.
///
/// `T` is the application result type. An application can use an enum when it
/// has several kinds of jobs, or a simple type such as `PathBuf` when all jobs
/// produce the same result.
#[derive(Debug)]
pub struct JobRunner<T: Send + 'static> {
    sender: Sender<(JobId, Result<T, String>)>,
    receiver: Receiver<(JobId, Result<T, String>)>,
    jobs: Vec<JobInfo>,
    next_id: u64,
}

impl<T: Send + 'static> Default for JobRunner<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Send + 'static> JobRunner<T> {
    /// Create an empty runner.
    #[must_use]
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver,
            jobs: Vec::new(),
            next_id: 1,
        }
    }

    /// Start a job on its own thread.
    ///
    /// The closure receives a [`JobContext`] for progress and cancellation and
    /// returns either the result or an error message.
    ///
    /// # Errors
    ///
    /// Returns [`JobSpawnError`] when the operating system refuses to start a
    /// thread.
    pub fn spawn<F>(&mut self, label: impl Into<String>, work: F) -> Result<JobId, JobSpawnError>
    where
        F: FnOnce(JobContext) -> Result<T, String> + Send + 'static,
    {
        let id = JobId(self.next_id);
        self.next_id += 1;

        let progress = Arc::new(JobProgress::default());
        let cancel = Arc::new(AtomicBool::new(false));
        let context = JobContext {
            progress: Arc::clone(&progress),
            cancel: Arc::clone(&cancel),
        };
        let sender = self.sender.clone();

        let thread_name = format!("craftkit-{id}");
        std::thread::Builder::new()
            .name(thread_name)
            .spawn(move || {
                let result =
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| work(context)))
                        .unwrap_or_else(|_| Err("background job panicked".to_owned()));
                let _ = sender.send((id, result));
            })
            .map_err(|error| JobSpawnError(error.to_string()))?;

        self.jobs.push(JobInfo {
            id,
            label: label.into(),
            progress,
            cancel,
        });
        Ok(id)
    }

    /// Collect completed jobs without blocking.
    ///
    /// The corresponding entries disappear from [`JobRunner::jobs`].
    pub fn poll(&mut self) -> Vec<(JobId, Result<T, String>)> {
        let mut completed = Vec::new();
        while let Ok((id, result)) = self.receiver.try_recv() {
            self.jobs.retain(|job| job.id != id);
            completed.push((id, result));
        }
        completed
    }

    /// Currently running jobs.
    #[must_use]
    pub fn jobs(&self) -> &[JobInfo] {
        &self.jobs
    }

    /// Request cancellation of one job.
    pub fn cancel(&mut self, id: JobId) {
        if let Some(job) = self.jobs.iter().find(|job| job.id == id) {
            job.cancel();
        }
    }

    /// Request cancellation of every running job.
    pub fn cancel_all(&mut self) {
        for job in &self.jobs {
            job.cancel();
        }
    }
}

impl<T: Send + 'static> Drop for JobRunner<T> {
    fn drop(&mut self) {
        self.cancel_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn wait_for_one(runner: &mut JobRunner<u32>, timeout: Duration) -> Result<u32, String> {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some((_, result)) = runner.poll().into_iter().next() {
                return result;
            }
            assert!(Instant::now() < deadline, "job did not finish in time");
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn returns_a_result_and_removes_the_job() {
        let mut runner = JobRunner::<u32>::new();
        runner
            .spawn("double", |context| {
                context.set_progress(0.5);
                context.set_message("half");
                Ok(21 * 2)
            })
            .expect("spawn");
        assert_eq!(runner.jobs().len(), 1);
        assert_eq!(wait_for_one(&mut runner, Duration::from_secs(2)), Ok(42));
        assert!(runner.jobs().is_empty());
    }

    #[test]
    fn reports_errors_and_panics() {
        let mut runner = JobRunner::<u32>::new();
        runner
            .spawn("error", |_| Err("nope".to_owned()))
            .expect("spawn");
        assert_eq!(
            wait_for_one(&mut runner, Duration::from_secs(2)),
            Err("nope".to_owned())
        );

        runner.spawn("panic", |_| panic!("boom")).expect("spawn");
        assert!(
            wait_for_one(&mut runner, Duration::from_secs(2))
                .expect_err("panic becomes error")
                .contains("panicked")
        );
    }

    #[test]
    fn cooperative_cancellation_is_visible_to_the_worker() {
        let mut runner = JobRunner::<u32>::new();
        let id = runner
            .spawn("cancelled", |context| {
                std::thread::sleep(Duration::from_millis(50));
                context
                    .check_cancelled()
                    .map_err(|error| error.to_string())?;
                Ok(1)
            })
            .expect("spawn");
        runner.cancel(id);
        assert_eq!(
            wait_for_one(&mut runner, Duration::from_secs(2)),
            Err("job cancelled".to_owned())
        );
    }

    #[test]
    fn progress_is_clamped() {
        let progress = JobProgress::default();
        progress.set_fraction(-1.0);
        assert_eq!(progress.fraction(), 0.0);
        progress.set_fraction(3.0);
        assert_eq!(progress.fraction(), 1.0);
        progress.set_message("working");
        assert_eq!(progress.message(), "working");
    }
}
