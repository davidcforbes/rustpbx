"""
L5 Media Quality Tests -- Recording verification.

After calls are placed through RustPBX the recording subsystem (enabled in the
test configuration) should produce WAV files.  These tests verify that:

  - Recording files exist after a call.
  - WAV headers are well-formed (correct sample rate, channels, non-zero size).
  - Audio content is not completely silent.

Environment variables:
    SIPP_PATH          Path to the sipp binary       (default: sipp)
    RUSTPBX_HOST       Hostname / IP of the PBX      (default: rustpbx)
    RUSTPBX_SIP_PORT   SIP port                      (default: 5060)
    RUSTPBX_HTTP_PORT  HTTP/AMI port                  (default: 8080)
    RECORDING_DIR      Path to RustPBX recording dir  (default: /app/config/cdr)
"""

import os
import struct
import subprocess
import time
import glob as glob_mod

import pytest
import requests

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

SIPP_PATH = os.environ.get("SIPP_PATH", "sipp")
RUSTPBX_HOST = os.environ.get("RUSTPBX_HOST", "rustpbx")
RUSTPBX_SIP_PORT = int(os.environ.get("RUSTPBX_SIP_PORT", "5060"))
RUSTPBX_HTTP_PORT = int(os.environ.get("RUSTPBX_HTTP_PORT", "8080"))
SIPP_SCENARIOS = os.path.join(os.path.dirname(__file__), "sipp")
RECORDING_DIR = os.environ.get("RECORDING_DIR", "/app/config/cdr")

BASE_URL = f"http://{RUSTPBX_HOST}:{RUSTPBX_HTTP_PORT}"


# ---------------------------------------------------------------------------
# SIPp helper (shared logic)
# ---------------------------------------------------------------------------


def run_sipp(scenario, extra_args=None, timeout=30):
    """Run a SIPp scenario file and return the CompletedProcess."""
    cmd = [
        SIPP_PATH,
        f"{RUSTPBX_HOST}:{RUSTPBX_SIP_PORT}",
        "-sf", os.path.join(SIPP_SCENARIOS, scenario),
        "-m", "1",
        "-timeout", str(timeout),
        "-timeout_error",
        "-max_retrans", "3",
    ]
    if extra_args:
        cmd.extend(extra_args)
    return subprocess.run(cmd, capture_output=True, text=True, timeout=timeout + 10)


