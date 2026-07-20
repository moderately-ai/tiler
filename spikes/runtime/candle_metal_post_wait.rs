//! Source-level transition test for Candle Metal's completion check.
//!
//! This models the exact control-flow shape inspected at Candle commit
//! 31f35b147389700ed2a178ee66a91c3cc25cc80d in
//! candle-metal-kernels/src/metal/commands.rs:317-337. It does not claim to
//! inject a real GPU fault. Its purpose is to execute the permitted status
//! transition that the concrete function currently fails to observe.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Status {
    NotEnqueued,
    Enqueued,
    Committed,
    Scheduled,
    Completed,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum CompletionError {
    CommandBuffer(String),
    UnexpectedTerminalStatus(Status),
}

trait CommandBuffer {
    fn status(&self) -> Status;
    fn commit(&mut self);
    fn wait_until_completed(&mut self);
    fn error(&self) -> Option<&str>;
}

#[derive(Debug)]
struct TransitionBuffer {
    status: Status,
    status_after_wait: Status,
    error: Option<&'static str>,
    commits: usize,
    waits: usize,
    status_reads: usize,
}

impl TransitionBuffer {
    fn new(status: Status, status_after_wait: Status) -> Self {
        Self {
            status,
            status_after_wait,
            error: Some("intentional simulated GPU runtime fault"),
            commits: 0,
            waits: 0,
            status_reads: 0,
        }
    }
}

impl CommandBuffer for TransitionBuffer {
    fn status(&self) -> Status {
        // Interior mutability would obscure the test. The read count is instead
        // recorded by the two wrappers below immediately before this call.
        self.status
    }

    fn commit(&mut self) {
        self.commits += 1;
        self.status = Status::Committed;
    }

    fn wait_until_completed(&mut self) {
        self.waits += 1;
        self.status = self.status_after_wait;
    }

    fn error(&self) -> Option<&str> {
        self.error
    }
}

fn read_status(cb: &mut TransitionBuffer) -> Status {
    cb.status_reads += 1;
    cb.status()
}

/// The inspected Candle control flow: status is read before waiting only.
fn inspected_ensure_completed(cb: &mut TransitionBuffer) -> Result<(), CompletionError> {
    match read_status(cb) {
        Status::NotEnqueued | Status::Enqueued => {
            cb.commit();
            cb.wait_until_completed();
        }
        Status::Committed | Status::Scheduled => {
            cb.wait_until_completed();
        }
        Status::Completed => {}
        Status::Error => {
            return Err(CompletionError::CommandBuffer(
                cb.error().unwrap_or("unknown error").to_owned(),
            ));
        }
    }

    Ok(())
}

/// Required shape: every successful return is justified by a final Completed
/// observation made after any commit/wait transition.
fn checked_ensure_completed(cb: &mut TransitionBuffer) -> Result<(), CompletionError> {
    match read_status(cb) {
        Status::NotEnqueued | Status::Enqueued => {
            cb.commit();
            cb.wait_until_completed();
        }
        Status::Committed | Status::Scheduled => cb.wait_until_completed(),
        Status::Completed | Status::Error => {}
    }

    match read_status(cb) {
        Status::Completed => Ok(()),
        Status::Error => Err(CompletionError::CommandBuffer(
            cb.error().unwrap_or("unknown error").to_owned(),
        )),
        status => Err(CompletionError::UnexpectedTerminalStatus(status)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspected_committed_path_returns_success_after_error_transition() {
        let mut cb = TransitionBuffer::new(Status::Committed, Status::Error);

        assert_eq!(inspected_ensure_completed(&mut cb), Ok(()));
        assert_eq!(cb.status, Status::Error);
        assert_eq!((cb.waits, cb.status_reads), (1, 1));
    }

    #[test]
    fn inspected_scheduled_path_returns_success_after_error_transition() {
        let mut cb = TransitionBuffer::new(Status::Scheduled, Status::Error);

        assert_eq!(inspected_ensure_completed(&mut cb), Ok(()));
        assert_eq!(cb.status, Status::Error);
        assert_eq!((cb.waits, cb.status_reads), (1, 1));
    }

    #[test]
    fn inspected_commit_path_has_the_same_gap() {
        let mut cb = TransitionBuffer::new(Status::NotEnqueued, Status::Error);

        assert_eq!(inspected_ensure_completed(&mut cb), Ok(()));
        assert_eq!(cb.status, Status::Error);
        assert_eq!((cb.commits, cb.waits, cb.status_reads), (1, 1, 1));
    }

    #[test]
    fn inspected_preexisting_error_is_detected() {
        let mut cb = TransitionBuffer::new(Status::Error, Status::Error);

        assert_eq!(
            inspected_ensure_completed(&mut cb),
            Err(CompletionError::CommandBuffer(
                "intentional simulated GPU runtime fault".to_owned()
            ))
        );
        assert_eq!((cb.waits, cb.status_reads), (0, 1));
    }

    #[test]
    fn required_check_rejects_error_after_wait_and_preserves_detail() {
        let mut cb = TransitionBuffer::new(Status::Committed, Status::Error);

        assert_eq!(
            checked_ensure_completed(&mut cb),
            Err(CompletionError::CommandBuffer(
                "intentional simulated GPU runtime fault".to_owned()
            ))
        );
        assert_eq!((cb.waits, cb.status_reads), (1, 2));
    }

    #[test]
    fn required_check_accepts_observed_completed_terminal_state() {
        let mut cb = TransitionBuffer::new(Status::Scheduled, Status::Completed);

        assert_eq!(checked_ensure_completed(&mut cb), Ok(()));
        assert_eq!((cb.waits, cb.status_reads), (1, 2));
    }

    #[test]
    fn required_check_fails_closed_on_impossible_nonterminal_return() {
        let mut cb = TransitionBuffer::new(Status::Committed, Status::Scheduled);

        assert_eq!(
            checked_ensure_completed(&mut cb),
            Err(CompletionError::UnexpectedTerminalStatus(Status::Scheduled))
        );
    }

    #[test]
    fn completed_fast_path_is_still_checked_once_at_return() {
        let mut cb = TransitionBuffer::new(Status::Completed, Status::Completed);

        assert_eq!(checked_ensure_completed(&mut cb), Ok(()));
        assert_eq!((cb.waits, cb.status_reads), (0, 2));
    }

    #[test]
    fn enqueued_commit_path_is_covered() {
        let mut cb = TransitionBuffer::new(Status::Enqueued, Status::Completed);

        assert_eq!(checked_ensure_completed(&mut cb), Ok(()));
        assert_eq!((cb.commits, cb.waits, cb.status_reads), (1, 1, 2));
    }
}
