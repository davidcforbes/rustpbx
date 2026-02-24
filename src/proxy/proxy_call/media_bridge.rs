use crate::config::QualityConfig;
use crate::media::call_quality::{self, CallQuality, CallQualitySnapshot, QualityThresholds};
use crate::media::recorder::{Leg, Recorder, RecorderOption};
use crate::proxy::proxy_call::media_peer::MediaPeer;
use crate::sipflow::{SipFlowBackend, SipFlowItem, SipFlowMsgType};
use anyhow::Result;
use audio_codec::CodecType;
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use rustrtc::media::{MediaKind, MediaSample, MediaStreamTrack, SampleStreamSource};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tracing::{debug, info, warn};

/// Monitor mode for the supervisor leg.
///
/// Controls how audio flows between the monitor and the call participants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
pub struct MonitorLeg {
    /// The sample source feeding the monitor's PeerConnection track.
    pub source: SampleStreamSource,
    /// The codec the monitor expects to receive.
    pub codec: CodecType,
    /// The RTP codec parameters for the monitor's track.
    pub params: rustrtc::RtpCodecParameters,
    /// The current monitor mode.
    pub mode: MonitorMode,
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
            });
        }

        // Set up the PeerConnection track asynchronously
        let track_target: Arc<dyn MediaStreamTrack> = track;
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
    /// Currently only `SilentListen` is fully implemented. `Whisper` and `Barge`
    /// modes set the mode flag but do not yet alter audio routing.
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
                                    let track_kind = track.kind();
                                    info!("Track event Leg A: track_id={} kind={:?}", track_id, track_kind);
                                    if started_track_ids.insert(format!("A-{}", track_id)) {
                                        info!("Starting track forwarder from event: Leg A track_id={}", track_id);
                                        forwarders.push(Self::forward_track(
                                            leg_a.clone(),
                                            track,
                                            pc_b.clone(),
                                            params_b.clone(),
                                            codec_a,
                                            codec_b,
                                            Leg::A,
                                            dtmf_pt_a,
                                            dtmf_pt_b,
                                            None,
                                            recorder.clone(),
                                            call_id.clone(),
                                            sipflow_backend.clone(),
                                            input_gain,
                                            output_gain,
                                            quality.clone(),
                                            monitor.clone(),
                                        ));
                                    } else {
                                        debug!("Track event for already started Leg A track id={}, skipping", track_id);
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
                                    let track_kind = track.kind();
                                    info!("Track event Leg B: track_id={} kind={:?}", track_id, track_kind);
                                    if started_track_ids.insert(format!("B-{}", track_id)) {
                                        info!("Starting track forwarder from event: Leg B track_id={}", track_id);
                                        forwarders.push(Self::forward_track(
                                            leg_b.clone(),
                                            track,
                                            pc_a.clone(),
                                            params_a.clone(),
                                            codec_b,
                                            codec_a,
                                            Leg::B,
                                            dtmf_pt_b,
                                            dtmf_pt_a,
                                            None,
                                            recorder.clone(),
                                            call_id.clone(),
                                            sipflow_backend.clone(),
                                            input_gain,
                                            output_gain,
                                            quality.clone(),
                                            monitor.clone(),
                                        ));
                                    } else {
                                        debug!("Track event for already started Leg B track id={}, skipping", track_id);
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
                                if started_track_ids.insert(format!("A-{}", track_id)) {
                                    forwarders.push(Self::forward_track(
                                        leg_a.clone(),
                                        track,
                                        pc_b.clone(),
                                        params_b.clone(),
                                        codec_a,
                                        codec_b,
                                        Leg::A,
                                        dtmf_pt_a,
                                        dtmf_pt_b,
                                        None,
                                        recorder.clone(),
                                        call_id.clone(),
                                        sipflow_backend.clone(),
                                        input_gain,
                                        output_gain,
                                        quality.clone(),
                                        monitor.clone(),
                                    ));
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
                                if started_track_ids.insert(format!("B-{}", track_id)) {
                                    forwarders.push(Self::forward_track(
                                        leg_b.clone(),
                                        track,
                                        pc_a.clone(),
                                        params_a.clone(),
                                        codec_b,
                                        codec_a,
                                        Leg::B,
                                        dtmf_pt_b,
                                        dtmf_pt_a,
                                        None,
                                        recorder.clone(),
                                        call_id.clone(),
                                        sipflow_backend.clone(),
                                        input_gain,
                                        output_gain,
                                        quality.clone(),
                                        monitor.clone(),
                                    ));
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

        // Only forward audio samples in SilentListen mode (both legs to monitor).
        // Whisper and Barge modes will be extended in future tasks.
        if monitor_leg.mode != MonitorMode::SilentListen {
            // For now, all modes forward audio to monitor. The difference will
            // be in how monitor audio is routed back, which is not yet implemented.
        }

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

    #[tokio::test]
    async fn test_monitor_attach_detach() {
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

        // Initially no monitor
        assert!(!bridge.has_monitor());
        assert!(bridge.monitor_mode().is_none());

        // Attach a monitor
        let monitor_peer = Arc::new(MockMediaPeer::new());
        bridge.attach_monitor(monitor_peer, CodecType::PCMU).unwrap();
        assert!(bridge.has_monitor());
        assert_eq!(bridge.monitor_mode(), Some(MonitorMode::SilentListen));

        // Change mode
        bridge.set_monitor_mode(MonitorMode::Whisper).unwrap();
        assert_eq!(bridge.monitor_mode(), Some(MonitorMode::Whisper));

        // Detach
        bridge.detach_monitor().unwrap();
        assert!(!bridge.has_monitor());
        assert!(bridge.monitor_mode().is_none());

        // set_monitor_mode should fail when no monitor attached
        assert!(bridge.set_monitor_mode(MonitorMode::Barge).is_err());
    }

    #[tokio::test]
    async fn test_monitor_mode_default() {
        assert_eq!(MonitorMode::default(), MonitorMode::SilentListen);
    }
}
