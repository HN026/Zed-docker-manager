//! Mock Docker executor for unit testing command handlers without
//! requiring a Docker installation. Records all invocations and
//! returns a preconfigured response.

use std::cell::RefCell;
use crate::docker::DockerExecutor;

pub struct MockDockerExecutor {
    captured: RefCell<Vec<Vec<String>>>,
    response: Result<String, String>,
}

impl MockDockerExecutor {
    // Creates a mock that always returns the given stdout.
    pub fn with_success(output: &str) -> Self {
        Self {
            captured: RefCell::new(Vec::new()),
            response: Ok(output.to_string()),
        }
    }

    // Creates a mock that always returns the given error.
    pub fn with_error(error: &str) -> Self {
        Self {
            captured: RefCell::new(Vec::new()),
            response: Err(error.to_string()),
        }
    }

    // Returns all argument lists that were passed to execute.
    pub fn captured_args(&self) -> Vec<Vec<String>> {
        self.captured.borrow().clone()
    }
}

impl DockerExecutor for MockDockerExecutor {
    fn execute(&self, args: &[&str]) -> Result<String, String> {
        self.captured
            .borrow_mut()
            .push(args.iter().map(|s| s.to_string()).collect());
        self.response.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_returns_configured_success() {
        let mock = MockDockerExecutor::with_success("hello");
        assert_eq!(mock.execute(&["ps"]).unwrap(), "hello");
    }

    #[test]
    fn mock_returns_configured_error() {
        let mock = MockDockerExecutor::with_error("fail");
        assert_eq!(mock.execute(&["stop"]).unwrap_err(), "fail");
    }

    #[test]
    fn mock_captures_all_calls() {
        let mock = MockDockerExecutor::with_success("ok");
        let _ = mock.execute(&["ps", "-a"]);
        let _ = mock.execute(&["stop", "abc"]);
        let captured = mock.captured_args();
        assert_eq!(captured.len(), 2);
        assert_eq!(captured[0], vec!["ps", "-a"]);
        assert_eq!(captured[1], vec!["stop", "abc"]);
    }
}
