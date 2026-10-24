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

    /// Validates the batch length and runs `executor` for each item in order.
    ///
    /// Note: This helper does not perform caller authentication; the
    /// integrating contract must authenticate the caller before executing
    /// batched operations.
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
    use alloc::vec;

    #[test]
    fn validate_len_enforces_empty_and_upper_bounds() {
        let batch = BatchExecutor::new(3);
        assert_eq!(batch.max_size(), 3);
        assert_eq!(batch.validate_len(0), Err(StandardsError::BatchEmpty));
        assert_eq!(batch.validate_len(1), Ok(()));
        assert_eq!(batch.validate_len(3), Ok(()));
        assert_eq!(batch.validate_len(4), Err(StandardsError::BatchTooLarge));
    }

    #[test]
    fn execute_validates_bounds_and_propagates_callback_errors() {
        let batch = BatchExecutor::new(2);

        assert_eq!(
            batch.execute::<u32, u32>(&[], |item| Ok(*item)),
            Err(StandardsError::BatchEmpty)
        );
        assert_eq!(
            batch.execute(&[1u32, 2, 3], |item| Ok(*item)),
            Err(StandardsError::BatchTooLarge)
        );
        assert_eq!(
            batch.execute(&[10u32, 20], |item| Ok(item + 1)),
            Ok(vec![11, 21])
        );
        assert_eq!(
            batch.execute(&[1u32, 2], |item| {
                if *item == 2 {
                    Err(StandardsError::Unauthorized)
                } else {
                    Ok(*item)
                }
            }),
            Err(StandardsError::Unauthorized)
        );
    }
}
