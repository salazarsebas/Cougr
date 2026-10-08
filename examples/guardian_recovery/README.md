# Guardian Recovery Example

This example demonstrates how an account can be recovered using social guardians, even while a match is ongoing.
It simulates a two-player match where one player loses a device, but through social recovery, they can regain access and continue the match.

## Trust Assumptions & Security

- **Guardians see recovery requests:** Guardians are trusted to inspect a recovery request and decide whether to approve it based on off-chain authentication or communication.
- **Guardians cannot play:** The guardians are only allowed to vote on account recovery. They do not gain session access to the match and cannot perform gameplay actions on behalf of the player.
- **Account control:** Only a successful recovery meeting the threshold transfers the account to a new owner address. A failed recovery does not change the match state or give anyone access.

## Flow
1. Players register sessions for a match.
2. The match begins, players can move (in-session play).
3. A player's device is revoked (simulating a lost device).
4. The lost device can no longer act.
5. The player initiates a recovery request.
6. Guardians approve the recovery.
7. The recovery is executed, restoring the player to a new owner address.
8. The restored player can resume playing the same match.
