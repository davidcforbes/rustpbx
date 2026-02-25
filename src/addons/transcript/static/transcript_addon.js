
(function () {
    // Wait for Alpine to be available
    document.addEventListener('alpine:init', () => {
        Alpine.data('transcriptAddon', (recordId) => ({
            recordId: recordId,
            transcript: {
                available: false,
                content: null,
                generated_at: null,
            },
            transcriptStatus: 'pending', // pending, processing, completed, failed
            transcriptLoading: false,
            transcriptVisible: false,
            asrProcessing: false,
            errorMessage: null,
            selectedLanguage: 'auto',
            languageOptions: [
                { value: 'auto', label: 'Auto Detect' },
                { value: 'zh', label: 'Chinese' },
                { value: 'en', label: 'English' },
                { value: 'ja', label: 'Japanese' },
                { value: 'ko', label: 'Korean' },
                { value: 'yue', label: 'Cantonese' },
            ],

            init() {
                // Try to get initial state from the main record object if available
                // We assume the main record object is available in the scope or we fetch it
                // For now, we'll fetch the transcript status
                this.checkTranscriptStatus();
            },

            async checkTranscriptStatus() {
                if (!this.recordId) return;

                try {
                    const response = await fetch(`/console/call-records/${this.recordId}/transcript`);
                    if (response.ok) {
                        const data = await response.json();
                        if (data && data.transcript) {
                            this.transcript = data.transcript;
                            this.transcriptStatus = 'completed';
                            this.transcriptVisible = true;
                        } else if (data && data.status) {
                            this.transcriptStatus = data.status;
                        }
                    }
                } catch (e) {
                    console.error("Failed to check transcript status", e);
                }
            },

            async requestTranscript(force = false) {
                if (this.asrProcessing || this.transcriptLoading) return;

                // If we already have it and not forcing, just toggle visibility
                if (this.transcriptStatus === 'completed' && !force) {
                    this.transcriptVisible = !this.transcriptVisible;
                    return;
                }

                this.asrProcessing = true;
                try {
                    const response = await fetch(`/console/call-records/${this.recordId}/transcript`, {
                        method: 'POST',
                        headers: {
                            'Content-Type': 'application/json',
                        },
                        body: JSON.stringify({
                            language: this.selectedLanguage,
                            action: 'transcribe',
                            force: force
                        })
                    });

                    if (response.ok) {
                        const data = await response.json();
                        this.transcriptStatus = 'processing';
                        this.errorMessage = null;
                        // Poll for status
                        this.pollStatus();
                    } else {
                        this.transcriptStatus = 'failed';
                        try {
                            const errorData = await response.json();
                            this.errorMessage = errorData.message || "Request failed";
                        } catch (e) {
                            this.errorMessage = "Failed to request transcript";
                        }
                        console.error("Failed to request transcript");
                    }
                } catch (e) {
                    console.error("Error requesting transcript", e);
                    this.transcriptStatus = 'failed';
                    this.errorMessage = e.message || "Network error";
                } finally {
                    this.asrProcessing = false;
                }
            },

            pollStatus() {
                const interval = setInterval(async () => {
                    if (this.transcriptStatus === 'completed' || this.transcriptStatus === 'failed') {
                        clearInterval(interval);
                        return;
                    }
                    await this.checkTranscriptStatus();
                }, 2000);
            },

            // Helpers for UI
            transcriptStatusTone(status) {
                switch ((status || '').toLowerCase()) {
                    case 'completed': return 'text-emerald-600';
                    case 'processing': return 'text-amber-500';
                    case 'failed': return 'text-rose-500';
                    default: return 'text-slate-400';
                }
            },

            transcriptStatusLabel(status) {
                switch ((status || '').toLowerCase()) {
                    case 'completed': return 'Ready';
                    case 'processing': return 'Processing';
                    case 'failed': return 'Failed';
                    default: return 'Pending';
                }
            },

            transcriptButtonLabel() {
                if (this.asrProcessing) return 'Submitting…';
                if (this.transcriptLoading) return 'Loading…';
                const status = (this.transcriptStatus || '').toLowerCase();
                if (status === 'processing') return 'Refresh status';
                if (status === 'failed') return 'Retry transcript';
                if (this.transcriptVisible) return 'Re-run transcript';
                if (this.transcriptStatus === 'completed') return 'Load transcript';
                return 'Request transcript';
            },

            formatDateTime(value) {
                if (!value) return '—';
                return new Date(value).toLocaleString();
            },

            // Timeline helpers
            transcriptTimeline() {
                const segments = Array.isArray(this.transcript?.segments) ? this.transcript.segments : [];
                if (!segments.length) {
                    return [];
                }
                const enriched = segments.map((segment, index) => {
                    const startValue = Number(segment?.start);
                    const endValue = Number(segment?.end);
                    const rawChannel = segment ? segment.channel : null;
                    let channelKey = 'mono';
                    if (rawChannel !== null && rawChannel !== undefined && rawChannel !== '') {
                        const numericChannel = Number(rawChannel);
                        channelKey = Number.isFinite(numericChannel) ? numericChannel : 'mono';
                    }
                    let side = 'mono';
                    if (channelKey !== 'mono') {
                        const numeric = Number(channelKey);
                        if (Number.isFinite(numeric)) {
                            if (numeric === 0) {
                                side = 'left';
                            } else if (numeric === 1) {
                                side = 'right';
                            } else {
                                side = numeric % 2 === 0 ? 'left' : 'right';
                            }
                        }
                    }
                    const label = segment?.speaker || this.channelLabel(channelKey);
                    const start = Number.isFinite(startValue) ? startValue : null;
                    const end = Number.isFinite(endValue) ? endValue : null;
                    return {
                        key: `timeline-${index}-${segment?.idx ?? index}-${start ?? ''}`,
                        segment,
                        side,
                        label,
                        start,
                        end,
                    };
                });
                enriched.sort((a, b) => {
                    if (a.start === null && b.start === null) {
                        return 0;
                    }
                    if (a.start === null) {
                        return 1;
                    }
                    if (b.start === null) {
                        return -1;
                    }
                    if (a.start === b.start) {
                        return 0;
                    }
                    return a.start - b.start;
                });
                return enriched;
            },
            transcriptAlignmentClass(side) {
                if (side === 'right') {
                    return 'justify-end';
                }
                if (side === 'mono') {
                    return 'justify-center';
                }
                return 'justify-start';
            },
            transcriptCardTone(side) {
                if (side === 'right') {
                    return 'border-emerald-200 bg-emerald-50/80';
                }
                if (side === 'left') {
                    return 'border-sky-200 bg-sky-50/80';
                }
                return 'border-slate-200 bg-white/90';
            },
            channelLabel(key) {
                if (key === 'mono' || key === null || key === undefined) {
                    return 'Mono channel';
                }
                const numeric = Number(key);
                if (!Number.isFinite(numeric)) {
                    return `Channel ${key}`;
                }
                if (numeric === 0) {
                    return 'Left channel';
                }
                if (numeric === 1) {
                    return 'Right channel';
                }
                return `Channel ${numeric + 1}`;
            },
            formatSegmentTimestamp(value) {
                if (value === undefined || value === null) {
                    return '';
                }
                const numeric = Number(value);
                if (!Number.isFinite(numeric)) {
                    return String(value);
                }
                return `${numeric.toFixed(1)}s`;
            },
            formatSegmentRange(start, end) {
                const startText = this.formatSegmentTimestamp(start);
                const endText = this.formatSegmentTimestamp(end);
                const hasStart = Boolean(startText);
                const hasEnd = end !== undefined && end !== null && end !== '' && Boolean(endText);
                if (hasStart && hasEnd) {
                    if (startText === endText) {
                        return startText;
                    }
                    return `${startText} → ${endText}`;
                }
                if (!hasStart && hasEnd) {
                    return endText;
                }
                return startText;
            }
        }));
    });

    // UI injection removed — the built-in call_record_detail.html template
    // already renders the transcript section with better speaker labels and analytics.
})();
