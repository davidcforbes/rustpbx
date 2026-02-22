# GitHub Issue: Transcript addon cannot find recording file when sipflow is enabled

**Submit to:** https://github.com/restsend/rustpbx/issues/new

**Title:** Transcript addon cannot find recording file when sipflow is enabled

---

## Description

When sipflow is enabled in config, the Call Transcription addon reports "Recording file not found for this call" even though a recording WAV file exists on disk.

## Root Cause

In `src/proxy/proxy_call/reporter.rs` around line 165, the recording file collection is gated by `!has_sipflow_backend`:

```rust
if self.context.dialplan.recording.enabled && !has_sipflow_backend {
```

This means when sipflow is active, the WAV file path is never added to the CDR record, even though the media bridge's `Recorder` still writes the WAV file during the call (both the recorder and sipflow run in parallel in `media_bridge.rs`).

The transcript handler in `src/addons/transcript/handlers.rs` calls `select_recording_path()` which checks CDR recorder paths and `recording_url` from the database — both are empty when sipflow is active, so it returns `None` and the transcript request fails with "Recording file not found".

## Steps to Reproduce

1. Enable both `[sipflow]` and `[recording]` with `auto_start = true` in config
2. Enable the transcript addon (`addons = ["transcript"]`)
3. Make a call and wait for it to complete
4. Go to Call Records > select the call > click "Request Transcript"
5. Error: "Failed - Recording file not found for this call"
6. However, clicking "Play" on the same recording works (reconstructed from sipflow data)

## Expected Behavior

The transcript addon should be able to find the recording file. Either:
- The reporter should always collect the WAV file path when recording is enabled (regardless of sipflow)
- Or the transcript handler should fall back to exporting a WAV from sipflow data when no physical file is found

## Suggested Fix

Remove the `!has_sipflow_backend` condition in the reporter:

```rust
// Before (reporter.rs line 165):
if self.context.dialplan.recording.enabled && !has_sipflow_backend {

// After:
if self.context.dialplan.recording.enabled {
```

This allows the WAV file (which is already written by the recorder during the call) to be referenced in the CDR, making it available to the transcript handler.

## Environment

- RustPBX v0.3.18
- Config: sipflow enabled (local), recording enabled with auto_start
- Transcript addon with external CLI command
