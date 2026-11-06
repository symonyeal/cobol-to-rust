# Acceptance contract

The acceptance unit is program + source revision + execution profile + declared input domain + corpus. A result applies only to this unit.

Each run starts with the same initial bytes at the same absolute logical job path. Programs run directly with argument arrays, cleared and explicitly populated environments, binary pipes and a timeout. The workbench captures stdout, stderr, return status, and the complete regular-file tree before and after a run. Paths are sorted only to enumerate comparisons; records and bytes are never sorted or transformed.

Changed or deleted inputs are compared along with output files. Missing and empty files differ. A first differing offset and bounded hexadecimal contexts aid diagnosis without altering the acceptance bytes. Hashes identify the source and profile; equality is decided using actual bytes.

Reference baselines contain at least three runs per scenario and are accepted only when all observations are identical. Variable references retain an unstable bundle and cannot be verified. Baselines use exclusive creation and a new label is required to regenerate them. Verification reads the baseline and writes a separate result. It rechecks source/profile hashes, manifest bytes and job path, and cannot generate expected observations.

The implementation observes file states at job boundaries. It does not trace transient writes removed before a boundary or filesystem metadata. An additional step requiring intermediate observations must be admitted only after those observations are exposed at a capture boundary. The admitted cases must not spawn descendants that outlive the main process or write beyond the declared job directory; the timeout currently terminates only the direct child. These restrictions are admission requirements, not security sandbox guarantees.

The maintained Rust launcher reserves exit 2 plus its versioned `infrastructure_error` JSON envelope for its own failures. Verification retains such a failed capture separately and returns an infrastructure error before comparing program output. The three admitted ports use program statuses 0 and 1; this reserved launcher envelope is outside their declared output domain.

Do not trim, normalize newlines, decode/re-encode acceptance bytes, mask dates, sort records or accept numeric tolerance. Control time through the runtime operation the source actually uses, then prove the control with executable evidence. A corrected original defect is a separate behavioral change.

To add a case, pin the exact source and licence, identify external dependencies, document storage and control semantics, declare every file input and scenario, establish stable reference captures, implement and review the Rust port, then run both debug and release verification. The required-case registry cannot be empty. A blocked or missing required case fails `verify --all`.
