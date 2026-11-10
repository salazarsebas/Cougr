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

    pub fn validate_len(&self, len: usize) -> Result<(), StandardsError> {
        if len == 0 {
            return Err(StandardsError::BatchEmpty);
        }
        if len > self.max_size {
            return Err(StandardsError::BatchTooLarge);
        }
        Ok(())
    }

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
