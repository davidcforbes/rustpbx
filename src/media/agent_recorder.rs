use anyhow::{Result, anyhow};
use audio_codec::{PcmBuf, samples_to_bytes};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::Path,
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
    u32,
};
use tokio::{
    fs::File,
    io::{AsyncSeekExt, AsyncWriteExt},
    select,
    sync::mpsc::UnboundedReceiver,
};
use tokio_stream::wrappers::IntervalStream;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use crate::media::{AudioFrame, Samples};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RecorderFormat {
    Wav,
    Pcm,
    Pcmu,
    Pcma,
    G722,
}

impl RecorderFormat {
    pub fn extension(&self) -> &'static str {
        "wav"
    }

    pub fn is_supported(&self) -> bool {
        true
    }

    pub fn effective(&self) -> RecorderFormat {
        *self
    }
}

impl Default for RecorderFormat {
    fn default() -> Self {
        RecorderFormat::Wav
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct RecorderOption {
    #[serde(default)]
    pub recorder_file: String,
    #[serde(default)]
    pub samplerate: u32,
    #[serde(default)]
    pub ptime: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<RecorderFormat>,
}

impl RecorderOption {
    pub fn new(recorder_file: String) -> Self {
        Self {
            recorder_file,
            ..Default::default()
        }
    }

    pub fn resolved_format(&self, default: RecorderFormat) -> RecorderFormat {
        self.format.unwrap_or(default).effective()
    }

    pub fn ensure_path_extension(&mut self, fallback_format: RecorderFormat) {
        let effective_format = self.format.unwrap_or(fallback_format).effective();
        self.format = Some(effective_format);

        if self.recorder_file.is_empty() {
            return;
        }

        let extension = effective_format.extension();
        if !self
            .recorder_file
            .to_lowercase()
            .ends_with(&format!(".{}", extension.to_lowercase()))
        {
            self.recorder_file = format!("{}.{}", self.recorder_file, extension);
        }
    }
}

impl Default for RecorderOption {
    fn default() -> Self {
        Self {
            recorder_file: "".to_string(),
            samplerate: 16000,
            ptime: 200,
            format: None,
        }
    }
}

impl From<crate::media::recorder::RecorderOption> for RecorderOption {
    fn from(opt: crate::media::recorder::RecorderOption) -> Self {
        Self {
            recorder_file: opt.recorder_file,
            samplerate: opt.samplerate,
            ptime: opt.ptime,
            format: None,
        }
    }
}

pub struct Recorder {
    session_id: String,
    option: RecorderOption,
    samples_written: AtomicUsize,
    cancel_token: CancellationToken,
    channel_idx: AtomicUsize,
    channels: Mutex<HashMap<String, usize>>,
    stereo_buf: Mutex<PcmBuf>,
    mono_buf: Mutex<PcmBuf>,
}

impl Recorder {
    pub fn new(
        cancel_token: CancellationToken,
        session_id: String,
        option: RecorderOption,
    ) -> Self {
        Self {
            session_id,
            option,
            samples_written: AtomicUsize::new(0),
            cancel_token,
            channel_idx: AtomicUsize::new(0),
            channels: Mutex::new(HashMap::new()),
            stereo_buf: Mutex::new(Vec::new()),
            mono_buf: Mutex::new(Vec::new()),
        }
    }

    async fn update_wav_header(&self, file: &mut File, payload_type: Option<u8>) -> Result<()> {
        let total = self.samples_written.load(Ordering::SeqCst);

        let (format_tag, sample_rate, channels, bits_per_sample, data_size): (
            u16,
            u32,
            u16,
            u16,
            usize,
        ) = match payload_type {
            Some(pt) => {
                let (tag, rate, chan): (u16, u32, u16) = match pt {
                    0 => (0x0007, 8000, 1),   // PCMU
                    8 => (0x0006, 8000, 1),   // PCMA
                    9 => (0x0064, 16000, 1),  // G722
                    10 => (0x0001, 44100, 2), // L16 Stereo 44.1k
                    11 => (0x0001, 44100, 1), // L16 Mono 44.1k
                    _ => (0x0001, 16000, 1),  // Default to PCM 16k Mono
                };
                let bits: u16 = match pt {
                    9 => 4,
                    0 | 8 => 8,
                    _ => 16,
                };
                (tag, rate, chan, bits, total)
            }
            None => (0x0001, self.option.samplerate, 2, 16, total),
        };

        let mut header_buf = Vec::new();
        header_buf.extend_from_slice(b"RIFF");
        let file_size = data_size + 36;
        header_buf.extend_from_slice(&(file_size as u32).to_le_bytes());
        header_buf.extend_from_slice(b"WAVE");

        header_buf.extend_from_slice(b"fmt ");
        header_buf.extend_from_slice(&16u32.to_le_bytes());
        header_buf.extend_from_slice(&format_tag.to_le_bytes());
        header_buf.extend_from_slice(&(channels as u16).to_le_bytes());
        header_buf.extend_from_slice(&sample_rate.to_le_bytes());

        let bytes_per_sec: u32 = match format_tag {
            0x0064 => 8000, // G.722 is 64kbps
            _ => sample_rate * (channels as u32) * (bits_per_sample as u32 / 8),
        };
        header_buf.extend_from_slice(&bytes_per_sec.to_le_bytes());

        let block_align: u16 = match format_tag {
            0x0064 | 0x0007 | 0x0006 => 1 * channels,
            _ => (bits_per_sample / 8) * channels,
        };
        header_buf.extend_from_slice(&block_align.to_le_bytes());
        header_buf.extend_from_slice(&bits_per_sample.to_le_bytes());

        header_buf.extend_from_slice(b"data");
        header_buf.extend_from_slice(&(data_size as u32).to_le_bytes());

        file.seek(std::io::SeekFrom::Start(0)).await?;
        file.write_all(&header_buf).await?;
        file.seek(std::io::SeekFrom::End(0)).await?;

        Ok(())
    }

    pub async fn process_recording(
        &self,
        file_path: &Path,
        mut receiver: UnboundedReceiver<AudioFrame>,
    ) -> Result<()> {
        let first_frame = match receiver.recv().await {
            Some(f) => f,
            None => return Ok(()),
        };

        if let Samples::RTP { .. } = first_frame.samples {
            return self
                .process_recording_rtp(file_path, receiver, first_frame)
                .await;
        }

        let _requested_format = self.option.format.unwrap_or(RecorderFormat::Wav);

        self.process_recording_wav(file_path, receiver, first_frame)
            .await
    }

    fn ensure_parent_dir(&self, file_path: &Path) -> Result<()> {
        if let Some(parent) = file_path.parent() {
            if !parent.exists() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    warn!(
                        "Failed to create recording file parent directory: {} {}",
                        e,
                        file_path.display()
                    );
                    return Err(anyhow!("Failed to create recording file parent directory"));
                }
            }
        }
        Ok(())
    }

