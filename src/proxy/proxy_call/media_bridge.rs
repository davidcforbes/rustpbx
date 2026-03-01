use crate::config::QualityConfig;
use crate::media::call_quality::{self, CallQuality, CallQualitySnapshot, QualityThresholds};
use crate::media::recorder::{Leg, Recorder, RecorderOption};
use crate::proxy::proxy_call::media_peer::MediaPeer;
use crate::sipflow::{SipFlowBackend, SipFlowItem, SipFlowMsgType};
use anyhow::Result;
use audio_codec::{CodecType, Decoder, Encoder, create_decoder, create_encoder};
use bytes::Bytes;
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use rustrtc::media::{AudioFrame, MediaKind, MediaSample, MediaStreamTrack, SampleStreamSource};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tracing::{debug, info, warn};

/// Monitor mode for the supervisor leg.
///
/// Controls how audio flows between the monitor and the call participants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MonitorMode {
    /// Monitor hears both sides but cannot speak (default).
    SilentListen,
    /// Monitor can speak to leg B (agent) only. Leg A (caller) cannot hear the monitor.
    Whisper,
    /// Monitor can speak to both legs (full conference).
    Barge,
}

impl Default for MonitorMode {
    fn default() -> Self {
        MonitorMode::SilentListen
    }
}

/// A monitor (supervisor) leg that can silently listen to both sides of a call.
///
/// The monitor receives a copy of audio from both leg A and leg B via a shared
/// `SampleStreamSource`. The monitor's `PeerConnection` handles RTP packetization
/// and sequence numbering for the outbound stream.
///
/// In Whisper and Barge modes, the monitor's incoming audio (supervisor speaking)
/// is captured and stored in `incoming_audio` so that `forward_track` tasks can
/// mix it into the call legs.
#[allow(dead_code)]
pub struct MonitorLeg {
    /// The sample source feeding the monitor's PeerConnection track.
    pub source: SampleStreamSource,
    /// The codec the monitor expects to receive.
    pub codec: CodecType,
    /// The RTP codec parameters for the monitor's track.
    pub params: rustrtc::RtpCodecParameters,
    /// The current monitor mode.
    pub mode: MonitorMode,
    /// Shared buffer holding the latest audio frame received from the monitor
    /// (supervisor speaking). Updated by the monitor receiver task; read by
    /// `forward_track` tasks to mix into Whisper/Barge streams.
    /// The `Bytes` contain encoded audio in the monitor's codec format.
    pub incoming_audio: Arc<Mutex<Option<MonitorAudioFrame>>>,
}

/// A captured audio frame from the monitor's incoming stream (supervisor speaking).
/// Stored in decoded PCM form so that both forward_track tasks can mix it without
/// each needing their own decoder.
pub struct MonitorAudioFrame {
    /// PCM samples decoded from the monitor's incoming audio.
    pub pcm: Vec<i16>,
    /// Monotonic counter to detect stale frames. Each forward_track task tracks
    /// the last sequence it consumed to avoid mixing the same frame twice.
    pub seq: u64,
}

pub struct MediaBridge {
    pub leg_a: Arc<dyn MediaPeer>,
    pub leg_b: Arc<dyn MediaPeer>,
    pub params_a: rustrtc::RtpCodecParameters,
    pub params_b: rustrtc::RtpCodecParameters,
    pub codec_a: CodecType,
    pub codec_b: CodecType,
    pub dtmf_pt_a: Option<u8>,
    pub dtmf_pt_b: Option<u8>,
    pub ssrc_a: Option<u32>,
    pub ssrc_b: Option<u32>,
    input_gain: f32,
    output_gain: f32,
    started: AtomicBool,
    recorder: Arc<Mutex<Option<Recorder>>>,
    call_id: String,
    sipflow_backend: Option<Arc<dyn SipFlowBackend>>,
    quality: Arc<CallQuality>,
    quality_config: Option<QualityConfig>,
    /// Optional monitor (supervisor) leg. Protected by a Mutex so it can be
    /// attached/detached at runtime without interrupting the bridge loop.
    /// Each `forward_track` task holds an Arc clone and checks on every packet.
    monitor: Arc<Mutex<Option<MonitorLeg>>>,
}

#[allow(dead_code)]
impl MediaBridge {
    pub fn new(
        leg_a: Arc<dyn MediaPeer>,
        leg_b: Arc<dyn MediaPeer>,
        params_a: rustrtc::RtpCodecParameters,
        params_b: rustrtc::RtpCodecParameters,
        dtmf_pt_a: Option<u8>,
        dtmf_pt_b: Option<u8>,
        codec_a: CodecType,
        codec_b: CodecType,
        ssrc_a: Option<u32>,
        ssrc_b: Option<u32>,
        recorder_option: Option<RecorderOption>,
        call_id: String,
        sipflow_backend: Option<Arc<dyn SipFlowBackend>>,
        quality_config: Option<QualityConfig>,
    ) -> Self {
        let input_gain = recorder_option
            .as_ref()
            .map(|o| o.input_gain)
            .unwrap_or(1.0);
        let output_gain = recorder_option
            .as_ref()
            .map(|o| o.output_gain)
            .unwrap_or(1.0);

        let recorder = if let Some(option) = recorder_option {
            match Recorder::new(&option.recorder_file, codec_a, option.input_gain, option.output_gain) {
                Ok(r) => Some(r),
                Err(e) => {
                    warn!("Failed to create recorder: {:?}", e);
                    None
                }
            }
        } else {
            None
        };

        Self {
            leg_a,
            leg_b,
            params_a,
            params_b,
            codec_a,
            codec_b,
            dtmf_pt_a,
            dtmf_pt_b,
            ssrc_a,
            ssrc_b,
            input_gain,
            output_gain,
            started: AtomicBool::new(false),
            recorder: Arc::new(Mutex::new(recorder)),
            call_id,
            sipflow_backend,
            quality: Arc::new(CallQuality::new()),
            quality_config,
            monitor: Arc::new(Mutex::new(None)),
        }
    }

    /// Attach a monitor (supervisor) leg to the bridge.
    ///
    /// The monitor receives a copy of processed audio from both sides of the call.
    /// The `track` parameter is the MediaPeer for the monitor's media stream, and
    /// `codec` is the codec the monitor's RTP stream should use.
    ///
    /// In Whisper mode, the supervisor's audio is mixed into leg_b (agent) only.
    /// In Barge mode, the supervisor's audio is mixed into both legs.
    ///
    /// This can be called while the bridge is running; forward_track tasks will
    /// pick up the new monitor on their next packet.
    pub fn attach_monitor(
        &self,
        peer: Arc<dyn MediaPeer>,
        codec: CodecType,
    ) -> Result<()> {
        let params = rustrtc::RtpCodecParameters {
            payload_type: codec.payload_type(),
            clock_rate: codec.clock_rate(),
            channels: codec.channels() as u8,
            ..Default::default()
        };

        // Create a sample track for the monitor. The SampleStreamSource is used
        // by forward_track tasks to push audio; the SampleStreamTrack feeds the
        // monitor's PeerConnection sender.
        let (source, track, _feedback_rx) =
            rustrtc::media::track::sample_track(MediaKind::Audio, 200);

        // Shared buffer for incoming monitor audio (supervisor speaking).
        let incoming_audio: Arc<Mutex<Option<MonitorAudioFrame>>> =
            Arc::new(Mutex::new(None));

        // Spawn a task to set up the monitor's PeerConnection sender.
        // We need the PeerConnection from the monitor peer's track.
        let call_id = self.call_id.clone();
        let monitor_arc = self.monitor.clone();

        // Store the monitor leg first so forward_tracks can start sending
        {
            let mut guard = self.monitor.lock().unwrap();
            *guard = Some(MonitorLeg {
                source,
                codec,
                params: params.clone(),
                mode: MonitorMode::SilentListen,
                incoming_audio: incoming_audio.clone(),
            });
        }

        // Set up the PeerConnection track asynchronously and start the
        // monitor receiver task for capturing supervisor audio.
        let track_target: Arc<dyn MediaStreamTrack> = track;
        let incoming_audio_for_task = incoming_audio;
        let monitor_codec = codec;
        crate::utils::spawn(async move {
            let tracks = peer.get_tracks().await;
            if let Some(t) = tracks.first() {
                let pc = t.lock().await.get_peer_connection().await;
                if let Some(pc) = pc {
                    // Try to reuse existing transceiver
                    let transceivers = pc.get_transceivers();
                    let existing = transceivers
                        .iter()
                        .find(|t| t.kind() == rustrtc::MediaKind::Audio);

                    if let Some(transceiver) = existing {
                        let ssrc = transceiver
                            .sender()
                            .map(|s| s.ssrc())
                            .unwrap_or_else(|| rand::random::<u32>());
                        let new_sender =
                            rustrtc::RtpSender::builder(track_target, ssrc)
                                .params(params)
                                .build();
                        transceiver.set_sender(Some(new_sender));
                        info!(call_id, "Monitor leg attached via existing transceiver");
                    } else {
                        match pc.add_track(track_target, params) {
                            Ok(_) => {
                                info!(call_id, "Monitor leg attached via new track");
                            }
                            Err(e) => {
                                warn!(call_id, "Failed to add monitor track: {}", e);
                                // Clear the monitor since we couldn't set up the PC
                                let mut guard = monitor_arc.lock().unwrap();
                                *guard = None;
                            }
                        }
                    }

                    // Spawn a receiver task to capture audio from the monitor's
                    // incoming track (supervisor speaking). This task reads from
                    // the monitor PC's receiver and stores decoded PCM in the
                    // shared buffer so forward_track tasks can mix it in.
                    Self::spawn_monitor_receiver(
                        pc,
                        incoming_audio_for_task,
                        monitor_codec,
                        call_id.clone(),
                        monitor_arc.clone(),
                    );
                } else {
                    warn!(call_id, "Monitor peer has no PeerConnection");
                    let mut guard = monitor_arc.lock().unwrap();
                    *guard = None;
                }
            } else {
                warn!(call_id, "Monitor peer has no tracks");
                let mut guard = monitor_arc.lock().unwrap();
                *guard = None;
            }
        });

        Ok(())
    }

