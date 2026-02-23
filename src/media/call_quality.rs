use crate::config::QualityConfig;
use crate::sipflow::{SipFlowBackend, SipFlowItem, SipFlowMsgType};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, warn};

/// Per-leg RTP quality metrics, updated atomically from forward_track.
pub struct LegQuality {
    pub total_packets: AtomicU64,
    pub lost_packets: AtomicU64,
    pub out_of_order: AtomicU64,
    max_jitter_us: AtomicU32,
    /// Running jitter estimate (RFC 3550 §6.4.1), in microseconds.
    jitter_us: AtomicU32,
    last_arrival: Mutex<Option<Instant>>,
    last_rtp_ts: Mutex<Option<u32>>,
    last_seq: Mutex<Option<u16>>,
}

impl LegQuality {
    pub fn new() -> Self {
        Self {
            total_packets: AtomicU64::new(0),
            lost_packets: AtomicU64::new(0),
            out_of_order: AtomicU64::new(0),
            max_jitter_us: AtomicU32::new(0),
            jitter_us: AtomicU32::new(0),
            last_arrival: Mutex::new(None),
            last_rtp_ts: Mutex::new(None),
            last_seq: Mutex::new(None),
        }
    }

    /// Record a received packet for jitter and loss computation.
    /// `seq` is the RTP sequence number, `rtp_ts` is the RTP timestamp,
    /// `clock_rate` is the codec clock rate (e.g. 8000 for PCMU).
    pub fn record_packet(&self, _seq: u16, rtp_ts: u32, clock_rate: u32) {
        let now = Instant::now();
        self.total_packets.fetch_add(1, Ordering::Relaxed);

        // Jitter calculation per RFC 3550 §6.4.1
        let mut last_arrival_guard = self.last_arrival.lock().unwrap();
        let mut last_rtp_ts_guard = self.last_rtp_ts.lock().unwrap();

        if let (Some(prev_arrival), Some(prev_rtp_ts)) =
            (*last_arrival_guard, *last_rtp_ts_guard)
        {
            let arrival_diff_us = now.duration_since(prev_arrival).as_micros() as i64;
            // RTP timestamp diff in microseconds
            let ts_diff = rtp_ts.wrapping_sub(prev_rtp_ts) as i64;
            let ts_diff_us = if clock_rate > 0 {
                (ts_diff * 1_000_000) / clock_rate as i64
            } else {
                0
            };

            let d = (arrival_diff_us - ts_diff_us).unsigned_abs() as u32;

            // J(i) = J(i-1) + (|D(i,j)| - J(i-1)) / 16
            let prev_jitter = self.jitter_us.load(Ordering::Relaxed);
            let new_jitter = if d > prev_jitter {
                prev_jitter + (d - prev_jitter) / 16
            } else {
                prev_jitter - (prev_jitter - d) / 16
            };
            self.jitter_us.store(new_jitter, Ordering::Relaxed);

            // Update max jitter
            let mut current_max = self.max_jitter_us.load(Ordering::Relaxed);
            while new_jitter > current_max {
                match self.max_jitter_us.compare_exchange_weak(
                    current_max,
                    new_jitter,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(val) => current_max = val,
                }
            }
        }

        *last_arrival_guard = Some(now);
        *last_rtp_ts_guard = Some(rtp_ts);
    }

    /// Record detected packet loss (gap in sequence numbers).
    pub fn record_loss(&self, count: u64) {
        self.lost_packets.fetch_add(count, Ordering::Relaxed);
    }

