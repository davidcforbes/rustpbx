#!/usr/bin/env python3
"""
groq-sensevoice-wrapper.py — Drop-in replacement for sensevoice-cli
that calls Groq's Whisper API instead of running local ONNX inference.

Outputs the same JSON format as sensevoice-cli so RustPBX transcript
addon can parse it without modification.

Usage (same as sensevoice-cli):
  groq-sensevoice-wrapper.py [--models-path <ignored>] [-o output.json] [--channels=2] input.wav

Environment:
  GROQ_API_KEY — Required. Your Groq API key.

Install: Just copy this script and make it executable. No pip dependencies.
"""

import argparse
import json
import os
import subprocess
import sys
import tempfile
import urllib.request
import urllib.error
import wave
from pathlib import Path


GROQ_API_URL = "https://api.groq.com/openai/v1/audio/transcriptions"
GROQ_MODEL = "whisper-large-v3-turbo"


def parse_args():
    parser = argparse.ArgumentParser(description="Groq Whisper wrapper (sensevoice-cli compatible)")
    parser.add_argument("input", help="Input audio file (WAV, MP3, etc.)")
    parser.add_argument("-o", "--output", help="Output JSON file path")
    parser.add_argument("--models-path", help="Ignored (compatibility with sensevoice-cli)")
    parser.add_argument("--channels", default="2", help="Number of channels (default: 2)")
    parser.add_argument("-l", "--language", default=None, help="Language code (en, zh, etc.)")
    parser.add_argument("--use-itn", action="store_true", help="Ignored (compatibility)")
    parser.add_argument("--no-vad", action="store_true", help="Ignored (compatibility)")
    parser.add_argument("-t", "--threads", default=None, help="Ignored (compatibility)")
    return parser.parse_args()


def get_wav_duration(filepath):
    """Get duration of a WAV file in seconds."""
    try:
        with wave.open(filepath, 'rb') as wf:
            frames = wf.getnframes()
            rate = wf.getframerate()
            return frames / float(rate) if rate > 0 else 0.0
    except Exception:
        return 0.0


def split_stereo_wav(input_path):
    """Split a stereo WAV into two mono WAV files using Python only (no ffmpeg)."""
    try:
        with wave.open(input_path, 'rb') as wf:
            n_channels = wf.getnchannels()
            sampwidth = wf.getsampwidth()
            framerate = wf.getframerate()
            n_frames = wf.getnframes()
            raw_data = wf.readframes(n_frames)
    except Exception:
        return [input_path], 1

    if n_channels < 2:
        return [input_path], 1

    # Split interleaved samples
    bytes_per_sample = sampwidth
    frame_size = n_channels * bytes_per_sample
    channels_data = [bytearray() for _ in range(n_channels)]

    for i in range(n_frames):
        offset = i * frame_size
        for ch in range(n_channels):
            start = offset + ch * bytes_per_sample
            channels_data[ch].extend(raw_data[start:start + bytes_per_sample])

    temp_files = []
    for ch in range(min(n_channels, 2)):  # Only process first 2 channels
        tmp = tempfile.NamedTemporaryFile(suffix=f"_ch{ch}.wav", delete=False)
        with wave.open(tmp.name, 'wb') as out_wf:
            out_wf.setnchannels(1)
            out_wf.setsampwidth(sampwidth)
            out_wf.setframerate(framerate)
            out_wf.writeframes(bytes(channels_data[ch]))
        temp_files.append(tmp.name)

    return temp_files, n_channels


def transcribe_with_groq(audio_path, api_key, language=None):
    """Call Groq Whisper API using curl (avoids needing requests library)."""
    cmd = [
        "curl", "-s", "-X", "POST", GROQ_API_URL,
        "-H", f"Authorization: Bearer {api_key}",
        "-F", f"file=@{audio_path}",
        "-F", f"model={GROQ_MODEL}",
        "-F", "response_format=verbose_json",
        "-F", "timestamp_granularities[]=segment",
    ]
    if language:
        cmd.extend(["-F", f"language={language}"])

    result = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
    if result.returncode != 0:
        print(f"Error: curl failed with code {result.returncode}", file=sys.stderr)
        print(result.stderr, file=sys.stderr)
        sys.exit(1)

    try:
        response = json.loads(result.stdout)
    except json.JSONDecodeError:
        print(f"Error: Invalid JSON response from Groq API", file=sys.stderr)
        print(result.stdout[:500], file=sys.stderr)
        sys.exit(1)

    if "error" in response:
        print(f"Groq API error: {response['error']}", file=sys.stderr)
        sys.exit(1)

    return response


def groq_to_sensevoice_format(groq_response, channel_num, duration):
    """Convert Groq Whisper response to sensevoice-cli JSON format."""
    segments = []
    if "segments" in groq_response:
        for seg in groq_response["segments"]:
            segments.append({
                "start_sec": seg.get("start", 0.0),
                "end_sec": seg.get("end", 0.0),
                "text": seg.get("text", "").strip(),
                "tags": []
            })
    elif "text" in groq_response:
        # Fallback: single segment with full text
        segments.append({
            "start_sec": 0.0,
            "end_sec": duration,
            "text": groq_response["text"].strip(),
            "tags": []
        })

    return {
        "channel": channel_num,
        "duration_sec": duration,
        "rtf": 0.0,
        "segments": segments
    }


def main():
    args = parse_args()
    api_key = os.environ.get("GROQ_API_KEY")
    if not api_key:
        print("Error: GROQ_API_KEY environment variable is required", file=sys.stderr)
        sys.exit(1)

    input_path = args.input
    if not os.path.exists(input_path):
        print(f"Error: Input file not found: {input_path}", file=sys.stderr)
        sys.exit(1)

    duration = get_wav_duration(input_path)
    n_channels = int(args.channels)
    temp_files = []

    try:
        # Split stereo into mono channels for per-channel transcription
        if n_channels >= 2 and input_path.lower().endswith(".wav"):
            channel_files, actual_channels = split_stereo_wav(input_path)
            if actual_channels >= 2:
                temp_files = channel_files
            else:
                channel_files = [input_path]
        else:
            channel_files = [input_path]

        results = []
        for ch_idx, ch_file in enumerate(channel_files):
            print(f"Transcribing channel {ch_idx}...", file=sys.stderr)
            response = transcribe_with_groq(ch_file, api_key, args.language)
            ch_result = groq_to_sensevoice_format(response, ch_idx, duration)
            results.append(ch_result)

        # If only 1 channel file but --channels=2 was specified,
        # add an empty second channel
        while len(results) < n_channels:
            results.append({
                "channel": len(results),
                "duration_sec": duration,
                "rtf": 0.0,
                "segments": []
            })

        output_json = json.dumps(results, indent=2)

        if args.output:
            with open(args.output, 'w') as f:
                f.write(output_json)
            print(f"Transcript written to {args.output}", file=sys.stderr)
        else:
            print(output_json)

    finally:
        # Clean up temp files
        for tf in temp_files:
            try:
                os.unlink(tf)
            except OSError:
                pass


if __name__ == "__main__":
    main()