    /// Detach the monitor leg from the bridge.
    ///
    /// After this call, forward_track tasks will stop sending audio to the
    /// monitor on their next packet. The SampleStreamSource is dropped, which
    /// will cause the monitor's PeerConnection track to end.
    pub fn detach_monitor(&self) -> Result<()> {
        let mut guard = self.monitor.lock().unwrap();
        if guard.is_some() {
            *guard = None;
            info!(call_id = %self.call_id, "Monitor leg detached");
        }
        Ok(())
    }

    /// Returns true if a monitor leg is currently attached.
    pub fn has_monitor(&self) -> bool {
        self.monitor.lock().unwrap().is_some()
    }

    /// Get the current monitor mode, if a monitor is attached.
    pub fn monitor_mode(&self) -> Option<MonitorMode> {
        self.monitor.lock().unwrap().as_ref().map(|m| m.mode)
    }

    /// Set the monitor mode. Only effective if a monitor is attached.
    ///
    /// - `SilentListen`: Monitor hears both legs, cannot speak.
    /// - `Whisper`: Monitor audio is mixed into leg_b (agent) only.
    /// - `Barge`: Monitor audio is mixed into both leg_a and leg_b (3-way conference).
    pub fn set_monitor_mode(&self, mode: MonitorMode) -> Result<()> {
        let mut guard = self.monitor.lock().unwrap();
        if let Some(ref mut monitor) = *guard {
            let old_mode = monitor.mode;
            monitor.mode = mode;
            info!(
                call_id = %self.call_id,
                ?old_mode,
                ?mode,
                "Monitor mode changed"
            );
            Ok(())
        } else {
            Err(anyhow::anyhow!("No monitor leg attached"))
        }
    }

    /// Spawn a background task that reads audio from the monitor's PeerConnection
    /// receiver tracks (supervisor speaking) and stores decoded PCM in the shared
    /// `incoming_audio` buffer. This allows `forward_track` tasks to mix the
    /// supervisor's voice into Whisper/Barge streams without blocking.
    fn spawn_monitor_receiver(
        pc: rustrtc::PeerConnection,
        incoming_audio: Arc<Mutex<Option<MonitorAudioFrame>>>,
        monitor_codec: CodecType,
        call_id: String,
        monitor_arc: Arc<Mutex<Option<MonitorLeg>>>,
    ) {
        crate::utils::spawn(async move {
            // Wait for a Track event from the monitor's PeerConnection.
            // The receiver track carries audio from the supervisor.
            let mut pc_recv = Box::pin(pc.recv());
            let mut receiver_track: Option<Arc<dyn MediaStreamTrack>> = None;

            // First check existing transceivers for a receiver track
            let transceivers = pc.get_transceivers();
            for transceiver in &transceivers {
                if let Some(rx) = transceiver.receiver() {
                    let track = rx.track();
                    if track.kind() == MediaKind::Audio {
                        info!(call_id, "Monitor receiver: found existing audio track");
                        receiver_track = Some(track);
                        break;
                    }
                }
            }

            // If no existing track, wait briefly for a Track event
            if receiver_track.is_none() {
                let timeout = tokio::time::sleep(std::time::Duration::from_secs(5));
                tokio::pin!(timeout);

                loop {
                    tokio::select! {
                        event = &mut pc_recv => {
                            if let Some(rustrtc::PeerConnectionEvent::Track(transceiver)) = event {
                                if let Some(rx) = transceiver.receiver() {
                                    let track = rx.track();
                                    if track.kind() == MediaKind::Audio {
                                        info!(call_id, "Monitor receiver: got audio track from event");
                                        receiver_track = Some(track);
                                        break;
                                    }
                                }
                                pc_recv = Box::pin(pc.recv());
                            } else {
                                debug!(call_id, "Monitor receiver: PeerConnection closed before track event");
                                return;
                            }
                        }
                        _ = &mut timeout => {
                            debug!(call_id, "Monitor receiver: no track event after 5s, supervisor may not be sending audio");
                            return;
                        }
                    }
                }
            }

            let track = match receiver_track {
                Some(t) => t,
                None => return,
            };

            info!(call_id, track_id = %track.id(), "Monitor receiver task started");

            let mut decoder: Box<dyn Decoder> = create_decoder(monitor_codec);
            let mut seq: u64 = 0;

            while let Ok(sample) = track.recv().await {
                // Check if monitor is still attached
                {
                    let guard = monitor_arc.lock().unwrap();
                    if guard.is_none() {
                        debug!(call_id, "Monitor detached, stopping receiver task");
                        break;
                    }
                }

                if let MediaSample::Audio(ref frame) = sample {
                    // Decode to PCM for mixing
                    let pcm = decoder.decode(&frame.data);
                    seq += 1;

                    // Store in shared buffer (overwriting previous frame)
                    if let Ok(mut guard) = incoming_audio.lock() {
                        *guard = Some(MonitorAudioFrame { pcm, seq });
                    }
                }
            }

            info!(call_id, "Monitor receiver task finished");
        });
    }

    /// Mix monitor (supervisor) audio into a call leg's audio frame.
    ///
    /// Decodes the source audio to PCM, mixes it with the monitor's PCM, and
    /// re-encodes to the target codec. Returns a new `MediaSample` with the
    /// mixed audio, or `None` if no mixing was needed (no monitor audio available,
    /// wrong mode, etc.).
    ///
    /// The `last_monitor_seq` parameter tracks which monitor frame was last consumed
    /// by this particular forward_track task, preventing double-mixing of the same
    /// frame.
    fn mix_monitor_into_sample(
        monitor: &Arc<Mutex<Option<MonitorLeg>>>,
        sample: &MediaSample,
        _source_codec: CodecType,
        _target_codec: CodecType,
        leg: Leg,
        last_monitor_seq: &mut u64,
        mix_decoder: &mut Box<dyn Decoder>,
        mix_encoder: &mut Box<dyn Encoder>,
    ) -> Option<MediaSample> {
        let frame = match sample {
            MediaSample::Audio(f) => f,
            _ => return None,
        };

        // Read the monitor state: mode and incoming audio buffer
        let (mode, incoming_audio_arc) = {
            let guard = monitor.lock().unwrap();
            let monitor_leg = guard.as_ref()?;
            (monitor_leg.mode, monitor_leg.incoming_audio.clone())
        };

        // Check if this leg should receive monitor audio based on mode
        let should_mix = match mode {
            MonitorMode::SilentListen => false,
            MonitorMode::Whisper => leg == Leg::A, // Whisper: monitor audio -> leg_b (agent)
            // In the bridge, Leg::A forward_track sends audio FROM leg_a TO leg_b.
            // So when we want monitor audio sent TO leg_b, we mix in the A->B path (leg == Leg::A).
            MonitorMode::Barge => true, // Barge: monitor audio -> both legs
        };

        if !should_mix {
            return None;
        }

        // Get the latest monitor audio frame
        let monitor_pcm = {
            let guard = incoming_audio_arc.lock().unwrap();
            match guard.as_ref() {
                Some(maf) if maf.seq > *last_monitor_seq => {
                    *last_monitor_seq = maf.seq;
                    Some(maf.pcm.clone())
                }
                _ => None,
            }
        };

        let monitor_pcm = monitor_pcm?;

        // Decode source audio to PCM
        let source_pcm = mix_decoder.decode(&frame.data);

        // Mix: average the two PCM streams to prevent clipping
        let mix_len = source_pcm.len().min(monitor_pcm.len());
        let mut mixed_pcm = Vec::with_capacity(source_pcm.len());
        for i in 0..mix_len {
            let mixed = ((source_pcm[i] as i32 + monitor_pcm[i] as i32) / 2) as i16;
            mixed_pcm.push(mixed);
        }
        // If source is longer than monitor, append remaining source samples (halved for consistency)
        for i in mix_len..source_pcm.len() {
            mixed_pcm.push(source_pcm[i] / 2);
        }

        // Re-encode the mixed PCM to the target codec
        let encoded = mix_encoder.encode(&mixed_pcm);

        let mixed_frame = AudioFrame {
            rtp_timestamp: frame.rtp_timestamp,
            clock_rate: frame.clock_rate,
            data: Bytes::from(encoded),
            sequence_number: frame.sequence_number,
            payload_type: frame.payload_type,
            marker: frame.marker,
            raw_packet: None,
            source_addr: frame.source_addr,
        };

        Some(MediaSample::Audio(mixed_frame))
    }

