/**
 * cougr-sdk-core public API
 *
 * Re-exports the full error taxonomy and the simulate-and-submit client.
 */

export {
  TAXONOMY_VERSION,
  SdkCoreError,
  isSdkCoreError,
  isSimulationError,
  isSubmissionError,
  isOnchainRejected,
  isTimeoutError,
  type SdkError,
  type SdkErrorBase,
  type SdkErrorCode,
  type SdkErrorKind,
  type SimulationError,
  type SubmissionError,
  type OnchainRejectedError,
  type TimeoutError,
} from './errors.ts';

export {
  SorobanRpcClient,
  simulateAndSubmit,
  augmentWithSimulation,
  extractContractReason,
  type FetchLike,
  type SorobanRpcClientOptions,
  type SimulateTransactionResult,
  type SendTransactionResult,
  type GetTransactionResult,
  type SubmitOptions,
  type SubmitResult,
} from './submit.ts';