    async fn create_output_file(&self, file_path: &Path) -> Result<File> {
        self.ensure_parent_dir(file_path)?;
        match File::create(file_path).await {
            Ok(file) => {
                info!(
                    session_id = self.session_id,
                    "recorder: created recording file: {}",
                    file_path.display()
                );
                Ok(file)
            }
            Err(e) => {
                warn!(
                    "Failed to create recording file: {} {}",
                    e,
                    file_path.display()
                );
                Err(anyhow!("Failed to create recording file"))
            }
        }
    }

    async fn process_recording_rtp(
        &self,
        file_path: &Path,
        mut receiver: UnboundedReceiver<AudioFrame>,
        first_frame: AudioFrame,
    ) -> Result<()> {
        let (payload_type, mut file) =
            if let Samples::RTP { payload_type, .. } = &first_frame.samples {
                let file = self.create_output_file(file_path).await?;
                (*payload_type, file)
            } else {
                return Err(anyhow!("Invalid frame type for RTP recording"));
            };

        self.update_wav_header(&mut file, Some(payload_type))
            .await?;

        if let Samples::RTP { payload, .. } = first_frame.samples {
            file.write_all(&payload).await?;
            self.samples_written
                .fetch_add(payload.len(), Ordering::SeqCst);
        }

        loop {
            match receiver.recv().await {
                Some(frame) => {
                    if let Samples::RTP { payload, .. } = frame.samples {
                        file.write_all(&payload).await?;
                        self.samples_written
                            .fetch_add(payload.len(), Ordering::SeqCst);
                    }
                }
                None => break,
            }
        }

        self.update_wav_header(&mut file, Some(payload_type))
            .await?;

        file.sync_all().await?;

        Ok(())
    }