    /// Record an out-of-order packet.
    pub fn record_out_of_order(&self) {
        self.out_of_order.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a sequence number and detect gaps/reordering.
    /// Returns true if the packet should be processed (not a duplicate).
    pub fn record_seq(&self, seq: u16) -> bool {
        let mut last_seq_guard = self.last_seq.lock().unwrap();
        if let Some(last) = *last_seq_guard {
            if seq == last {
                return false; // duplicate
            }
            let expected = last.wrapping_add(1);
            if seq != expected {
                // Check if out of order (seq < expected in wrapping arithmetic)
                let diff = seq.wrapping_sub(last);
                if diff > 0x8000 {
                    // Out of order (seq is behind last)
                    self.record_out_of_order();
                } else if diff > 1 {
                    // Gap: diff-1 packets were lost
                    self.record_loss((diff - 1) as u64);
                }
            }
        }
        *last_seq_guard = Some(seq);
        true
    }

    pub fn snapshot(&self) -> LegQualitySnapshot {
        let total = self.total_packets.load(Ordering::Relaxed);
        let lost = self.lost_packets.load(Ordering::Relaxed);
        let out_of_order = self.out_of_order.load(Ordering::Relaxed);
        let jitter_us = self.jitter_us.load(Ordering::Relaxed);
        let max_jitter_us = self.max_jitter_us.load(Ordering::Relaxed);

        let loss_percent = if total > 0 {
            (lost as f32 / (total + lost) as f32) * 100.0
        } else {
            0.0
        };

        LegQualitySnapshot {
            total_packets: total,
            lost_packets: lost,
            out_of_order,
            loss_percent,
            avg_jitter_ms: jitter_us as f32 / 1000.0,
            max_jitter_ms: max_jitter_us as f32 / 1000.0,
        }
    }
}

/// Aggregated quality for both call legs.
pub struct CallQuality {
    pub leg_a: LegQuality,
    pub leg_b: LegQuality,
    pub created_at: Instant,
    pub media_state: Mutex<MediaLayerState>,
}

impl CallQuality {
    pub fn new() -> Self {
        Self {
            leg_a: LegQuality::new(),
            leg_b: LegQuality::new(),
            created_at: Instant::now(),
            media_state: Mutex::new(MediaLayerState::Negotiating),
        }
    }

    pub fn snapshot(&self) -> CallQualitySnapshot {
        let media_state = self.media_state.lock().unwrap().clone();
        CallQualitySnapshot {
            leg_a: self.leg_a.snapshot(),
            leg_b: self.leg_b.snapshot(),
            duration_secs: self.created_at.elapsed().as_secs_f64(),
            media_state,
        }
    }

    /// Transition media state and log the change.
    pub fn set_media_state(&self, new_state: MediaLayerState) {
        let mut state = self.media_state.lock().unwrap();
        if *state != new_state {
            debug!(
                from = %*state,
                to = %new_state,
                "Media state transition"
            );
            *state = new_state;
        }
    }
}

/// Media layer state machine for tracking call media lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MediaLayerState {
    /// SDP negotiation in progress
    Negotiating,
    /// Media path established (ICE/RTP ready), waiting for first packet
    Established,
    /// RTP packets flowing normally
    Flowing,
    /// Quality has degraded below thresholds
    Degraded(String),
    /// No packets received for extended period
    Stalled,
    /// Unrecoverable media failure
    Failed(String),
}

impl std::fmt::Display for MediaLayerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Negotiating => write!(f, "negotiating"),
            Self::Established => write!(f, "established"),
            Self::Flowing => write!(f, "flowing"),
            Self::Degraded(reason) => write!(f, "degraded: {}", reason),
            Self::Stalled => write!(f, "stalled"),
            Self::Failed(reason) => write!(f, "failed: {}", reason),
        }
    }
}

/// Snapshot of per-leg quality metrics (cheap to clone, serializable).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegQualitySnapshot {
    pub total_packets: u64,
    pub lost_packets: u64,
    pub out_of_order: u64,
    pub loss_percent: f32,
    pub avg_jitter_ms: f32,
    pub max_jitter_ms: f32,
}

/// Snapshot of full call quality (both legs).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallQualitySnapshot {
    pub leg_a: LegQualitySnapshot,
    pub leg_b: LegQualitySnapshot,
    pub duration_secs: f64,
    pub media_state: MediaLayerState,
}

impl CallQualitySnapshot {
    /// Combined packet loss percentage across both legs.
    pub fn combined_loss_percent(&self) -> f32 {
        let total = self.leg_a.total_packets + self.leg_b.total_packets
            + self.leg_a.lost_packets + self.leg_b.lost_packets;
        if total == 0 {
            return 0.0;
        }
        let lost = self.leg_a.lost_packets + self.leg_b.lost_packets;
        (lost as f32 / total as f32) * 100.0
    }