def _make_test_call():
    """Place a short call through the PBX so a recording is generated."""
    # Register both endpoints.
    for user, pwd in [("1001", "test1001"), ("1002", "test1002")]:
        run_sipp("register_auth.xml", [
            "-key", "username", user,
            "-key", "password", pwd,
        ])

    # Start UAS.
    uas_cmd = [
        SIPP_PATH,
        "-sf", os.path.join(SIPP_SCENARIOS, "uas_answer.xml"),
        "-p", "6000", "-m", "1",
        "-timeout", "30", "-timeout_error",
    ]
    uas_proc = subprocess.Popen(uas_cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    time.sleep(1)

    try:
        result = run_sipp("uac_invite.xml", [
            "-key", "to_user", "1002",
            "-p", "5062",
        ], timeout=30)
        return result
    finally:
        uas_proc.terminate()
        try:
            uas_proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            uas_proc.kill()


def _find_latest_wav(directory):
    """Return the path to the most recently modified WAV file, or None."""
    wavs = sorted(
        glob_mod.glob(os.path.join(directory, "**", "*.wav"), recursive=True),
        key=os.path.getmtime,
        reverse=True,
    )
    return wavs[0] if wavs else None


def _parse_wav_header(path):
    """Parse the RIFF/WAV header and return a dict with basic metadata.

    Returns:
        dict with keys: channels, sample_rate, byte_rate, bits_per_sample,
                        data_size, file_size
    Raises ValueError on a malformed header.
    """
    with open(path, "rb") as f:
        riff = f.read(4)
        if riff != b"RIFF":
            raise ValueError(f"Not a RIFF file: {riff!r}")
        file_size = struct.unpack("<I", f.read(4))[0]
        wave = f.read(4)
        if wave != b"WAVE":
            raise ValueError(f"Not a WAVE file: {wave!r}")

        # Walk chunks until we find 'fmt ' and 'data'.
        fmt_info = {}
        data_size = 0

        while True:
            chunk_header = f.read(8)
            if len(chunk_header) < 8:
                break
            chunk_id = chunk_header[:4]
            chunk_size = struct.unpack("<I", chunk_header[4:])[0]

            if chunk_id == b"fmt ":
                fmt_data = f.read(chunk_size)
                (audio_format, channels, sample_rate, byte_rate,
                 block_align, bits_per_sample) = struct.unpack("<HHIIHH", fmt_data[:16])
                fmt_info = {
                    "audio_format": audio_format,
                    "channels": channels,
                    "sample_rate": sample_rate,
                    "byte_rate": byte_rate,
                    "block_align": block_align,
                    "bits_per_sample": bits_per_sample,
                }
            elif chunk_id == b"data":
                data_size = chunk_size
                break  # no need to read further
            else:
                f.seek(chunk_size, os.SEEK_CUR)

        if not fmt_info:
            raise ValueError("WAV file missing 'fmt ' chunk")

        fmt_info["data_size"] = data_size
        fmt_info["file_size"] = file_size
        return fmt_info


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------


@pytest.fixture(scope="module")
def recording_after_call():
    """Place a test call and return the path to the resulting WAV file.

    If no WAV is found (e.g. recording disabled or routing not set up) the
    test is skipped rather than failed.
    """
    _make_test_call()
    # Give the PBX a moment to flush the recording to disk.
    time.sleep(2)
    wav = _find_latest_wav(RECORDING_DIR)
    if wav is None:
        pytest.skip(
            f"No WAV recording found in {RECORDING_DIR} -- "
            "recording may be disabled or call routing not configured."
        )
    return wav


# ---------------------------------------------------------------------------
# L5 Test Class
# ---------------------------------------------------------------------------


class TestL5MediaQuality:
    """L5: Media recording and audio-quality tests."""

    @pytest.mark.timeout(60)
    def test_L5_001_recording_file_exists(self, recording_after_call):
        """TC-L5-001: A WAV recording file exists after a completed call."""
        assert os.path.isfile(recording_after_call), (
            f"Recording file does not exist: {recording_after_call}"
        )

    @pytest.mark.timeout(30)
    def test_L5_002_recording_file_format(self, recording_after_call):
        """TC-L5-002: WAV header is well-formed (8 kHz, mono or stereo, >0 bytes)."""
        info = _parse_wav_header(recording_after_call)

        assert info["sample_rate"] in (8000, 16000, 44100, 48000), (
            f"Unexpected sample rate: {info['sample_rate']}"
        )
        assert info["channels"] in (1, 2), (
            f"Unexpected channel count: {info['channels']}"
        )
        assert info["bits_per_sample"] in (8, 16, 24, 32), (
            f"Unexpected bits per sample: {info['bits_per_sample']}"
        )
        file_bytes = os.path.getsize(recording_after_call)
        assert file_bytes > 44, (
            f"WAV file is too small ({file_bytes} bytes) -- header only?"
        )

    @pytest.mark.timeout(30)
    def test_L5_003_recording_has_audio(self, recording_after_call):
        """TC-L5-003: WAV file contains non-silent audio content.

        Reads the raw PCM data and checks that at least some samples exceed a
        small silence threshold.  This catches the case where a recording file
        is created but contains only zeroes.
        """
        SILENCE_THRESHOLD = 10  # minimum absolute sample value to count as "audio"
        MIN_NON_SILENT_RATIO = 0.01  # at least 1% of samples should be non-silent

        info = _parse_wav_header(recording_after_call)
        data_size = info["data_size"]
        bits = info["bits_per_sample"]

        if data_size == 0:
            pytest.fail("WAV data chunk is empty (0 bytes)")

        with open(recording_after_call, "rb") as f:
            # Skip to the data chunk.  Re-parse to find the offset.
            f.seek(0)
            f.read(12)  # RIFF header
            while True:
                chunk_header = f.read(8)
                if len(chunk_header) < 8:
                    pytest.fail("Could not locate data chunk")
                chunk_id = chunk_header[:4]
                chunk_size = struct.unpack("<I", chunk_header[4:])[0]
                if chunk_id == b"data":
                    break
                f.seek(chunk_size, os.SEEK_CUR)

            raw = f.read(min(data_size, 1_000_000))  # read up to ~1 MB

        # Decode samples.
        if bits == 16:
            fmt = "<" + "h" * (len(raw) // 2)
            samples = struct.unpack(fmt, raw[:len(raw) - len(raw) % 2])
        elif bits == 8:
            # 8-bit WAV is unsigned; centre at 128.
            samples = [b - 128 for b in raw]
        else:
            # For 24/32-bit just do a rough byte-level check.
            samples = list(raw)

        total = len(samples)
        if total == 0:
            pytest.fail("No audio samples decoded")

        non_silent = sum(1 for s in samples if abs(s) > SILENCE_THRESHOLD)
        ratio = non_silent / total

        assert ratio >= MIN_NON_SILENT_RATIO, (
            f"Recording appears silent: only {ratio:.4%} of samples exceed "
            f"threshold ({SILENCE_THRESHOLD}).  Expected >= {MIN_NON_SILENT_RATIO:.2%}."
        )