    async fn process_recording_wav(
        &self,
        file_path: &Path,
        mut receiver: UnboundedReceiver<AudioFrame>,
        first_frame: AudioFrame,
    ) -> Result<()> {
        let mut file = self.create_output_file(file_path).await?;
        self.update_wav_header(&mut file, None).await?;

        self.append_frame(first_frame).await.ok();

        let chunk_size = (self.option.samplerate / 1000 * self.option.ptime) as usize;
        info!(
            session_id = self.session_id,
            format = "wav",
            "Recording to {} ptime: {}ms chunk_size: {}",
            file_path.display(),
            self.option.ptime,
            chunk_size
        );

        let mut interval = IntervalStream::new(tokio::time::interval(Duration::from_millis(
            self.option.ptime as u64,
        )));
        loop {
            select! {
                Some(frame) = receiver.recv() => {
                    self.append_frame(frame).await.ok();
                }
                _ = interval.next() => {
                    let (mono_buf, stereo_buf) = self.pop(chunk_size).await;
                    self.process_buffers(&mut file, mono_buf, stereo_buf).await?;
                    self.update_wav_header(&mut file, None).await?;
                }
                _ = self.cancel_token.cancelled() => {
                    self.flush_buffers(&mut file).await?;
                    self.update_wav_header(&mut file, None).await?;
                    return Ok(());
                }
            }
        }
    }

    fn get_channel_index(&self, track_id: &str) -> usize {
        let mut channels = self.channels.lock().unwrap();
        if let Some(&channel_idx) = channels.get(track_id) {
            channel_idx % 2
        } else {
            let new_idx = self.channel_idx.fetch_add(1, Ordering::SeqCst);
            channels.insert(track_id.to_string(), new_idx);
            info!(
                session_id = self.session_id,
                "Assigned channel {} to track: {}",
                new_idx % 2,
                track_id
            );
            new_idx % 2
        }
    }

    async fn append_frame(&self, frame: AudioFrame) -> Result<()> {
        let buffer = match frame.samples {
            Samples::PCM { samples } => samples,
            _ => return Ok(()), // ignore non-PCM frames
        };

        if buffer.is_empty() {
            return Ok(());
        }

        let channel_idx = self.get_channel_index(&frame.track_id);
        match channel_idx {
            0 => {
                let mut mono_buf = self.mono_buf.lock().unwrap();
                mono_buf.extend(buffer.iter());
            }
            1 => {
                let mut stereo_buf = self.stereo_buf.lock().unwrap();
                stereo_buf.extend(buffer.iter());
            }
            _ => {}
        }

        Ok(())
    }

    pub(crate) fn extract_samples(buffer: &mut PcmBuf, extract_size: usize) -> PcmBuf {
        if extract_size > 0 && !buffer.is_empty() {
            let take_size = extract_size.min(buffer.len());
            buffer.drain(..take_size).collect()
        } else {
            Vec::new()
        }
    }

    async fn pop(&self, chunk_size: usize) -> (PcmBuf, PcmBuf) {
        let mut mono_buf = self.mono_buf.lock().unwrap();
        let mut stereo_buf = self.stereo_buf.lock().unwrap();

        let safe_chunk_size = chunk_size.min(16000 * 10);

        let mono_result = if mono_buf.len() >= safe_chunk_size {
            Self::extract_samples(&mut mono_buf, safe_chunk_size)
        } else if !mono_buf.is_empty() {
            let available_len = mono_buf.len();
            let mut result = Self::extract_samples(&mut mono_buf, available_len);
            if chunk_size != usize::MAX {
                result.resize(safe_chunk_size, 0);
            }
            result
        } else {
            if chunk_size != usize::MAX {
                vec![0; safe_chunk_size]
            } else {
                Vec::new()
            }
        };

        let stereo_result = if stereo_buf.len() >= safe_chunk_size {
            Self::extract_samples(&mut stereo_buf, safe_chunk_size)
        } else if !stereo_buf.is_empty() {
            let available_len = stereo_buf.len();
            let mut result = Self::extract_samples(&mut stereo_buf, available_len);
            if chunk_size != usize::MAX {
                result.resize(safe_chunk_size, 0);
            }
            result
        } else {
            if chunk_size != usize::MAX {
                vec![0; safe_chunk_size]
            } else {
                Vec::new()
            }
        };

        if chunk_size == usize::MAX {
            let max_len = mono_result.len().max(stereo_result.len());
            let mut mono_final = mono_result;
            let mut stereo_final = stereo_result;
            mono_final.resize(max_len, 0);
            stereo_final.resize(max_len, 0);
            (mono_final, stereo_final)
        } else {
            (mono_result, stereo_result)
        }
    }