    /// Average jitter across both legs in ms.
    pub fn combined_avg_jitter_ms(&self) -> f32 {
        let mut count = 0;
        let mut sum = 0.0;
        if self.leg_a.total_packets > 0 {
            sum += self.leg_a.avg_jitter_ms;
            count += 1;
        }
        if self.leg_b.total_packets > 0 {
            sum += self.leg_b.avg_jitter_ms;
            count += 1;
        }
        if count > 0 { sum / count as f32 } else { 0.0 }
    }

    /// Total RTP packets across both legs.
    pub fn total_rtp_packets(&self) -> u64 {
        self.leg_a.total_packets + self.leg_b.total_packets
    }

    /// Estimate MOS score using simplified ITU-T G.107 E-model.
    ///
    /// R = 93.2 - Id - Ie_eff
    /// MOS = 1 + 0.035*R + R*(R-60)*(100-R)*7e-6  (for R > 0)
    pub fn mos_estimate(&self) -> f32 {
        let loss = self.combined_loss_percent();
        let jitter = self.combined_avg_jitter_ms();

        // Id: delay impairment from jitter (simplified)
        // Assume effective delay ≈ jitter * 2 (one-way)
        let effective_delay_ms = jitter * 2.0;
        let id = if effective_delay_ms > 177.3 {
            0.024 * effective_delay_ms + 0.11 * (effective_delay_ms - 177.3)
        } else {
            0.024 * effective_delay_ms
        };

        // Ie_eff: equipment impairment from loss
        // For G.711 (PCMU/PCMA): Ie = 0, Bpl = 25.1
        // Ie_eff = Ie + (95 - Ie) * loss / (loss + Bpl)
        let ie = 0.0_f32;
        let bpl = 25.1_f32;
        let ie_eff = ie + (95.0 - ie) * loss / (loss + bpl);

        let r = (93.2 - id - ie_eff).max(0.0);

        if r <= 0.0 {
            1.0
        } else {
            let mos = 1.0 + 0.035 * r + r * (r - 60.0) * (100.0 - r) * 7e-6;
            mos.clamp(1.0, 4.5) as f32
        }
    }
}

/// Quality assessment action based on current metrics.
#[derive(Debug, Clone, PartialEq)]
pub enum QualityAction {
    Healthy,
    LogWarning { loss_pct: f32, jitter_ms: f32 },
    NotifyDegraded { loss_pct: f32, jitter_ms: f32 },
    Critical { loss_pct: f32, jitter_ms: f32 },
}

/// Quality thresholds (configurable).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityThresholds {
    pub loss_warning_pct: f32,
    pub loss_critical_pct: f32,
    pub jitter_warning_ms: f32,
    pub jitter_critical_ms: f32,
}

impl Default for QualityThresholds {
    fn default() -> Self {
        Self {
            loss_warning_pct: 3.0,
            loss_critical_pct: 20.0,
            jitter_warning_ms: 60.0,
            jitter_critical_ms: 200.0,
        }
    }
}

/// Evaluate quality of a single leg against thresholds.
pub fn evaluate_leg(snap: &LegQualitySnapshot, thresholds: &QualityThresholds) -> QualityAction {
    let loss = snap.loss_percent;
    let jitter = snap.avg_jitter_ms;

    if loss >= thresholds.loss_critical_pct || jitter >= thresholds.jitter_critical_ms {
        QualityAction::Critical {
            loss_pct: loss,
            jitter_ms: jitter,
        }
    } else if loss >= thresholds.loss_critical_pct * 0.5 || jitter >= thresholds.jitter_critical_ms * 0.6 {
        QualityAction::NotifyDegraded {
            loss_pct: loss,
            jitter_ms: jitter,
        }
    } else if loss >= thresholds.loss_warning_pct || jitter >= thresholds.jitter_warning_ms {
        QualityAction::LogWarning {
            loss_pct: loss,
            jitter_ms: jitter,
        }
    } else {
        QualityAction::Healthy
    }
}

