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

    /// Validates `items.len()` and executes `executor` for each element in order.
    ///
    /// Note: The integrating contract is responsible for authenticating any `caller` at the entrypoint.
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
    use alloc::vec;

    use super::*;

    #[test]
    fn batch_executor_validate_len_enforces_boundaries() {
        let batch = BatchExecutor::new(3);
        assert_eq!(batch.max_size(), 3);

        assert_eq!(batch.validate_len(0), Err(StandardsError::BatchEmpty));
        assert!(batch.validate_len(1).is_ok());
        assert!(batch.validate_len(3).is_ok());
        assert_eq!(batch.validate_len(4), Err(StandardsError::BatchTooLarge));
    }

    #[test]
    fn batch_executor_execute_covers_success_limits_and_short_circuit() {
        let batch = BatchExecutor::new(2);

        assert_eq!(
            batch.execute::<u32, u32>(&[], |_| Ok(1)),
            Err(StandardsError::BatchEmpty)
        );
        assert_eq!(
            batch.execute(&[1u32, 2, 3], |item| Ok(*item)),
            Err(StandardsError::BatchTooLarge)
        );

        let results = batch.execute(&[10u32, 20], |item| Ok(item + 1)).unwrap();
        assert_eq!(results, vec![11, 21]);

        let err = batch.execute(&[1u32, 2], |item| {
            if *item == 2 {
                Err(StandardsError::Unauthorized)
            } else {
                Ok(*item)
            }
        });
        assert_eq!(err, Err(StandardsError::Unauthorized));
    }
}
