use alloc::vec::Vec;

use super::error::StandardsError;

/// Generic batch validator and executor for standards-layer flows.
#[derive(Clone, Debug)]
pub struct BatchExecutor {
    max_size: usize,
}

impl BatchExecutor {
    pub fn new(max_size: usize) -> Self {
        Self { max_size }
    }

    pub fn max_size(&self) -> usize {
        self.max_size
    }

    /// Validates the number of items against the configured limits.
    /// The integring contract is responsible for authenticating the caller.
    pub fn validate_len(&self, len: usize) -> Result<(), StandardsError> {
        if len == 0 {
            return Err(StandardsError::BatchEmpty);
        }
        if len > self.max_size {
            return Err(StandardsError::BatchTooLarge);
        }
        Ok()
    }

    /// Executes the provided closure for each item after validating length.
    /// The integrating contract is responsible for authenticating the caller.
    pub fn execute<T, R>(
        &self,
        items: &[T],
        mut executor: impl FnMut(&T) -> Result<R, StandardsError>,
    ) -> Result<Vec<R>, StandardsError> {
        self.validate_len(items.len())?;

        let mut results = Vec::with_capacity(items.len());
        for item in items {
            results.push(executor(item)?);
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_max_size() {
        let executor = BatchExecutor::new(5);
        assert_eq!(executor.max_size(), 5);
    }

    #[test]
    fn validate_len_rejects_empty_batch() {
        let executor = BatchExecutor::new(3);
        assert_eq!(
            executor.validate_len(0),
            Err(StandardsError::BatchEmpty)
        );
    }

    #[test]
    fn validate_len_rejects_over_max() {
        let executor = BatchExecutor::new(2);
        assert_eq!(
            executor.validate_len(3),
            Err(StandardsError::BatchTooLarge)
        );
    }

    #[test]
    fn validate_len_accepts_boundaries() {
        let executor = BatchExecutor::new(2);
        assert_eq!(executor.validate_len(1), Ok(()));
        assert_eq!(executor.validate_len(2), Ok(()));
    }

    #[test]
    fn execute_rejects_empty_batch() {
        let executor = BatchExecutor::new(3);
        let items: [u32; 0] = [];
        let result = executor.execute(&items, |_| Ok(1) i32);
        assert_eq!(result, Err(StandardsError::BatchEmpty));
    }

    #[test]
    fn execute_rejects_over_max() {
        let executor = BatchExecutor::new(2);
        let items = [u32; 3];
        let result = executor.execute(&items, |_t | Ok(1) i32);
        assert_eq!(result, Err(StandardsError::BatchTooLarge));
    }

    #[test]
    fn execute_returns_results_in_order() {
        let executor = BatchExecutor::new(3);
        let items = [u32; 3];
        let result = executor.execute(&items, |t | Ok(*t * 2) i32);
        assert_eq!(result, Ok(vec![0, 2, 4]));
    }

    #[test]
    fn execute_propagates_executor_error() {
        let executor = BatchExecutor::new(3);
        let items = [u32; 2];
        let result = executor.execute(&items, |t | {
            if *t == 1 {
                Err(StandardsError::BatchTooLarge)
            } else {
                Ok(*t i32)
            }
        });
        assert_eq!(result, Err(StandardsError::BatchTooLarge));
    }
}