/// Evaluate both legs of a call.
pub fn evaluate(quality: &CallQuality, thresholds: &QualityThresholds) -> (QualityAction, QualityAction) {
    let snap_a = quality.leg_a.snapshot();
    let snap_b = quality.leg_b.snapshot();
    (evaluate_leg(&snap_a, thresholds), evaluate_leg(&snap_b, thresholds))
}

impl QualityThresholds {
    /// Create thresholds from config, using defaults for any unset values.
    pub fn from_config(config: &QualityConfig) -> Self {
        let defaults = Self::default();
        Self {
            loss_warning_pct: config.loss_warning_pct.unwrap_or(defaults.loss_warning_pct),
            loss_critical_pct: config.loss_critical_pct.unwrap_or(defaults.loss_critical_pct),
            jitter_warning_ms: config.jitter_warning_ms.unwrap_or(defaults.jitter_warning_ms),
            jitter_critical_ms: config.jitter_critical_ms.unwrap_or(defaults.jitter_critical_ms),
        }
    }
}

/// Quality watchdog that periodically evaluates call quality and logs/reports issues.
///
/// Runs every `interval_secs` seconds, evaluating both legs against thresholds.
/// Logs warnings/errors and emits quality events to SipFlow when configured.
pub async fn quality_watchdog(
    quality: Arc<CallQuality>,
    call_id: String,
    cancel_token: CancellationToken,
    sipflow_backend: Option<Arc<dyn SipFlowBackend>>,
    thresholds: QualityThresholds,
    interval_secs: u64,
) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
    // Skip the first immediate tick
    interval.tick().await;

    let mut consecutive_critical_a: u32 = 0;
    let mut consecutive_critical_b: u32 = 0;
    let mut last_total_packets: u64 = 0;
    let mut stall_count: u32 = 0;
    const STALL_THRESHOLD: u32 = 3; // 3 intervals with no new packets = stalled

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                debug!(call_id, "Quality watchdog stopped (cancelled)");
                break;
            }
            _ = interval.tick() => {
                let snap = quality.snapshot();
                let (action_a, action_b) = {
                    let snap_a = quality.leg_a.snapshot();
                    let snap_b = quality.leg_b.snapshot();
                    (evaluate_leg(&snap_a, &thresholds), evaluate_leg(&snap_b, &thresholds))
                };

                // Track consecutive critical strikes
                match &action_a {
                    QualityAction::Critical { .. } => consecutive_critical_a += 1,
                    QualityAction::Healthy => consecutive_critical_a = 0,
                    _ => {} // Warning/Degraded don't reset but don't increment
                }
                match &action_b {
                    QualityAction::Critical { .. } => consecutive_critical_b += 1,
                    QualityAction::Healthy => consecutive_critical_b = 0,
                    _ => {}
                }

                // Media state tracking
                let current_total = snap.total_rtp_packets();
                if current_total == 0 {
                    // Still waiting for first packet
                    quality.set_media_state(MediaLayerState::Established);
                } else if current_total == last_total_packets {
                    stall_count += 1;
                    if stall_count >= STALL_THRESHOLD {
                        quality.set_media_state(MediaLayerState::Stalled);
                    }
                } else {
                    stall_count = 0;
                    // Determine state based on worst action across both legs
                    let worst = worst_action(&action_a, &action_b);
                    match worst {
                        QualityAction::Critical { loss_pct, jitter_ms } => {
                            quality.set_media_state(MediaLayerState::Degraded(
                                format!("loss={:.1}% jitter={:.1}ms", loss_pct, jitter_ms),
                            ));
                        }
                        QualityAction::NotifyDegraded { loss_pct, jitter_ms } => {
                            quality.set_media_state(MediaLayerState::Degraded(
                                format!("loss={:.1}% jitter={:.1}ms", loss_pct, jitter_ms),
                            ));
                        }
                        _ => {
                            quality.set_media_state(MediaLayerState::Flowing);
                        }
                    }
                }
                last_total_packets = current_total;

                // Log based on action severity
                log_quality_action(&call_id, "leg_a", &action_a, consecutive_critical_a);
                log_quality_action(&call_id, "leg_b", &action_b, consecutive_critical_b);

                // Emit quality event to SipFlow
                if let Some(ref backend) = sipflow_backend {
                    let mos = snap.mos_estimate();
                    let quality_json = serde_json::json!({
                        "type": "quality_report",
                        "mos": mos,
                        "leg_a": {
                            "loss_pct": snap.leg_a.loss_percent,
                            "jitter_ms": snap.leg_a.avg_jitter_ms,
                            "packets": snap.leg_a.total_packets,
                            "action": format!("{:?}", action_a),
                        },
                        "leg_b": {
                            "loss_pct": snap.leg_b.loss_percent,
                            "jitter_ms": snap.leg_b.avg_jitter_ms,
                            "packets": snap.leg_b.total_packets,
                            "action": format!("{:?}", action_b),
                        },
                        "duration_secs": snap.duration_secs,
                    });

                    let payload = bytes::Bytes::from(quality_json.to_string());
                    let item = SipFlowItem {
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                        seq: 0,
                        msg_type: SipFlowMsgType::Quality,
                        src_addr: "watchdog".to_string(),
                        dst_addr: "quality".to_string(),
                        payload,
                    };

                    if let Err(e) = backend.record(&call_id, item) {
                        debug!(call_id, "Failed to record quality event to sipflow: {}", e);
                    }
                }
            }
        }
    }
}