    pub fn stop_recording(&self) -> Result<()> {
        self.cancel_token.cancel();
        Ok(())
    }

    pub(crate) fn mix_buffers(mono_buf: &PcmBuf, stereo_buf: &PcmBuf) -> Vec<i16> {
        assert_eq!(
            mono_buf.len(),
            stereo_buf.len(),
            "Buffer lengths must be equal after pop()"
        );

        let len = mono_buf.len();
        let mut mix_buff = Vec::with_capacity(len * 2);

        for i in 0..len {
            mix_buff.push(mono_buf[i]);
            mix_buff.push(stereo_buf[i]);
        }

        mix_buff
    }

    async fn write_audio_data(
        &self,
        file: &mut File,
        mono_buf: &PcmBuf,
        stereo_buf: &PcmBuf,
    ) -> Result<usize> {
        let max_len = mono_buf.len().max(stereo_buf.len());
        if max_len == 0 {
            return Ok(0);
        }

        let mix_buff = Self::mix_buffers(mono_buf, stereo_buf);

        file.seek(std::io::SeekFrom::End(0)).await?;
        file.write_all(&samples_to_bytes(&mix_buff)).await?;

        Ok(max_len)
    }

    async fn process_buffers(
        &self,
        file: &mut File,
        mono_buf: PcmBuf,
        stereo_buf: PcmBuf,
    ) -> Result<()> {
        if mono_buf.is_empty() && stereo_buf.is_empty() {
            return Ok(());
        }
        let samples_written = self.write_audio_data(file, &mono_buf, &stereo_buf).await?;
        if samples_written > 0 {
            self.samples_written
                .fetch_add(samples_written * 4, Ordering::SeqCst);
        }
        Ok(())
    }