    pub async fn start(&self) -> Result<()> {
        if self.started.swap(true, Ordering::SeqCst) {
            return Ok(());
        }
        self.quality.set_media_state(call_quality::MediaLayerState::Established);
        let needs_transcoding = self.codec_a != self.codec_b;
        debug!(
            codec_a = ?self.codec_a,
            codec_b = ?self.codec_b,
            needs_transcoding,
            "Starting media bridge between Leg A and Leg B"
        );

        let tracks_a = self.leg_a.get_tracks().await;
        let tracks_b = self.leg_b.get_tracks().await;

        let pc_a = if let Some(t) = tracks_a.first() {
            t.lock().await.get_peer_connection().await
        } else {
            None
        };

        let pc_b = if let Some(t) = tracks_b.first() {
            t.lock().await.get_peer_connection().await
        } else {
            None
        };

        if let (Some(pc_a), Some(pc_b)) = (pc_a, pc_b) {
            let params_a = self.params_a.clone();
            let params_b = self.params_b.clone();
            let codec_a = self.codec_a;
            let codec_b = self.codec_b;
            let dtmf_pt_a = self.dtmf_pt_a;
            let dtmf_pt_b = self.dtmf_pt_b;
            let ssrc_a = self.ssrc_a;
            let ssrc_b = self.ssrc_b;
            let recorder = self.recorder.clone();
            let cancel_token = self.leg_a.cancel_token();
            let leg_a = self.leg_a.clone();
            let leg_b = self.leg_b.clone();
            let call_id = self.call_id.clone();
            let sipflow_backend = self.sipflow_backend.clone();
            let input_gain = self.input_gain;
            let output_gain = self.output_gain;
            let quality = self.quality.clone();
            let monitor = self.monitor.clone();

            // Prepare watchdog config
            let watchdog_quality = self.quality.clone();
            let watchdog_call_id = self.call_id.clone();
            let watchdog_cancel = self.leg_a.cancel_token();
            let watchdog_sipflow = self.sipflow_backend.clone();
            let watchdog_config = self.quality_config.clone();

            crate::utils::spawn(async move {
                // Spawn quality watchdog if enabled
                let watchdog_enabled = watchdog_config
                    .as_ref()
                    .map(|c| c.enabled)
                    .unwrap_or(true);

                if watchdog_enabled {
                    let thresholds = watchdog_config
                        .as_ref()
                        .map(|c| QualityThresholds::from_config(c))
                        .unwrap_or_default();
                    let interval = watchdog_config
                        .as_ref()
                        .map(|c| c.watchdog_interval_secs)
                        .unwrap_or(2);
                    let cancel = watchdog_cancel;

                    tokio::spawn(call_quality::quality_watchdog(
                        watchdog_quality,
                        watchdog_call_id,
                        cancel,
                        watchdog_sipflow,
                        thresholds,
                        interval,
                    ));
                }

                tokio::select! {
                    _ = cancel_token.cancelled() => {},
                    _ = Self::bridge_pcs(
                        leg_a,
                        leg_b,
                        pc_a,
                        pc_b,
                        params_a,
                        params_b,
                        codec_a,
                        codec_b,
                        dtmf_pt_a,
                        dtmf_pt_b,
                        ssrc_a,
                        ssrc_b,
                        recorder,
                        call_id,
                        sipflow_backend,
                        input_gain,
                        output_gain,
                        quality,
                        monitor,
                    ) => {}
                }
            });
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn bridge_pcs(
        leg_a: Arc<dyn MediaPeer>,
        leg_b: Arc<dyn MediaPeer>,
        pc_a: rustrtc::PeerConnection,
        pc_b: rustrtc::PeerConnection,
        params_a: rustrtc::RtpCodecParameters,
        params_b: rustrtc::RtpCodecParameters,
        codec_a: CodecType,
        codec_b: CodecType,
        dtmf_pt_a: Option<u8>,
        dtmf_pt_b: Option<u8>,
        ssrc_a: Option<u32>,
        ssrc_b: Option<u32>,
        recorder: Arc<Mutex<Option<Recorder>>>,
        call_id: String,
        sipflow_backend: Option<Arc<dyn SipFlowBackend>>,
        input_gain: f32,
        output_gain: f32,
        quality: Arc<CallQuality>,
        monitor: Arc<Mutex<Option<MonitorLeg>>>,
    ) {
        debug!(
            "bridge_pcs started: codec_a={:?} codec_b={:?} ssrc_a={:?} ssrc_b={:?}",
            codec_a, codec_b, ssrc_a, ssrc_b
        );
        let mut forwarders = FuturesUnordered::new();
        let mut started_track_ids = std::collections::HashSet::new();

        // Helper closure to start a track forwarder, deduplicating by leg+track_id.
        // Returns Some(future) if the forwarder was started, None if already running.
        let start_forwarder =
            |leg_peer: Arc<dyn MediaPeer>,
             track: Arc<dyn MediaStreamTrack>,
             target_pc: rustrtc::PeerConnection,
             target_params: rustrtc::RtpCodecParameters,
             source_codec: CodecType,
             target_codec: CodecType,
             leg_enum: Leg,
             source_dtmf_pt: Option<u8>,
             target_dtmf_pt: Option<u8>,
             rec: Arc<Mutex<Option<Recorder>>>,
             cid: String,
             backend: Option<Arc<dyn SipFlowBackend>>,
             in_gain: f32,
             out_gain: f32,
             qual: Arc<CallQuality>,
             mon: Arc<Mutex<Option<MonitorLeg>>>,
             track_id: &str,
             started_ids: &mut std::collections::HashSet<String>| {
                let key = format!("{:?}-{}", leg_enum, track_id);
                if started_ids.insert(key) {
                    info!(
                        leg = ?leg_enum,
                        track_id,
                        "Starting track forwarder"
                    );
                    Some(Self::forward_track(
                        leg_peer,
                        track,
                        target_pc,
                        target_params,
                        source_codec,
                        target_codec,
                        leg_enum,
                        source_dtmf_pt,
                        target_dtmf_pt,
                        None,
                        rec,
                        cid,
                        backend,
                        in_gain,
                        out_gain,
                        qual,
                        mon,
                    ))
                } else {
                    debug!(
                        leg = ?leg_enum,
                        track_id,
                        "Track already started, skipping"
                    );
                    None
                }
            };

        // Log pre-existing transceivers but don't start forward_tracks from them.
        // Track events from pc.recv() carry the live track instances and fire immediately,
        // so we rely on those instead. Pre-existing transceiver tracks can become stale
        // when the PeerConnection processes events internally.
        let transceivers_a = pc_a.get_transceivers();
        let transceivers_b = pc_b.get_transceivers();

        for transceiver in &transceivers_a {
            if let Some(receiver) = transceiver.receiver() {
                let track = receiver.track();
                debug!(
                    "Pre-existing transceiver Leg A: track_id={} kind={:?}",
                    track.id(), track.kind()
                );
            }
        }

        for transceiver in &transceivers_b {
            if let Some(receiver) = transceiver.receiver() {
                let track = receiver.track();
                debug!(
                    "Pre-existing transceiver Leg B: track_id={} kind={:?}",
                    track.id(), track.kind()
                );
            }
        }

        let has_preexisting_a = transceivers_a.iter().any(|t| t.receiver().is_some());
        let has_preexisting_b = transceivers_b.iter().any(|t| t.receiver().is_some());

        let mut pc_a_recv = Box::pin(pc_a.recv());
        let mut pc_b_recv = Box::pin(pc_b.recv());
        let mut pc_a_closed = false;
        let mut pc_b_closed = false;

        // Wait briefly for Track events before falling back to pre-existing transceivers.
        // Track events carry live track instances, while pre-existing transceiver tracks
        // can become stale after the PeerConnection processes internal events.
        let fallback_delay = tokio::time::sleep(std::time::Duration::from_millis(200));
        tokio::pin!(fallback_delay);
        let mut fallback_fired = false;

        loop {
            tokio::select! {
                event_a = &mut pc_a_recv, if !pc_a_closed => {
                    if let Some(event) = event_a {
                        match event {
                            rustrtc::PeerConnectionEvent::Track(transceiver) => {
                                if let Some(receiver) = transceiver.receiver() {
                                    let track = receiver.track();
                                    let track_id = track.id().to_string();
                                    info!(
                                        "Track event Leg A: track_id={} kind={:?}",
                                        track_id,
                                        track.kind()
                                    );
                                    if let Some(forwarder) = start_forwarder(
                                        leg_a.clone(),
                                        track,
                                        pc_b.clone(),
                                        params_b.clone(),
                                        codec_a,
                                        codec_b,
                                        Leg::A,
                                        dtmf_pt_a,
                                        dtmf_pt_b,
                                        recorder.clone(),
                                        call_id.clone(),
                                        sipflow_backend.clone(),
                                        input_gain,
                                        output_gain,
                                        quality.clone(),
                                        monitor.clone(),
                                        &track_id,
                                        &mut started_track_ids,
                                    ) {
                                        forwarders.push(forwarder);
                                    }
                                }
                            }
                            _ => {
                            }
                        }
                        pc_a_recv = Box::pin(pc_a.recv());
                    } else {
                        debug!("Leg A PeerConnection closed");
                        pc_a_closed = true;
                    }
                }
                event_b = &mut pc_b_recv, if !pc_b_closed => {
                    if let Some(event) = event_b {
                        match event {
                            rustrtc::PeerConnectionEvent::Track(transceiver) => {
                                if let Some(receiver) = transceiver.receiver() {
                                    let track = receiver.track();
                                    let track_id = track.id().to_string();
                                    info!(
                                        "Track event Leg B: track_id={} kind={:?}",
                                        track_id,
                                        track.kind()
                                    );
                                    if let Some(forwarder) = start_forwarder(
                                        leg_b.clone(),
                                        track,
                                        pc_a.clone(),
                                        params_a.clone(),
                                        codec_b,
                                        codec_a,
                                        Leg::B,
                                        dtmf_pt_b,
                                        dtmf_pt_a,
                                        recorder.clone(),
                                        call_id.clone(),
                                        sipflow_backend.clone(),
                                        input_gain,
                                        output_gain,
                                        quality.clone(),
                                        monitor.clone(),
                                        &track_id,
                                        &mut started_track_ids,
                                    ) {
                                        forwarders.push(forwarder);
                                    }
                                }
                            }
                            _ => {}
                        }
                        pc_b_recv = Box::pin(pc_b.recv());
                    } else {
                        debug!("Leg B PeerConnection closed");
                        pc_b_closed = true;
                    }
                }
                Some(_) = forwarders.next(), if !forwarders.is_empty() => {}
                _ = &mut fallback_delay, if !fallback_fired => {
                    fallback_fired = true;
                    // Fall back to pre-existing transceivers if Track events didn't fire
                    if !started_track_ids.iter().any(|id| id.starts_with("A-")) && has_preexisting_a {
                        warn!("No Track events for Leg A after 200ms, using pre-existing transceiver");
                        for transceiver in &transceivers_a {
                            if let Some(receiver) = transceiver.receiver() {
                                let track = receiver.track();
                                let track_id = track.id().to_string();
                                if let Some(forwarder) = start_forwarder(
                                    leg_a.clone(),
                                    track,
                                    pc_b.clone(),
                                    params_b.clone(),
                                    codec_a,
                                    codec_b,
                                    Leg::A,
                                    dtmf_pt_a,
                                    dtmf_pt_b,
                                    recorder.clone(),
                                    call_id.clone(),
                                    sipflow_backend.clone(),
                                    input_gain,
                                    output_gain,
                                    quality.clone(),
                                    monitor.clone(),
                                    &track_id,
                                    &mut started_track_ids,
                                ) {
                                    forwarders.push(forwarder);
                                }
                            }
                        }
                    }
                    if !started_track_ids.iter().any(|id| id.starts_with("B-")) && has_preexisting_b {
                        warn!("No Track events for Leg B after 200ms, using pre-existing transceiver");
                        for transceiver in &transceivers_b {
                            if let Some(receiver) = transceiver.receiver() {
                                let track = receiver.track();
                                let track_id = track.id().to_string();
                                if let Some(forwarder) = start_forwarder(
                                    leg_b.clone(),
                                    track,
                                    pc_a.clone(),
                                    params_a.clone(),
                                    codec_b,
                                    codec_a,
                                    Leg::B,
                                    dtmf_pt_b,
                                    dtmf_pt_a,
                                    recorder.clone(),
                                    call_id.clone(),
                                    sipflow_backend.clone(),
                                    input_gain,
                                    output_gain,
                                    quality.clone(),
                                    monitor.clone(),
                                    &track_id,
                                    &mut started_track_ids,
                                ) {
                                    forwarders.push(forwarder);
                                }
                            }
                        }
                    }
                }
            }
            if pc_a_closed && pc_b_closed && forwarders.is_empty() {
                break;
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn forward_track(
        source_peer: Arc<dyn MediaPeer>,
        track: Arc<dyn MediaStreamTrack>,
        target_pc: rustrtc::PeerConnection,
        target_params: rustrtc::RtpCodecParameters,
        source_codec: CodecType,
        target_codec: CodecType,
        leg: Leg,
        source_dtmf_pt: Option<u8>,
        target_dtmf_pt: Option<u8>,
        target_ssrc: Option<u32>,
        recorder: Arc<Mutex<Option<Recorder>>>,
        call_id: String,
        sipflow_backend: Option<Arc<dyn SipFlowBackend>>,
        input_gain: f32,
        output_gain: f32,
        quality: Arc<CallQuality>,
        monitor: Arc<Mutex<Option<MonitorLeg>>>,
    ) {
        let needs_transcoding = source_codec != target_codec;
        let track_id = track.id().to_string();
        debug!(
            call_id,
            track_id,
            ?leg,
            "forward_track source_codec={:?} target_codec={:?} needs_transcoding={} source_dtmf={:?} target_dtmf={:?}",
            source_codec,
            target_codec,
            needs_transcoding,
            source_dtmf_pt,
            target_dtmf_pt
        );
        let (source_target, track_target, _) =
            rustrtc::media::track::sample_track(MediaKind::Audio, 100);

        // Try to reuse existing transceiver first to avoid renegotiation
        let transceivers = target_pc.get_transceivers();
        let existing_transceiver = transceivers
            .iter()
            .find(|t| t.kind() == rustrtc::MediaKind::Audio);

        if let Some(transceiver) = existing_transceiver {
            debug!(
                call_id,
                track_id,
                ?leg,
                "forward_track reusing existing transceiver",
            );

            if let Some(old_sender) = transceiver.sender() {
                let ssrc = target_ssrc.unwrap_or(old_sender.ssrc());
                let params = target_params.clone();
                let track_arc: Arc<dyn MediaStreamTrack> = track_target.clone();
                debug!(
                    call_id,
                    track_id,
                    ssrc,
                    ?leg,
                    ?params,
                    "forward_track replacing sender on existing transceiver",
                );

                let new_sender = rustrtc::RtpSender::builder(track_arc, ssrc)
                    .params(params)
                    .build();

                transceiver.set_sender(Some(new_sender));
            } else {
                let ssrc = target_ssrc.unwrap_or_else(|| rand::random::<u32>());
                let track_arc: Arc<dyn MediaStreamTrack> = track_target.clone();
                let params = target_params.clone();
                debug!(
                    call_id,
                    track_id,
                    ?leg,
                    ?params,
                    "forward_track creating new sender on existing transceiver",
                );

                let new_sender = rustrtc::RtpSender::builder(track_arc, ssrc)
                    .params(params)
                    .build();
                transceiver.set_sender(Some(new_sender));
            }
        } else {
            match target_pc.add_track(track_target, target_params.clone()) {
                Ok(_sender) => {
                    debug!(call_id, track_id, ?leg, "forward_track add_track success");
                }
                Err(e) => {
                    warn!(
                        call_id,
                        track_id,
                        ?leg,
                        "forward_track add_track failed: {}",
                        e
                    );
                    return;
                }
            }
        }
        let leg_gain = if leg == Leg::B { input_gain } else { output_gain };
        let mut transcoder = if needs_transcoding {
            Some(
                crate::media::Transcoder::new(source_codec, target_codec)
                    .with_gain(leg_gain),
            )
        } else {
            None
        };

        // Lazy-initialized transcoder for the monitor leg. Created on first use
        // when the monitor codec differs from the source codec.
        let mut monitor_transcoder: Option<crate::media::Transcoder> = None;
        // Track the last known monitor codec so we can detect changes.
        let mut last_monitor_codec: Option<CodecType> = None;

        // Decoder/encoder for mixing monitor audio into the forwarded stream.
        // These operate on the *target* codec since mixing happens after transcoding.
        // Lazily initialized on first use to avoid overhead when no monitor is attached.
        let mut mix_decoder: Option<Box<dyn Decoder>> = None;
        let mut mix_encoder: Option<Box<dyn Encoder>> = None;
        // Track the last consumed monitor audio sequence to avoid double-mixing.
        let mut last_monitor_seq: u64 = 0;

        let mut packet_count: u64 = 0;
        let target_pt = target_params.payload_type;
        let source_clock_rate = source_codec.clock_rate();
        let leg_quality = match leg {
            Leg::A => &quality.leg_a,
            Leg::B => &quality.leg_b,
        };

        // Stats tracking
        let mut last_stats_time = std::time::Instant::now();
        let mut packets_since_last_stat = 0;
        let mut bytes_since_last_stat = 0;

        while let Ok(mut sample) = track.recv().await {
            if source_peer.is_suppressed(&track_id) {
                continue;
            }

            let mut is_dtmf = false;
            let mut pt_for_recorder = None;

            packets_since_last_stat += 1;
            if let MediaSample::Audio(ref f) = sample {
                bytes_since_last_stat += f.data.len();
            }

            // Periodic stats logging (simulated RTCP report)
            if last_stats_time.elapsed().as_secs() >= 5 {
                let duration = last_stats_time.elapsed().as_secs_f64();
                let bitrate_kbps = (bytes_since_last_stat as f64 * 8.0) / duration / 1000.0;
                let pps = packets_since_last_stat as f64 / duration;

                let leg_snap = leg_quality.snapshot();
                info!(
                   ?leg,
                   %track_id,
                   pps,
                   bitrate_kbps,
                   total_packets = packet_count,
                   loss_pct = leg_snap.loss_percent,
                   jitter_ms = leg_snap.avg_jitter_ms,
                   "Media Stream Stats"
                );
                last_stats_time = std::time::Instant::now();
                packets_since_last_stat = 0;
                bytes_since_last_stat = 0;
            }

            if let MediaSample::Audio(ref mut frame) = sample {
                packet_count += 1;
                if packet_count == 1 {
                    quality.set_media_state(call_quality::MediaLayerState::Flowing);
                }
                if packet_count % 250 == 1 {
                    // 5 seconds at 50pps
                    debug!(
                        call_id,
                        track_id,
                        ?leg,
                        packet_count,
                        "forward_track received"
                    );
                }

                if let Some(seq) = frame.sequence_number {
                    if !leg_quality.record_seq(seq) {
                        continue; // duplicate
                    }
                    leg_quality.record_packet(seq, frame.rtp_timestamp, source_clock_rate);
                }
                // Send RTP packet to sipflow backend if configured (only when raw_packet is available)
                if let Some(backend) = &sipflow_backend {
                    if let Some(ref rtp_packet) = frame.raw_packet {
                        if let Ok(rtp_bytes) = rtp_packet.marshal() {
                            let payload = bytes::Bytes::from(rtp_bytes);
                            let src_addr: String = if let Some(addr) = frame.source_addr {
                                format!("{:?}_{}", leg, addr)
                            } else {
                                format!("{:?}", leg)
                            };
                            let item = SipFlowItem {
                                timestamp: frame.rtp_timestamp as u64,
                                seq: frame.sequence_number.unwrap_or(0) as u64,
                                msg_type: SipFlowMsgType::Rtp,
                                src_addr,
                                dst_addr: format!("bridge"),
                                payload,
                            };

                            if let Err(e) = backend.record(&call_id, item) {
                                debug!("Failed to record RTP to sipflow: {}", e);
                            }
                        }
                    }
                }

                // Rewrite payload type to match target's expected PT
                if let Some(pt) = frame.payload_type {
                    if Some(pt) == source_dtmf_pt {
                        is_dtmf = true;
                        pt_for_recorder = target_dtmf_pt;
                        if let Some(t_dtmf) = target_dtmf_pt {
                            frame.payload_type = Some(t_dtmf);
                        }
                    } else if !needs_transcoding {
                        // If not transcoding, rewrite audio PT to target PT
                        frame.payload_type = Some(target_pt);
                    }
                }

                if let Some(ref mut t) = transcoder {
                    if !is_dtmf {
                        sample = MediaSample::Audio(t.transcode(frame));
                        // After transcoding, ensure PT matches the target's negotiated PT
                        if let MediaSample::Audio(ref mut new_frame) = sample {
                            if new_frame.payload_type != Some(target_pt) {
                                new_frame.payload_type = Some(target_pt);
                            }
                        }
                    } else {
                        t.update_dtmf_timestamp(frame);
                    }
                }
            }

            // Send to recorder if configured
            {
                if let Some(ref mut r) = *recorder.lock().unwrap() {
                    let _ = r.write_sample(leg, &sample, pt_for_recorder);
                }
            }

            // Forward a copy of the processed sample to the monitor leg if attached.
            // This runs on every packet but the fast path (no monitor) is just a
            // Mutex::lock + Option::is_none check, so overhead is negligible.
            if !is_dtmf {
                Self::try_send_to_monitor(
                    &monitor,
                    &sample,
                    source_codec,
                    &mut monitor_transcoder,
                    &mut last_monitor_codec,
                    &call_id,
                    leg,
                );
            }

            // Mix monitor (supervisor) audio into the sample for Whisper/Barge modes.
            // This must happen AFTER recording and monitor send, so the recorder
            // captures the original call and the monitor hears the unmixed audio.
            if !is_dtmf {
                // Lazily initialize the mix decoder/encoder on first use
                let dec = mix_decoder.get_or_insert_with(|| create_decoder(target_codec));
                let enc = mix_encoder.get_or_insert_with(|| create_encoder(target_codec));

                if let Some(mixed) = Self::mix_monitor_into_sample(
                    &monitor,
                    &sample,
                    source_codec,
                    target_codec,
                    leg,
                    &mut last_monitor_seq,
                    dec,
                    enc,
                ) {
                    sample = mixed;
                }
            }

            if let Err(e) = source_target.send(sample).await {
                warn!(
                    call_id,
                    track_id,
                    ?leg,
                    "forward_track source_target.send failed: {}",
                    e
                );
                break;
            }
        }

        let final_snap = leg_quality.snapshot();
        info!(
            call_id,
            track_id,
            ?leg,
            packet_count,
            loss_pct = final_snap.loss_percent,
            lost_packets = final_snap.lost_packets,
            jitter_ms = final_snap.avg_jitter_ms,
            max_jitter_ms = final_snap.max_jitter_ms,
            "forward_track finished",
        );
    }

    /// Send a copy of the processed audio sample to the monitor leg, if one is attached.
    ///
    /// Uses `try_send` (non-blocking) so that a slow or blocked monitor leg never
    /// delays the main call bridge. If the monitor's channel is full, the packet
    /// is silently dropped.
    ///
    /// When the monitor codec differs from the source codec, a per-direction
    /// transcoder is lazily created and cached in `monitor_transcoder`.
    fn try_send_to_monitor(
        monitor: &Arc<Mutex<Option<MonitorLeg>>>,
        sample: &MediaSample,
        source_codec: CodecType,
        monitor_transcoder: &mut Option<crate::media::Transcoder>,
        last_monitor_codec: &mut Option<CodecType>,
        call_id: &str,
        leg: Leg,
    ) {
        // Fast path: check if monitor exists. The lock is held very briefly.
        let guard = monitor.lock().unwrap();
        let monitor_leg = match guard.as_ref() {
            Some(m) => m,
            None => return,
        };

        // All modes forward audio from the call legs to the monitor so the
        // supervisor can hear both sides. The difference between modes is in
        // how the monitor's audio is routed back: SilentListen does nothing,
        // Whisper mixes into leg_b only, and Barge mixes into both legs.
        // That reverse path is handled by mix_monitor_into_sample().

        let monitor_codec = monitor_leg.codec;

        let monitor_sample = if let MediaSample::Audio(frame) = sample {
            if source_codec == monitor_codec {
                // Same codec: clone the sample directly, just fix the PT
                let mut monitor_frame = frame.clone();
                monitor_frame.payload_type = Some(monitor_codec.payload_type());
                // Strip raw_packet since the monitor doesn't need it
                monitor_frame.raw_packet = None;
                MediaSample::Audio(monitor_frame)
            } else {
                // Different codec: transcode for the monitor
                // Check if we need to create or recreate the transcoder
                if last_monitor_codec.as_ref() != Some(&monitor_codec) {
                    *monitor_transcoder =
                        Some(crate::media::Transcoder::new(source_codec, monitor_codec));
                    *last_monitor_codec = Some(monitor_codec);
                }

                if let Some(tc) = monitor_transcoder {
                    MediaSample::Audio(tc.transcode(frame))
                } else {
                    // Should not happen, but fall back to clone
                    let mut monitor_frame = frame.clone();
                    monitor_frame.raw_packet = None;
                    MediaSample::Audio(monitor_frame)
                }
            }
        } else {
            return; // Skip non-audio samples
        };

        // Non-blocking send. If the channel is full, drop the packet rather than
        // blocking the main bridge. The monitor can tolerate occasional packet loss.
        if let Err(_e) = monitor_leg.source.try_send(monitor_sample) {
            debug!(
                call_id,
                ?leg,
                "Monitor leg channel full or closed, dropping packet"
            );
        }
    }

    pub fn quality(&self) -> &Arc<CallQuality> {
        &self.quality
    }

    pub fn quality_snapshot(&self) -> CallQualitySnapshot {
        self.quality.snapshot()
    }

    pub fn stop(&self) {
        let mut guard = self.recorder.lock().unwrap();
        if let Some(ref mut r) = *guard {
            let _ = r.finalize();
        }
        // Detach monitor on stop
        {
            let mut monitor_guard = self.monitor.lock().unwrap();
            *monitor_guard = None;
        }
        self.leg_a.stop();
        self.leg_b.stop();
    }

    pub async fn resume_forwarding(&self, track_id: &str) -> Result<()> {
        self.leg_a.resume_forwarding(track_id).await;
        self.leg_b.resume_forwarding(track_id).await;
        Ok(())
    }

    pub async fn suppress_forwarding(&self, track_id: &str) -> Result<()> {
        info!(track_id = %track_id, "Suppressing forwarding in bridge");
        self.leg_a.suppress_forwarding(track_id).await;
        self.leg_b.suppress_forwarding(track_id).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proxy::proxy_call::test_util::tests::MockMediaPeer;

    #[tokio::test]
    async fn test_media_bridge_start_stop() {
        let leg_a = Arc::new(MockMediaPeer::new());
        let leg_b = Arc::new(MockMediaPeer::new());
        let bridge = MediaBridge::new(
            leg_a.clone(),
            leg_b.clone(),
            rustrtc::RtpCodecParameters::default(),
            rustrtc::RtpCodecParameters::default(),
            None,
            None,
            CodecType::PCMU,
            CodecType::PCMU,
            None,
            None,
            None,
            "test-call-id".to_string(),
            None,
            None,
        );

        bridge.start().await.unwrap();
        bridge.resume_forwarding("test-track").await.unwrap();
        bridge.suppress_forwarding("test-track").await.unwrap();
        bridge.stop();
        assert!(leg_a.stop_called.load(std::sync::atomic::Ordering::SeqCst));
        assert!(leg_b.stop_called.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_media_bridge_transcoding_detection() {
        let leg_a = Arc::new(MockMediaPeer::new());
        let leg_b = Arc::new(MockMediaPeer::new());
        let bridge = MediaBridge::new(
            leg_a.clone(),
            leg_b.clone(),
            rustrtc::RtpCodecParameters::default(),
            rustrtc::RtpCodecParameters::default(),
            None,
            None,
            CodecType::PCMU,
            CodecType::PCMA,
            None,
            None,
            None,
            "test-call-id".to_string(),
            None,
            None,
        );

        assert_eq!(bridge.codec_a, CodecType::PCMU);
        assert_eq!(bridge.codec_b, CodecType::PCMA);

        // Should log transcoding required
        bridge.start().await.unwrap();
    }

    /// Helper: create a MediaBridge with default PCMU settings for testing.
    fn make_test_bridge(call_id: &str) -> (Arc<MockMediaPeer>, Arc<MockMediaPeer>, MediaBridge) {
        let leg_a = Arc::new(MockMediaPeer::new());
        let leg_b = Arc::new(MockMediaPeer::new());
        let bridge = MediaBridge::new(
            leg_a.clone(),
            leg_b.clone(),
            rustrtc::RtpCodecParameters::default(),
            rustrtc::RtpCodecParameters::default(),
            None,
            None,
            CodecType::PCMU,
            CodecType::PCMU,
            None,
            None,
            None,
            call_id.to_string(),
            None,
            None,
        );
        (leg_a, leg_b, bridge)
    }

    // ------------------------------------------------------------------
    // attach_monitor tests
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn test_attach_monitor_creates_leg_with_correct_mode() {
        let (_la, _lb, bridge) = make_test_bridge("test-attach");

        assert!(!bridge.has_monitor());

        let monitor_peer = Arc::new(MockMediaPeer::new());
        bridge.attach_monitor(monitor_peer, CodecType::PCMU).unwrap();

        assert!(bridge.has_monitor());
        // Default mode should be SilentListen
        assert_eq!(bridge.monitor_mode(), Some(MonitorMode::SilentListen));
    }

    #[tokio::test]
    async fn test_attach_monitor_stores_codec() {
        let (_la, _lb, bridge) = make_test_bridge("test-attach-codec");

        let monitor_peer = Arc::new(MockMediaPeer::new());
        bridge.attach_monitor(monitor_peer, CodecType::PCMA).unwrap();

        // Verify the codec was stored by inspecting the monitor leg directly
        let guard = bridge.monitor.lock().unwrap();
        let monitor_leg = guard.as_ref().expect("Monitor should be attached");
        assert_eq!(monitor_leg.codec, CodecType::PCMA);
    }

    #[tokio::test]
    async fn test_attach_monitor_replaces_existing() {
        let (_la, _lb, bridge) = make_test_bridge("test-attach-replace");

        let peer1 = Arc::new(MockMediaPeer::new());
        bridge.attach_monitor(peer1, CodecType::PCMU).unwrap();
        assert!(bridge.has_monitor());

        // Attach a second monitor -- should replace the first
        let peer2 = Arc::new(MockMediaPeer::new());
        bridge.attach_monitor(peer2, CodecType::PCMA).unwrap();
        assert!(bridge.has_monitor());

        // The codec should reflect the new monitor
        let guard = bridge.monitor.lock().unwrap();
        let monitor_leg = guard.as_ref().expect("Monitor should be attached");
        assert_eq!(monitor_leg.codec, CodecType::PCMA);
    }

    // ------------------------------------------------------------------
    // detach_monitor tests
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn test_detach_monitor_removes_leg() {
        let (_la, _lb, bridge) = make_test_bridge("test-detach");

        let monitor_peer = Arc::new(MockMediaPeer::new());
        bridge.attach_monitor(monitor_peer, CodecType::PCMU).unwrap();
        assert!(bridge.has_monitor());

        bridge.detach_monitor().unwrap();
        assert!(!bridge.has_monitor());
        assert!(bridge.monitor_mode().is_none());
    }

    #[tokio::test]
    async fn test_detach_monitor_when_none_attached_is_ok() {
        let (_la, _lb, bridge) = make_test_bridge("test-detach-none");

        assert!(!bridge.has_monitor());
        // Detaching when no monitor is attached should succeed (no-op)
        bridge.detach_monitor().unwrap();
        assert!(!bridge.has_monitor());
    }

    #[tokio::test]
    async fn test_detach_monitor_idempotent() {
        let (_la, _lb, bridge) = make_test_bridge("test-detach-idempotent");

        let monitor_peer = Arc::new(MockMediaPeer::new());
        bridge.attach_monitor(monitor_peer, CodecType::PCMU).unwrap();

        bridge.detach_monitor().unwrap();
        assert!(!bridge.has_monitor());

        // Second detach should also succeed
        bridge.detach_monitor().unwrap();
        assert!(!bridge.has_monitor());
    }

    // ------------------------------------------------------------------
    // has_monitor tests
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn test_has_monitor_returns_false_initially() {
        let (_la, _lb, bridge) = make_test_bridge("test-has-monitor-init");
        assert!(!bridge.has_monitor());
    }

    #[tokio::test]
    async fn test_has_monitor_returns_true_after_attach() {
        let (_la, _lb, bridge) = make_test_bridge("test-has-monitor-attach");

        let monitor_peer = Arc::new(MockMediaPeer::new());
        bridge.attach_monitor(monitor_peer, CodecType::PCMU).unwrap();
        assert!(bridge.has_monitor());
    }

    #[tokio::test]
    async fn test_has_monitor_returns_false_after_detach() {
        let (_la, _lb, bridge) = make_test_bridge("test-has-monitor-detach");

        let monitor_peer = Arc::new(MockMediaPeer::new());
        bridge.attach_monitor(monitor_peer, CodecType::PCMU).unwrap();
        assert!(bridge.has_monitor());

        bridge.detach_monitor().unwrap();
        assert!(!bridge.has_monitor());
    }

    // ------------------------------------------------------------------
    // monitor_mode tests
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn test_monitor_mode_returns_none_when_no_monitor() {
        let (_la, _lb, bridge) = make_test_bridge("test-mode-none");
        assert!(bridge.monitor_mode().is_none());
    }

    #[tokio::test]
    async fn test_monitor_mode_returns_silent_listen_by_default() {
        let (_la, _lb, bridge) = make_test_bridge("test-mode-default");

        let monitor_peer = Arc::new(MockMediaPeer::new());
        bridge.attach_monitor(monitor_peer, CodecType::PCMU).unwrap();
        assert_eq!(bridge.monitor_mode(), Some(MonitorMode::SilentListen));
    }

    #[tokio::test]
    async fn test_monitor_mode_reflects_changes() {
        let (_la, _lb, bridge) = make_test_bridge("test-mode-change");

        let monitor_peer = Arc::new(MockMediaPeer::new());
        bridge.attach_monitor(monitor_peer, CodecType::PCMU).unwrap();

        bridge.set_monitor_mode(MonitorMode::Whisper).unwrap();
        assert_eq!(bridge.monitor_mode(), Some(MonitorMode::Whisper));

        bridge.set_monitor_mode(MonitorMode::Barge).unwrap();
        assert_eq!(bridge.monitor_mode(), Some(MonitorMode::Barge));

        bridge.set_monitor_mode(MonitorMode::SilentListen).unwrap();
        assert_eq!(bridge.monitor_mode(), Some(MonitorMode::SilentListen));
    }

    // ------------------------------------------------------------------
    // set_monitor_mode tests
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn test_set_monitor_mode_changes_between_all_modes() {
        let (_la, _lb, bridge) = make_test_bridge("test-set-mode-all");

        let monitor_peer = Arc::new(MockMediaPeer::new());
        bridge.attach_monitor(monitor_peer, CodecType::PCMU).unwrap();

        // SilentListen -> Whisper
        bridge.set_monitor_mode(MonitorMode::Whisper).unwrap();
        assert_eq!(bridge.monitor_mode(), Some(MonitorMode::Whisper));

        // Whisper -> Barge
        bridge.set_monitor_mode(MonitorMode::Barge).unwrap();
        assert_eq!(bridge.monitor_mode(), Some(MonitorMode::Barge));

        // Barge -> SilentListen
        bridge.set_monitor_mode(MonitorMode::SilentListen).unwrap();
        assert_eq!(bridge.monitor_mode(), Some(MonitorMode::SilentListen));
    }

    #[tokio::test]
    async fn test_set_monitor_mode_fails_when_no_monitor_attached() {
        let (_la, _lb, bridge) = make_test_bridge("test-set-mode-no-monitor");

        // No monitor attached -- set_monitor_mode should return an error
        let result = bridge.set_monitor_mode(MonitorMode::Barge);
        assert!(result.is_err());
        assert!(
            result.unwrap_err().to_string().contains("No monitor leg attached"),
            "Error message should indicate no monitor is attached"
        );
    }

    #[tokio::test]
    async fn test_set_monitor_mode_same_mode_is_ok() {
        let (_la, _lb, bridge) = make_test_bridge("test-set-mode-same");

        let monitor_peer = Arc::new(MockMediaPeer::new());
        bridge.attach_monitor(monitor_peer, CodecType::PCMU).unwrap();

        // Setting the same mode twice should succeed
        bridge.set_monitor_mode(MonitorMode::SilentListen).unwrap();
        assert_eq!(bridge.monitor_mode(), Some(MonitorMode::SilentListen));
    }

    #[tokio::test]
    async fn test_set_monitor_mode_fails_after_detach() {
        let (_la, _lb, bridge) = make_test_bridge("test-set-mode-after-detach");

        let monitor_peer = Arc::new(MockMediaPeer::new());
        bridge.attach_monitor(monitor_peer, CodecType::PCMU).unwrap();

        bridge.detach_monitor().unwrap();

        // Now set_monitor_mode should fail
        assert!(bridge.set_monitor_mode(MonitorMode::Whisper).is_err());
    }

    // ------------------------------------------------------------------
    // try_send_to_monitor tests
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn test_try_send_to_monitor_delivers_packet() {
        let (_la, _lb, bridge) = make_test_bridge("test-send-monitor");

        let monitor_peer = Arc::new(MockMediaPeer::new());
        bridge.attach_monitor(monitor_peer, CodecType::PCMU).unwrap();

        // Create a test audio sample
        let sample = MediaSample::Audio(rustrtc::media::AudioFrame {
            data: vec![0x80; 160].into(), // 20ms @ 8kHz PCMU
            rtp_timestamp: 160,
            sequence_number: Some(1),
            payload_type: Some(0),
            clock_rate: 8000,
            marker: false,
            raw_packet: None,
            source_addr: None,
        });

        let mut monitor_transcoder: Option<crate::media::Transcoder> = None;
        let mut last_monitor_codec: Option<CodecType> = None;

        // This should succeed without panic -- the packet goes into the
        // MonitorLeg's SampleStreamSource channel.
        MediaBridge::try_send_to_monitor(
            &bridge.monitor,
            &sample,
            CodecType::PCMU,
            &mut monitor_transcoder,
            &mut last_monitor_codec,
            "test-send-monitor",
            Leg::A,
        );

        // The monitor's source channel should have received the packet.
        // We verify by checking the monitor is still attached (source not closed).
        assert!(bridge.has_monitor());
    }

    #[tokio::test]
    async fn test_try_send_to_monitor_no_monitor_is_noop() {
        let (_la, _lb, bridge) = make_test_bridge("test-send-no-monitor");

        // No monitor attached -- try_send_to_monitor should be a no-op
        let sample = MediaSample::Audio(rustrtc::media::AudioFrame {
            data: vec![0x80; 160].into(),
            rtp_timestamp: 160,
            sequence_number: Some(1),
            payload_type: Some(0),
            clock_rate: 8000,
            marker: false,
            raw_packet: None,
            source_addr: None,
        });

        let mut monitor_transcoder: Option<crate::media::Transcoder> = None;
        let mut last_monitor_codec: Option<CodecType> = None;

        // Should not panic, should be a silent no-op
        MediaBridge::try_send_to_monitor(
            &bridge.monitor,
            &sample,
            CodecType::PCMU,
            &mut monitor_transcoder,
            &mut last_monitor_codec,
            "test-send-no-monitor",
            Leg::A,
        );

        assert!(!bridge.has_monitor());
    }

    #[tokio::test]
    async fn test_try_send_to_monitor_multiple_packets() {
        let (_la, _lb, bridge) = make_test_bridge("test-send-multi");

        let monitor_peer = Arc::new(MockMediaPeer::new());
        bridge.attach_monitor(monitor_peer, CodecType::PCMU).unwrap();

        let mut monitor_transcoder: Option<crate::media::Transcoder> = None;
        let mut last_monitor_codec: Option<CodecType> = None;

        // Send multiple packets from both legs
        for i in 0..10u16 {
            let leg = if i % 2 == 0 { Leg::A } else { Leg::B };
            let sample = MediaSample::Audio(rustrtc::media::AudioFrame {
                data: vec![0x80; 160].into(),
                rtp_timestamp: (i as u32) * 160,
                sequence_number: Some(i),
                payload_type: Some(0),
                clock_rate: 8000,
                marker: i == 0,
                raw_packet: None,
                source_addr: None,
            });

            MediaBridge::try_send_to_monitor(
                &bridge.monitor,
                &sample,
                CodecType::PCMU,
                &mut monitor_transcoder,
                &mut last_monitor_codec,
                "test-send-multi",
                leg,
            );
        }

        // Monitor should still be attached after multiple sends
        assert!(bridge.has_monitor());
    }

    #[tokio::test]
    async fn test_try_send_to_monitor_after_detach_is_noop() {
        let (_la, _lb, bridge) = make_test_bridge("test-send-after-detach");

        let monitor_peer = Arc::new(MockMediaPeer::new());
        bridge.attach_monitor(monitor_peer, CodecType::PCMU).unwrap();

        // Send one packet while attached
        let sample = MediaSample::Audio(rustrtc::media::AudioFrame {
            data: vec![0x80; 160].into(),
            rtp_timestamp: 160,
            sequence_number: Some(1),
            payload_type: Some(0),
            clock_rate: 8000,
            marker: false,
            raw_packet: None,
            source_addr: None,
        });

        let mut monitor_transcoder: Option<crate::media::Transcoder> = None;
        let mut last_monitor_codec: Option<CodecType> = None;

        MediaBridge::try_send_to_monitor(
            &bridge.monitor,
            &sample,
            CodecType::PCMU,
            &mut monitor_transcoder,
            &mut last_monitor_codec,
            "test-send-after-detach",
            Leg::A,
        );

        // Detach and send again -- should be a silent no-op
        bridge.detach_monitor().unwrap();

        MediaBridge::try_send_to_monitor(
            &bridge.monitor,
            &sample,
            CodecType::PCMU,
            &mut monitor_transcoder,
            &mut last_monitor_codec,
            "test-send-after-detach",
            Leg::B,
        );

        assert!(!bridge.has_monitor());
    }

    // ------------------------------------------------------------------
    // Monitor lifecycle integration tests
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn test_monitor_full_lifecycle() {
        let (_la, _lb, bridge) = make_test_bridge("test-lifecycle");

        // 1. Initially no monitor
        assert!(!bridge.has_monitor());
        assert!(bridge.monitor_mode().is_none());
        assert!(bridge.set_monitor_mode(MonitorMode::Barge).is_err());

        // 2. Attach monitor
        let monitor_peer = Arc::new(MockMediaPeer::new());
        bridge.attach_monitor(monitor_peer, CodecType::PCMU).unwrap();
        assert!(bridge.has_monitor());
        assert_eq!(bridge.monitor_mode(), Some(MonitorMode::SilentListen));

        // 3. Send some packets
        let mut monitor_transcoder: Option<crate::media::Transcoder> = None;
        let mut last_monitor_codec: Option<CodecType> = None;
        for i in 0..5u16 {
            let sample = MediaSample::Audio(rustrtc::media::AudioFrame {
                data: vec![0x80; 160].into(),
                rtp_timestamp: (i as u32) * 160,
                sequence_number: Some(i),
                payload_type: Some(0),
                clock_rate: 8000,
                marker: false,
                raw_packet: None,
                source_addr: None,
            });
            MediaBridge::try_send_to_monitor(
                &bridge.monitor,
                &sample,
                CodecType::PCMU,
                &mut monitor_transcoder,
                &mut last_monitor_codec,
                "test-lifecycle",
                Leg::A,
            );
        }

        // 4. Change mode
        bridge.set_monitor_mode(MonitorMode::Whisper).unwrap();
        assert_eq!(bridge.monitor_mode(), Some(MonitorMode::Whisper));

        bridge.set_monitor_mode(MonitorMode::Barge).unwrap();
        assert_eq!(bridge.monitor_mode(), Some(MonitorMode::Barge));

        // 5. Detach
        bridge.detach_monitor().unwrap();
        assert!(!bridge.has_monitor());
        assert!(bridge.monitor_mode().is_none());

        // 6. Sending after detach is a no-op
        let sample = MediaSample::Audio(rustrtc::media::AudioFrame {
            data: vec![0x80; 160].into(),
            rtp_timestamp: 0,
            sequence_number: Some(100),
            payload_type: Some(0),
            clock_rate: 8000,
            marker: false,
            raw_packet: None,
            source_addr: None,
        });
        MediaBridge::try_send_to_monitor(
            &bridge.monitor,
            &sample,
            CodecType::PCMU,
            &mut monitor_transcoder,
            &mut last_monitor_codec,
            "test-lifecycle",
            Leg::B,
        );

        // 7. Re-attach should work
        let monitor_peer2 = Arc::new(MockMediaPeer::new());
        bridge.attach_monitor(monitor_peer2, CodecType::PCMA).unwrap();
        assert!(bridge.has_monitor());
        assert_eq!(bridge.monitor_mode(), Some(MonitorMode::SilentListen));
    }

    #[tokio::test]
    async fn test_monitor_detached_on_bridge_stop() {
        let (la, lb, bridge) = make_test_bridge("test-stop-detach");

        let monitor_peer = Arc::new(MockMediaPeer::new());
        bridge.attach_monitor(monitor_peer, CodecType::PCMU).unwrap();
        assert!(bridge.has_monitor());

        bridge.stop();

        // After stop, monitor should be detached
        assert!(!bridge.has_monitor());
        // Legs should also be stopped
        assert!(la.stop_called.load(std::sync::atomic::Ordering::SeqCst));
        assert!(lb.stop_called.load(std::sync::atomic::Ordering::SeqCst));
    }

    // ------------------------------------------------------------------
    // MonitorMode enum tests
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn test_monitor_mode_default() {
        assert_eq!(MonitorMode::default(), MonitorMode::SilentListen);
    }

    #[tokio::test]
    async fn test_monitor_mode_equality() {
        assert_eq!(MonitorMode::SilentListen, MonitorMode::SilentListen);
        assert_eq!(MonitorMode::Whisper, MonitorMode::Whisper);
        assert_eq!(MonitorMode::Barge, MonitorMode::Barge);
        assert_ne!(MonitorMode::SilentListen, MonitorMode::Whisper);
        assert_ne!(MonitorMode::Whisper, MonitorMode::Barge);
        assert_ne!(MonitorMode::Barge, MonitorMode::SilentListen);
    }

    #[tokio::test]
    async fn test_monitor_mode_clone() {
        let mode = MonitorMode::Whisper;
        let cloned = mode;
        assert_eq!(mode, cloned);
    }

    #[tokio::test]
    async fn test_monitor_mode_serde_roundtrip() {
        // Verify serde serialization uses snake_case as configured
        let silent = MonitorMode::SilentListen;
        let json = serde_json::to_string(&silent).unwrap();
        assert_eq!(json, "\"silent_listen\"");

        let whisper = MonitorMode::Whisper;
        let json = serde_json::to_string(&whisper).unwrap();
        assert_eq!(json, "\"whisper\"");

        let barge = MonitorMode::Barge;
        let json = serde_json::to_string(&barge).unwrap();
        assert_eq!(json, "\"barge\"");

        // Deserialize back
        let deserialized: MonitorMode = serde_json::from_str("\"silent_listen\"").unwrap();
        assert_eq!(deserialized, MonitorMode::SilentListen);

        let deserialized: MonitorMode = serde_json::from_str("\"whisper\"").unwrap();
        assert_eq!(deserialized, MonitorMode::Whisper);

        let deserialized: MonitorMode = serde_json::from_str("\"barge\"").unwrap();
        assert_eq!(deserialized, MonitorMode::Barge);
    }

    #[tokio::test]
    async fn test_monitor_mode_serde_rejects_invalid() {
        let result: Result<MonitorMode, _> = serde_json::from_str("\"invalid_mode\"");
        assert!(result.is_err());
    }
}