fn action_severity(action: &QualityAction) -> u8 {
    match action {
        QualityAction::Healthy => 0,
        QualityAction::LogWarning { .. } => 1,
        QualityAction::NotifyDegraded { .. } => 2,
        QualityAction::Critical { .. } => 3,
    }
}

fn worst_action<'a>(a: &'a QualityAction, b: &'a QualityAction) -> &'a QualityAction {
    if action_severity(a) >= action_severity(b) { a } else { b }
}

fn log_quality_action(call_id: &str, leg: &str, action: &QualityAction, consecutive_critical: u32) {
    match action {
        QualityAction::Healthy => {
            debug!(call_id, leg, "Quality: healthy");
        }
        QualityAction::LogWarning { loss_pct, jitter_ms } => {
            warn!(
                call_id, leg,
                loss_pct, jitter_ms,
                "Quality: warning - elevated loss/jitter"
            );
        }
        QualityAction::NotifyDegraded { loss_pct, jitter_ms } => {
            warn!(
                call_id, leg,
                loss_pct, jitter_ms,
                "Quality: DEGRADED"
            );
        }
        QualityAction::Critical { loss_pct, jitter_ms } => {
            error!(
                call_id, leg,
                loss_pct, jitter_ms,
                consecutive_critical,
                "Quality: CRITICAL"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leg_quality_no_packets() {
        let leg = LegQuality::new();
        let snap = leg.snapshot();
        assert_eq!(snap.total_packets, 0);
        assert_eq!(snap.lost_packets, 0);
        assert_eq!(snap.loss_percent, 0.0);
        assert_eq!(snap.avg_jitter_ms, 0.0);
    }

    #[test]
    fn test_leg_quality_sequential_packets() {
        let leg = LegQuality::new();
        for seq in 0..100u16 {
            assert!(leg.record_seq(seq));
            leg.record_packet(seq, seq as u32 * 160, 8000);
        }
        let snap = leg.snapshot();
        assert_eq!(snap.total_packets, 100);
        assert_eq!(snap.lost_packets, 0);
        assert_eq!(snap.loss_percent, 0.0);
    }

    #[test]
    fn test_leg_quality_duplicate_detection() {
        let leg = LegQuality::new();
        assert!(leg.record_seq(1));
        assert!(leg.record_seq(2));
        assert!(!leg.record_seq(2)); // duplicate
        assert!(leg.record_seq(3));
    }

    #[test]
    fn test_leg_quality_gap_detection() {
        let leg = LegQuality::new();
        leg.record_seq(1);
        leg.record_seq(2);
        leg.record_seq(5); // gap: 3,4 lost
        let snap = leg.snapshot();
        assert_eq!(snap.lost_packets, 2);
    }

    #[test]
    fn test_leg_quality_out_of_order() {
        let leg = LegQuality::new();
        leg.record_seq(1);
        leg.record_seq(3);
        leg.record_seq(2); // out of order
        let snap = leg.snapshot();
        assert_eq!(snap.out_of_order, 1);
    }

    #[test]
    fn test_call_quality_snapshot() {
        let cq = CallQuality::new();
        for seq in 0..50u16 {
            cq.leg_a.record_seq(seq);
            cq.leg_a.record_packet(seq, seq as u32 * 160, 8000);
        }
        for seq in 0..50u16 {
            cq.leg_b.record_seq(seq);
            cq.leg_b.record_packet(seq, seq as u32 * 160, 8000);
        }
        let snap = cq.snapshot();
        assert_eq!(snap.total_rtp_packets(), 100);
        assert_eq!(snap.combined_loss_percent(), 0.0);
    }

    #[test]
    fn test_mos_perfect_call() {
        let snap = CallQualitySnapshot {
            leg_a: LegQualitySnapshot {
                total_packets: 1000,
                lost_packets: 0,
                out_of_order: 0,
                loss_percent: 0.0,
                avg_jitter_ms: 1.0,
                max_jitter_ms: 2.0,
            },
            leg_b: LegQualitySnapshot {
                total_packets: 1000,
                lost_packets: 0,
                out_of_order: 0,
                loss_percent: 0.0,
                avg_jitter_ms: 1.0,
                max_jitter_ms: 2.0,
            },
            duration_secs: 60.0,
            media_state: MediaLayerState::Flowing,
        };
        let mos = snap.mos_estimate();
        assert!(mos > 4.0, "Perfect call MOS should be > 4.0, got {}", mos);
    }

    #[test]
    fn test_mos_degraded_call() {
        let snap = CallQualitySnapshot {
            leg_a: LegQualitySnapshot {
                total_packets: 1000,
                lost_packets: 100,
                out_of_order: 5,
                loss_percent: 10.0,
                avg_jitter_ms: 50.0,
                max_jitter_ms: 100.0,
            },
            leg_b: LegQualitySnapshot {
                total_packets: 1000,
                lost_packets: 100,
                out_of_order: 5,
                loss_percent: 10.0,
                avg_jitter_ms: 50.0,
                max_jitter_ms: 100.0,
            },
            duration_secs: 60.0,
            media_state: MediaLayerState::Flowing,
        };
        let mos = snap.mos_estimate();
        assert!(mos < 4.0, "Degraded call MOS should be < 4.0, got {}", mos);
        assert!(mos > 1.0, "Degraded call MOS should be > 1.0, got {}", mos);
    }

    #[test]
    fn test_evaluate_healthy() {
        let cq = CallQuality::new();
        for seq in 0..100u16 {
            cq.leg_a.record_seq(seq);
            cq.leg_a.record_packet(seq, seq as u32 * 160, 8000);
            cq.leg_b.record_seq(seq);
            cq.leg_b.record_packet(seq, seq as u32 * 160, 8000);
        }
        let thresholds = QualityThresholds::default();
        let (a, b) = evaluate(&cq, &thresholds);
        assert_eq!(a, QualityAction::Healthy);
        assert_eq!(b, QualityAction::Healthy);
    }

    #[test]
    fn test_evaluate_with_loss() {
        let snap = LegQualitySnapshot {
            total_packets: 100,
            lost_packets: 25,
            out_of_order: 0,
            loss_percent: 20.0,
            avg_jitter_ms: 10.0,
            max_jitter_ms: 20.0,
        };
        let thresholds = QualityThresholds::default();
        let action = evaluate_leg(&snap, &thresholds);
        assert!(matches!(action, QualityAction::Critical { .. }));
    }

    #[test]
    fn test_evaluate_warning() {
        let snap = LegQualitySnapshot {
            total_packets: 100,
            lost_packets: 4,
            out_of_order: 0,
            loss_percent: 4.0,
            avg_jitter_ms: 30.0,
            max_jitter_ms: 50.0,
        };
        let thresholds = QualityThresholds::default();
        let action = evaluate_leg(&snap, &thresholds);
        assert!(matches!(action, QualityAction::LogWarning { .. }));
    }

    #[test]
    fn test_evaluate_degraded() {
        // loss >= critical*0.5 (10%) but < critical (20%) => NotifyDegraded
        let snap = LegQualitySnapshot {
            total_packets: 100,
            lost_packets: 12,
            out_of_order: 0,
            loss_percent: 12.0,
            avg_jitter_ms: 30.0,
            max_jitter_ms: 50.0,
        };
        let thresholds = QualityThresholds::default();
        let action = evaluate_leg(&snap, &thresholds);
        assert!(matches!(action, QualityAction::NotifyDegraded { .. }));
    }

    #[test]
    fn test_evaluate_high_jitter_critical() {
        // jitter >= 200ms => Critical
        let snap = LegQualitySnapshot {
            total_packets: 1000,
            lost_packets: 0,
            out_of_order: 0,
            loss_percent: 0.0,
            avg_jitter_ms: 250.0,
            max_jitter_ms: 300.0,
        };
        let thresholds = QualityThresholds::default();
        let action = evaluate_leg(&snap, &thresholds);
        assert!(matches!(action, QualityAction::Critical { .. }));
    }

    #[test]
    fn test_evaluate_high_jitter_degraded() {
        // jitter >= 120ms (200*0.6) but < 200ms => NotifyDegraded
        let snap = LegQualitySnapshot {
            total_packets: 1000,
            lost_packets: 0,
            out_of_order: 0,
            loss_percent: 0.0,
            avg_jitter_ms: 130.0,
            max_jitter_ms: 150.0,
        };
        let thresholds = QualityThresholds::default();
        let action = evaluate_leg(&snap, &thresholds);
        assert!(matches!(action, QualityAction::NotifyDegraded { .. }));
    }

    #[test]
    fn test_thresholds_from_config() {
        let config = QualityConfig {
            enabled: true,
            watchdog_interval_secs: 5,
            loss_warning_pct: Some(5.0),
            loss_critical_pct: None, // should use default 20.0
            jitter_warning_ms: Some(80.0),
            jitter_critical_ms: Some(300.0),
        };
        let thresholds = QualityThresholds::from_config(&config);
        assert_eq!(thresholds.loss_warning_pct, 5.0);
        assert_eq!(thresholds.loss_critical_pct, 20.0); // default
        assert_eq!(thresholds.jitter_warning_ms, 80.0);
        assert_eq!(thresholds.jitter_critical_ms, 300.0);
    }

    #[test]
    fn test_mos_zero_loss_zero_jitter() {
        // Perfect conditions should give MOS > 4.3
        let snap = CallQualitySnapshot {
            leg_a: LegQualitySnapshot {
                total_packets: 5000,
                lost_packets: 0,
                out_of_order: 0,
                loss_percent: 0.0,
                avg_jitter_ms: 0.0,
                max_jitter_ms: 0.0,
            },
            leg_b: LegQualitySnapshot {
                total_packets: 5000,
                lost_packets: 0,
                out_of_order: 0,
                loss_percent: 0.0,
                avg_jitter_ms: 0.0,
                max_jitter_ms: 0.0,
            },
            duration_secs: 120.0,
            media_state: MediaLayerState::Flowing,
        };
        let mos = snap.mos_estimate();
        assert!(mos > 4.3, "Perfect MOS should be > 4.3, got {}", mos);
        assert!(mos <= 4.5, "MOS should be clamped to 4.5, got {}", mos);
    }

    #[test]
    fn test_mos_severe_loss() {
        // 50% loss should give very low MOS
        let snap = CallQualitySnapshot {
            leg_a: LegQualitySnapshot {
                total_packets: 500,
                lost_packets: 500,
                out_of_order: 0,
                loss_percent: 50.0,
                avg_jitter_ms: 10.0,
                max_jitter_ms: 20.0,
            },
            leg_b: LegQualitySnapshot {
                total_packets: 500,
                lost_packets: 500,
                out_of_order: 0,
                loss_percent: 50.0,
                avg_jitter_ms: 10.0,
                max_jitter_ms: 20.0,
            },
            duration_secs: 60.0,
            media_state: MediaLayerState::Flowing,
        };
        let mos = snap.mos_estimate();
        assert!(mos < 2.0, "50% loss MOS should be < 2.0, got {}", mos);
    }

    #[test]
    fn test_mos_moderate_degradation() {
        // 5% loss, 40ms jitter should give MOS between 2.5 and 4.0
        let snap = CallQualitySnapshot {
            leg_a: LegQualitySnapshot {
                total_packets: 1000,
                lost_packets: 50,
                out_of_order: 2,
                loss_percent: 5.0,
                avg_jitter_ms: 40.0,
                max_jitter_ms: 80.0,
            },
            leg_b: LegQualitySnapshot {
                total_packets: 1000,
                lost_packets: 50,
                out_of_order: 2,
                loss_percent: 5.0,
                avg_jitter_ms: 40.0,
                max_jitter_ms: 80.0,
            },
            duration_secs: 60.0,
            media_state: MediaLayerState::Flowing,
        };
        let mos = snap.mos_estimate();
        assert!(mos > 2.5, "Moderate degradation MOS should be > 2.5, got {}", mos);
        assert!(mos < 4.0, "Moderate degradation MOS should be < 4.0, got {}", mos);
    }

    #[test]
    fn test_wrapping_sequence_numbers() {
        // Test sequence number wrapping around u16::MAX
        let leg = LegQuality::new();
        leg.record_seq(65534);
        leg.record_seq(65535);
        leg.record_seq(0); // wraps
        leg.record_seq(1);
        let snap = leg.snapshot();
        assert_eq!(snap.lost_packets, 0);
        assert_eq!(snap.out_of_order, 0);
    }

    #[test]
    fn test_large_sequence_gap() {
        let leg = LegQuality::new();
        leg.record_seq(100);
        leg.record_seq(200); // 99 packets lost
        let snap = leg.snapshot();
        assert_eq!(snap.lost_packets, 99);
    }

    #[tokio::test]
    async fn test_watchdog_cancellation() {
        let quality = Arc::new(CallQuality::new());
        let cancel = CancellationToken::new();
        let thresholds = QualityThresholds::default();

        let cancel_clone = cancel.clone();
        let handle = tokio::spawn(quality_watchdog(
            quality,
            "test-watchdog".to_string(),
            cancel_clone,
            None,
            thresholds,
            1,
        ));

        // Cancel after a short delay
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        cancel.cancel();

        // Should complete promptly
        let result = tokio::time::timeout(std::time::Duration::from_secs(2), handle).await;
        assert!(result.is_ok(), "Watchdog should exit after cancellation");
    }

    #[test]
    fn test_media_layer_state_transitions() {
        let cq = CallQuality::new();

        // Initial state is Negotiating
        let snap = cq.snapshot();
        assert_eq!(snap.media_state, MediaLayerState::Negotiating);

        // Transition to Established
        cq.set_media_state(MediaLayerState::Established);
        let snap = cq.snapshot();
        assert_eq!(snap.media_state, MediaLayerState::Established);

        // Transition to Flowing
        cq.set_media_state(MediaLayerState::Flowing);
        let snap = cq.snapshot();
        assert_eq!(snap.media_state, MediaLayerState::Flowing);

        // Transition to Degraded
        cq.set_media_state(MediaLayerState::Degraded("loss=15%".to_string()));
        let snap = cq.snapshot();
        assert!(matches!(snap.media_state, MediaLayerState::Degraded(_)));

        // Back to Flowing
        cq.set_media_state(MediaLayerState::Flowing);
        let snap = cq.snapshot();
        assert_eq!(snap.media_state, MediaLayerState::Flowing);

        // Transition to Stalled
        cq.set_media_state(MediaLayerState::Stalled);
        let snap = cq.snapshot();
        assert_eq!(snap.media_state, MediaLayerState::Stalled);
    }

    #[test]
    fn test_media_layer_state_display() {
        assert_eq!(format!("{}", MediaLayerState::Negotiating), "negotiating");
        assert_eq!(format!("{}", MediaLayerState::Established), "established");
        assert_eq!(format!("{}", MediaLayerState::Flowing), "flowing");
        assert_eq!(
            format!("{}", MediaLayerState::Degraded("high loss".to_string())),
            "degraded: high loss"
        );
        assert_eq!(format!("{}", MediaLayerState::Stalled), "stalled");
        assert_eq!(
            format!("{}", MediaLayerState::Failed("timeout".to_string())),
            "failed: timeout"
        );
    }
}
