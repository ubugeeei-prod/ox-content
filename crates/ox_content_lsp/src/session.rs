use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use rustc_hash::FxHashMap;
use tower_lsp::lsp_types::{Diagnostic, Url};

/// One in-flight diagnostic computation for a document version.
#[derive(Clone, Debug)]
pub struct DiagnosticJob {
    pub version: i32,
    cancelled: Arc<AtomicBool>,
}

impl DiagnosticJob {
    #[must_use]
    pub fn new(version: i32) -> Self {
        Self { version, cancelled: Arc::new(AtomicBool::new(false)) }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

/// Cancels superseded jobs and suppresses stale publishes.
#[derive(Default)]
pub struct PublishGate {
    latest: FxHashMap<Url, i32>,
    jobs: FxHashMap<Url, DiagnosticJob>,
}

impl PublishGate {
    #[must_use]
    pub fn begin(&mut self, uri: Url, version: i32) -> DiagnosticJob {
        if let Some(previous) = self.jobs.get(&uri) {
            previous.cancel();
        }
        let job = DiagnosticJob::new(version);
        self.latest.insert(uri.clone(), version);
        self.jobs.insert(uri, job.clone());
        job
    }

    #[must_use]
    pub fn should_publish(&self, uri: &Url, job: &DiagnosticJob) -> bool {
        !job.is_cancelled() && self.latest.get(uri) == Some(&job.version)
    }

    pub fn clear(&mut self, uri: &Url) {
        if let Some(job) = self.jobs.remove(uri) {
            job.cancel();
        }
        self.latest.remove(uri);
    }
}

/// Cached diagnostic slices so a body-only edit skips frontmatter work.
#[derive(Clone, Debug, Default)]
pub struct DiagnosticCache {
    pub frontmatter_src: String,
    pub body_src: String,
    pub frontmatter: Vec<Diagnostic>,
    pub parse: Vec<Diagnostic>,
    pub mdc: Vec<Diagnostic>,
    pub links: Vec<Diagnostic>,
    pub spacing: Vec<Diagnostic>,
}

impl DiagnosticCache {
    #[must_use]
    pub fn reuse(&self, frontmatter_src: &str, body_src: &str) -> (bool, bool) {
        let frontmatter = self.frontmatter_src == frontmatter_src;
        (frontmatter, frontmatter && self.body_src == body_src)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uri(path: &str) -> Url {
        Url::parse(&format!("file://{path}")).expect("valid file url")
    }

    #[test]
    fn cancelling_an_in_flight_job_lets_a_later_version_win() {
        let mut gate = PublishGate::default();
        let uri = uri("/tmp/doc.md");
        let first = gate.begin(uri.clone(), 1);
        let second = gate.begin(uri.clone(), 2);

        assert!(first.is_cancelled(), "the superseded job must be cancelled");
        assert!(!second.is_cancelled());
        assert!(!gate.should_publish(&uri, &first), "stale version 1 must not publish");
        assert!(gate.should_publish(&uri, &second), "version 2 must win");
    }

    #[test]
    fn cancelled_job_does_not_publish_even_if_it_is_latest() {
        let mut gate = PublishGate::default();
        let uri = uri("/tmp/doc.md");
        let job = gate.begin(uri.clone(), 4);
        job.cancel();
        assert!(!gate.should_publish(&uri, &job));
    }

    #[test]
    fn body_reuse_requires_unchanged_frontmatter() {
        let cache = DiagnosticCache {
            frontmatter_src: "---\ntitle: Doc\n---\n".into(),
            body_src: "See [a](./a.md).\n".into(),
            ..DiagnosticCache::default()
        };
        assert_eq!(cache.reuse("---\ntitle: Doc\n---\n", "See [a](./a.md).\n"), (true, true));
        assert_eq!(cache.reuse("---\ntitle: Doc\n---\n", "See [b](./b.md).\n"), (true, false));
        assert_eq!(cache.reuse("---\ntitle: Other\n---\n", "See [a](./a.md).\n"), (false, false));
    }
}