    async fn flush_buffers(&self, file: &mut File) -> Result<()> {
        loop {
            let (mono_buf, stereo_buf) = self.pop(usize::MAX).await;

            if mono_buf.is_empty() && stereo_buf.is_empty() {
                break;
            }

            let samples_written = self.write_audio_data(file, &mono_buf, &stereo_buf).await?;
            if samples_written > 0 {
                self.samples_written
                    .fetch_add(samples_written * 4, Ordering::SeqCst);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── RecorderFormat ──────────────────────────────────────────────────

    #[test]
    fn test_recorder_format_default_is_wav() {
        assert_eq!(RecorderFormat::default(), RecorderFormat::Wav);
    }

    #[test]
    fn test_recorder_format_extension_always_wav() {
        assert_eq!(RecorderFormat::Wav.extension(), "wav");
        assert_eq!(RecorderFormat::Pcm.extension(), "wav");
        assert_eq!(RecorderFormat::Pcmu.extension(), "wav");
        assert_eq!(RecorderFormat::Pcma.extension(), "wav");
        assert_eq!(RecorderFormat::G722.extension(), "wav");
    }

    #[test]
    fn test_recorder_format_all_supported() {
        assert!(RecorderFormat::Wav.is_supported());
        assert!(RecorderFormat::Pcm.is_supported());
        assert!(RecorderFormat::Pcmu.is_supported());
        assert!(RecorderFormat::Pcma.is_supported());
        assert!(RecorderFormat::G722.is_supported());
    }

    #[test]
    fn test_recorder_format_effective_returns_self() {
        assert_eq!(RecorderFormat::Pcmu.effective(), RecorderFormat::Pcmu);
        assert_eq!(RecorderFormat::G722.effective(), RecorderFormat::G722);
    }

    #[test]
    fn test_recorder_format_serde_roundtrip() {
        let json = serde_json::to_string(&RecorderFormat::Pcmu).unwrap();
        assert_eq!(json, r#""pcmu""#);
        let parsed: RecorderFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, RecorderFormat::Pcmu);
    }

    // ── RecorderOption ──────────────────────────────────────────────────

    #[test]
    fn test_recorder_option_defaults() {
        let opt = RecorderOption::default();
        assert_eq!(opt.recorder_file, "");
        assert_eq!(opt.samplerate, 16000);
        assert_eq!(opt.ptime, 200);
        assert!(opt.format.is_none());
    }

    #[test]
    fn test_recorder_option_new_sets_file() {
        let opt = RecorderOption::new("recording.wav".to_string());
        assert_eq!(opt.recorder_file, "recording.wav");
        assert_eq!(opt.samplerate, 16000);
        assert_eq!(opt.ptime, 200);
        assert!(opt.format.is_none());
    }

    #[test]
    fn test_resolved_format_uses_explicit() {
        let opt = RecorderOption {
            format: Some(RecorderFormat::Pcmu),
            ..Default::default()
        };
        assert_eq!(opt.resolved_format(RecorderFormat::Wav), RecorderFormat::Pcmu);
    }

    #[test]
    fn test_resolved_format_falls_back_to_default() {
        let opt = RecorderOption::default();
        assert_eq!(opt.resolved_format(RecorderFormat::G722), RecorderFormat::G722);
    }

    #[test]
    fn test_ensure_path_extension_adds_wav() {
        let mut opt = RecorderOption::new("recording".to_string());
        opt.ensure_path_extension(RecorderFormat::Wav);
        assert_eq!(opt.recorder_file, "recording.wav");
        assert_eq!(opt.format, Some(RecorderFormat::Wav));
    }

    #[test]
    fn test_ensure_path_extension_no_double_extension() {
        let mut opt = RecorderOption::new("recording.wav".to_string());
        opt.ensure_path_extension(RecorderFormat::Wav);
        assert_eq!(opt.recorder_file, "recording.wav");
    }

    #[test]
    fn test_ensure_path_extension_empty_filename_noop() {
        let mut opt = RecorderOption::new("".to_string());
        opt.ensure_path_extension(RecorderFormat::Wav);
        assert_eq!(opt.recorder_file, "");
        assert_eq!(opt.format, Some(RecorderFormat::Wav));
    }

    #[test]
    fn test_ensure_path_extension_case_insensitive() {
        let mut opt = RecorderOption::new("recording.WAV".to_string());
        opt.ensure_path_extension(RecorderFormat::Wav);
        assert_eq!(opt.recorder_file, "recording.WAV");
    }

    #[test]
    fn test_recorder_option_camelcase_deserialization() {
        let json = r#"{
            "recorderFile": "test.wav",
            "samplerate": 8000,
            "ptime": 100,
            "format": "pcmu"
        }"#;
        let opt: RecorderOption = serde_json::from_str(json).unwrap();
        assert_eq!(opt.recorder_file, "test.wav");
        assert_eq!(opt.samplerate, 8000);
        assert_eq!(opt.ptime, 100);
        assert_eq!(opt.format, Some(RecorderFormat::Pcmu));
    }

    #[test]
    fn test_recorder_option_deserialization_defaults() {
        // Field-level #[serde(default)] uses type defaults (u32 → 0),
        // not the struct-level Default impl values.
        let json = r#"{}"#;
        let opt: RecorderOption = serde_json::from_str(json).unwrap();
        assert_eq!(opt.recorder_file, "");
        assert_eq!(opt.samplerate, 0);
        assert_eq!(opt.ptime, 0);
        assert!(opt.format.is_none());
    }

    // ── extract_samples ─────────────────────────────────────────────────

    #[test]
    fn test_extract_samples_basic() {
        let mut buf = vec![1, 2, 3, 4, 5];
        let extracted = Recorder::extract_samples(&mut buf, 3);
        assert_eq!(extracted, vec![1, 2, 3]);
        assert_eq!(buf, vec![4, 5]);
    }

    #[test]
    fn test_extract_samples_empty_buffer() {
        let mut buf: PcmBuf = vec![];
        let extracted = Recorder::extract_samples(&mut buf, 5);
        assert!(extracted.is_empty());
    }

    #[test]
    fn test_extract_samples_zero_size() {
        let mut buf = vec![1, 2, 3];
        let extracted = Recorder::extract_samples(&mut buf, 0);
        assert!(extracted.is_empty());
        assert_eq!(buf, vec![1, 2, 3]);
    }

    #[test]
    fn test_extract_samples_more_than_available() {
        let mut buf = vec![10, 20];
        let extracted = Recorder::extract_samples(&mut buf, 100);
        assert_eq!(extracted, vec![10, 20]);
        assert!(buf.is_empty());
    }

    #[test]
    fn test_extract_samples_exact_size() {
        let mut buf = vec![1, 2, 3];
        let extracted = Recorder::extract_samples(&mut buf, 3);
        assert_eq!(extracted, vec![1, 2, 3]);
        assert!(buf.is_empty());
    }

    // ── mix_buffers ─────────────────────────────────────────────────────

    #[test]
    fn test_mix_buffers_interleaves_channels() {
        let mono = vec![1, 2, 3];
        let stereo = vec![10, 20, 30];
        let mixed = Recorder::mix_buffers(&mono, &stereo);
        assert_eq!(mixed, vec![1, 10, 2, 20, 3, 30]);
    }

    #[test]
    fn test_mix_buffers_empty() {
        let mono: PcmBuf = vec![];
        let stereo: PcmBuf = vec![];
        let mixed = Recorder::mix_buffers(&mono, &stereo);
        assert!(mixed.is_empty());
    }

    #[test]
    #[should_panic(expected = "Buffer lengths must be equal")]
    fn test_mix_buffers_panics_on_unequal_lengths() {
        let mono = vec![1, 2, 3];
        let stereo = vec![10, 20];
        Recorder::mix_buffers(&mono, &stereo);
    }

    // ── Recorder lifecycle ──────────────────────────────────────────────

    #[test]
    fn test_recorder_new_initializes_correctly() {
        let token = CancellationToken::new();
        let opt = RecorderOption::new("test.wav".to_string());
        let recorder = Recorder::new(token.clone(), "session-1".to_string(), opt);
        assert_eq!(recorder.session_id, "session-1");
        assert_eq!(recorder.samples_written.load(Ordering::SeqCst), 0);
        assert!(!token.is_cancelled());
    }

    #[test]
    fn test_recorder_stop_cancels_token() {
        let token = CancellationToken::new();
        let opt = RecorderOption::new("test.wav".to_string());
        let recorder = Recorder::new(token.clone(), "session-2".to_string(), opt);
        assert!(!token.is_cancelled());
        recorder.stop_recording().unwrap();
        assert!(token.is_cancelled());
    }

    #[test]
    fn test_channel_assignment_first_track_gets_channel_0() {
        let token = CancellationToken::new();
        let opt = RecorderOption::new("test.wav".to_string());
        let recorder = Recorder::new(token, "session-3".to_string(), opt);
        assert_eq!(recorder.get_channel_index("track-a"), 0);
    }

    #[test]
    fn test_channel_assignment_second_track_gets_channel_1() {
        let token = CancellationToken::new();
        let opt = RecorderOption::new("test.wav".to_string());
        let recorder = Recorder::new(token, "session-4".to_string(), opt);
        assert_eq!(recorder.get_channel_index("track-a"), 0);
        assert_eq!(recorder.get_channel_index("track-b"), 1);
    }

    #[test]
    fn test_channel_assignment_same_track_returns_same_channel() {
        let token = CancellationToken::new();
        let opt = RecorderOption::new("test.wav".to_string());
        let recorder = Recorder::new(token, "session-5".to_string(), opt);
        assert_eq!(recorder.get_channel_index("track-a"), 0);
        assert_eq!(recorder.get_channel_index("track-a"), 0);
        assert_eq!(recorder.get_channel_index("track-b"), 1);
        assert_eq!(recorder.get_channel_index("track-b"), 1);
    }

    #[test]
    fn test_channel_assignment_wraps_around_mod_2() {
        let token = CancellationToken::new();
        let opt = RecorderOption::new("test.wav".to_string());
        let recorder = Recorder::new(token, "session-6".to_string(), opt);
        assert_eq!(recorder.get_channel_index("t0"), 0);
        assert_eq!(recorder.get_channel_index("t1"), 1);
        assert_eq!(recorder.get_channel_index("t2"), 0); // 2 % 2 = 0
        assert_eq!(recorder.get_channel_index("t3"), 1); // 3 % 2 = 1
    }

    // ── Test helpers ──────────────────────────────────────────────────────

    fn pcm_frame(track_id: &str, samples: Vec<i16>) -> AudioFrame {
        AudioFrame {
            track_id: track_id.to_string(),
            samples: Samples::PCM { samples },
            sample_rate: 16000,
            channels: 1,
            timestamp: 0,
            src_packet: None,
        }
    }

    fn rtp_frame(track_id: &str, payload_type: u8, payload: Vec<u8>) -> AudioFrame {
        AudioFrame {
            track_id: track_id.to_string(),
            samples: Samples::RTP {
                payload_type,
                payload,
                sequence_number: 0,
            },
            sample_rate: 8000,
            channels: 1,
            timestamp: 0,
            src_packet: None,
        }
    }

    // ── append_frame / PCM recording ────────────────────────────────────

    #[tokio::test]
    async fn test_append_frame_pcm_mono_channel() {
        let token = CancellationToken::new();
        let opt = RecorderOption::new("test.wav".to_string());
        let recorder = Recorder::new(token, "session-7".to_string(), opt);

        recorder.append_frame(pcm_frame("track-a", vec![100, 200, 300])).await.unwrap();

        let mono = recorder.mono_buf.lock().unwrap();
        assert_eq!(*mono, vec![100, 200, 300]);
        let stereo = recorder.stereo_buf.lock().unwrap();
        assert!(stereo.is_empty());
    }

    #[tokio::test]
    async fn test_append_frame_pcm_stereo_channel() {
        let token = CancellationToken::new();
        let opt = RecorderOption::new("test.wav".to_string());
        let recorder = Recorder::new(token, "session-8".to_string(), opt);

        // First track goes to mono (channel 0)
        recorder.append_frame(pcm_frame("track-a", vec![10])).await.unwrap();
        // Second track goes to stereo (channel 1)
        recorder.append_frame(pcm_frame("track-b", vec![20])).await.unwrap();

        let mono = recorder.mono_buf.lock().unwrap();
        assert_eq!(*mono, vec![10]);
        let stereo = recorder.stereo_buf.lock().unwrap();
        assert_eq!(*stereo, vec![20]);
    }

    #[tokio::test]
    async fn test_append_frame_ignores_empty_pcm() {
        let token = CancellationToken::new();
        let opt = RecorderOption::new("test.wav".to_string());
        let recorder = Recorder::new(token, "session-9".to_string(), opt);

        recorder.append_frame(pcm_frame("track-a", vec![])).await.unwrap();

        let mono = recorder.mono_buf.lock().unwrap();
        assert!(mono.is_empty());
    }

    #[tokio::test]
    async fn test_append_frame_ignores_rtp() {
        let token = CancellationToken::new();
        let opt = RecorderOption::new("test.wav".to_string());
        let recorder = Recorder::new(token, "session-10".to_string(), opt);

        recorder.append_frame(rtp_frame("track-a", 0, vec![0x80, 0x00])).await.unwrap();

        let mono = recorder.mono_buf.lock().unwrap();
        assert!(mono.is_empty());
    }

    // ── WAV file recording integration ──────────────────────────────────

    #[tokio::test]
    async fn test_process_recording_creates_wav_file() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("output.wav");

        let token = CancellationToken::new();
        let opt = RecorderOption {
            recorder_file: file_path.to_string_lossy().to_string(),
            samplerate: 16000,
            ptime: 200,
            ..Default::default()
        };
        let recorder = Recorder::new(token.clone(), "session-wav".to_string(), opt);

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        tx.send(pcm_frame("caller", vec![1000; 160])).unwrap();
        tx.send(pcm_frame("callee", vec![2000; 160])).unwrap();

        // Cancel after a short delay to stop recording
        let token_clone = token.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(300)).await;
            token_clone.cancel();
        });

        recorder.process_recording(&file_path, rx).await.unwrap();

        // Verify file was created and has WAV header
        let data = tokio::fs::read(&file_path).await.unwrap();
        assert!(data.len() > 44, "WAV file should be larger than header");
        assert_eq!(&data[0..4], b"RIFF");
        assert_eq!(&data[8..12], b"WAVE");
        assert_eq!(&data[12..16], b"fmt ");
    }

    #[tokio::test]
    async fn test_process_recording_rtp_creates_wav_file() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("rtp_output.wav");

        let token = CancellationToken::new();
        let opt = RecorderOption::new(file_path.to_string_lossy().to_string());
        let recorder = Recorder::new(token, "session-rtp".to_string(), opt);

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        // Send RTP frames (PCMU payload type 0)
        tx.send(rtp_frame("rtp-track", 0, vec![0x7F; 160])).unwrap();
        tx.send(rtp_frame("rtp-track", 0, vec![0x7F; 160])).unwrap();
        drop(tx); // Close channel to end recording

        recorder.process_recording(&file_path, rx).await.unwrap();

        let data = tokio::fs::read(&file_path).await.unwrap();
        assert!(data.len() > 44);
        assert_eq!(&data[0..4], b"RIFF");
        // PCMU format tag is 0x0007
        assert_eq!(data[20], 0x07);
        assert_eq!(data[21], 0x00);
    }
}
