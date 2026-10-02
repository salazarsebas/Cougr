export type MatchStatus = 0 | 1 | 2;

export interface MatchState {
  player: string;
  last_tick: number;
  /**
   * Soroban u64. scValToNative decodes it as a bigint, which can exceed
   * Number.MAX_SAFE_INTEGER, so allow the exact bigint and decimal-string
   * forms alongside a safe number rather than lossy float arithmetic.
   */
  last_hash: number | bigint | string;
  last_score: number;
  status: MatchStatus;
}

export interface ActionResult {
  success: boolean;
  match_state: MatchState;
  message: string;
}

export const STATUS = { RUNNING: 0, FINALISED: 1, DISPUTED: 2 } as const;

export function statusLabel(status: MatchStatus): string {
  switch (status) {
    case STATUS.FINALISED: return "Finalised";
    case STATUS.DISPUTED: return "Disputed";
    default: return "Running";
  }
}

export function actionRejection(message: string): string {
  switch (message) {
    case "notrun": return "The match is not running.";
    case "oldtick": return "That tick is not newer than the last committed checkpoint.";
    case "badtick": return "The disputed tick does not match the committed checkpoint.";
    case "toolate": return "The dispute window has already closed.";
    case "nohash": return "The claimed hash matches the committed checkpoint, so there is nothing to dispute.";
    case "window": return "The dispute window is still open, so the match cannot be finalised yet.";
    case "notplay": return "The connected wallet is not authorised for this match.";
    default: return message || "The contract rejected the action.";
  }
}

export function isMatchState(value: unknown): value is MatchState {
  if (!value || typeof value !== "object") return false;
  const state = value as Partial<MatchState>;
  return typeof state.player === "string" &&
    typeof state.last_tick === "number" && Number.isSafeInteger(state.last_tick) && state.last_tick >= 0 &&
    parseStateHash(state.last_hash) !== null &&
    typeof state.last_score === "number" && Number.isSafeInteger(state.last_score) && state.last_score >= 0 &&
    [0, 1, 2].includes(state.status ?? -1);
}

export function isActionResult(value: unknown): value is ActionResult {
  if (!value || typeof value !== "object") return false;
  const result = value as Partial<ActionResult>;
  return typeof result.success === "boolean" && typeof result.message === "string" &&
    isMatchState(result.match_state);
}

/**
 * Parsed u64 state hash. Soroban u64 values can exceed Number.MAX_SAFE_INTEGER,
 * so the RPC value is kept as exact high and low 32-bit parts for display and
 * comparison instead of lossy float arithmetic.
 */
export interface StateHash {
  high: number;
  low: number;
  label: string;
}

export function parseStateHash(raw: unknown): StateHash | null {
  if (typeof raw === "bigint") {
    if (raw < 0n || raw > 0xffffffffffffffffn) return null;
    return fromParts(Number(raw >> 32n), Number(raw & 0xffffffffn));
  }
  if (typeof raw === "number") {
    if (!Number.isSafeInteger(raw) || raw < 0) return null;
    return fromParts(Math.floor(raw / 0x100000000), raw % 0x100000000);
  }
  if (typeof raw === "string" && /^[0-9]+$/.test(raw)) {
    // Schoolbook base conversion in two 32-bit limbs; overflows past u64 are rejected.
    let high = 0;
    let low = 0;
    for (const digit of raw) {
      const carry = low * 10 + Number(digit);
      low = carry % 0x100000000;
      high = high * 10 + Math.floor(carry / 0x100000000);
      if (!Number.isSafeInteger(high) || high > 0xffffffff) return null;
    }
    return fromParts(high, low);
  }
  return null;
}

function fromParts(high: number, low: number): StateHash {
  return { high, low, label: hashLabel(high, low) };
}

export function hashLabel(high: number, low: number): string {
  return `0x${high.toString(16).padStart(8, "0")}${low.toString(16).padStart(8, "0")}`;
}

export function isValidStateHashInput(raw: string): boolean {
  return /^[0-9]{1,20}$/.test(raw) && parseStateHash(raw) !== null;
}

/**
 * Derive a u64 state hash from an off-chain state summary with FNV-1a.
 *
 * The template contract does not mandate a hash scheme; it only compares
 * committed and claimed hashes. This is the canonical derivation this reference
 * client uses so a challenger can deliberately submit a differing hash when
 * disputing.
 */
export function hashStateSummary(summary: string): string {
  let hash = 0x811c9dc5;
  for (let index = 0; index < summary.length; index++) {
    hash ^= summary.charCodeAt(index);
    hash = Math.imul(hash, 0x01000193) >>> 0;
  }
  return String(hash >>> 0);
}
