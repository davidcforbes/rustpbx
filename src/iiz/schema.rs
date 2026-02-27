// @generated automatically by Diesel CLI.

pub mod iiz {
    pub mod sql_types {
        #[derive(diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "account_status", schema = "iiz"))]
        pub struct AccountStatus;

        #[derive(diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "account_type", schema = "iiz"))]
        pub struct AccountType;

        #[derive(diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "active_call_status", schema = "iiz"))]
        pub struct ActiveCallStatus;

        #[derive(diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "agent_status", schema = "iiz"))]
        pub struct AgentStatus;

        #[derive(diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "attestation_level", schema = "iiz"))]
        pub struct AttestationLevel;

        #[derive(diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "call_direction", schema = "iiz"))]
        pub struct CallDirection;

        #[derive(diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "call_status", schema = "iiz"))]
        pub struct CallStatus;

        #[derive(diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "channel_type", schema = "iiz"))]
        pub struct ChannelType;

        #[derive(diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "compliance_status", schema = "iiz"))]
        pub struct ComplianceStatus;

        #[derive(diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "export_format", schema = "iiz"))]
        pub struct ExportFormat;

        #[derive(diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "greeting_type", schema = "iiz"))]
        pub struct GreetingType;

        #[derive(diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "monitor_mode", schema = "iiz"))]
        pub struct MonitorMode;

        #[derive(diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "number_class", schema = "iiz"))]
        pub struct NumberClass;

        #[derive(diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "number_type", schema = "iiz"))]
        pub struct NumberType;

        #[derive(diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "queue_strategy", schema = "iiz"))]
        pub struct QueueStrategy;

        #[derive(diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "sip_transport", schema = "iiz"))]
        pub struct SipTransport;

        #[derive(diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "speaker_type", schema = "iiz"))]
        pub struct SpeakerType;

        #[derive(diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "summary_type", schema = "iiz"))]
        pub struct SummaryType;

        #[derive(diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "user_role", schema = "iiz"))]
        pub struct UserRole;

        pub use pgvector::sql_types::Vector;

        #[derive(diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "workflow_node_type", schema = "iiz"))]
        pub struct WorkflowNodeType;
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::ComplianceStatus;

        iiz.a2p_campaigns (id) {
            id -> Uuid,
            account_id -> Uuid,
            campaign_name -> Text,
            brand_name -> Nullable<Text>,
            use_case -> Nullable<Text>,
            description -> Nullable<Text>,
            sample_messages -> Nullable<Text>,
            opt_in_description -> Nullable<Text>,
            opt_out_description -> Nullable<Text>,
            assigned_numbers -> Int4,
            max_numbers -> Nullable<Int4>,
            monthly_cost -> Nullable<Numeric>,
            carrier -> Nullable<Text>,
            status -> ComplianceStatus,
            rejection_reason -> Nullable<Text>,
            dlc_campaign_id -> Nullable<Text>,
            submitted_at -> Nullable<Timestamptz>,
            approved_at -> Nullable<Timestamptz>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.account_variables (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Text,
            value -> Nullable<Text>,
            description -> Nullable<Text>,
            is_secret -> Bool,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::AccountType;
        use super::sql_types::AccountStatus;

        iiz.accounts (id) {
            id -> Uuid,
            name -> Text,
            account_type -> AccountType,
            parent_account_id -> Nullable<Uuid>,
            slug -> Text,
            timezone -> Text,
            status -> AccountStatus,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::CallDirection;
        use super::sql_types::ActiveCallStatus;
        use super::sql_types::MonitorMode;

        iiz.active_calls (id) {
            id -> Uuid,
            account_id -> Uuid,
            call_id -> Text,
            caller_name -> Nullable<Text>,
            caller_number -> Nullable<Text>,
            callee_number -> Nullable<Text>,
            agent_id -> Nullable<Uuid>,
            queue_id -> Nullable<Uuid>,
            source_id -> Nullable<Uuid>,
            tracking_number_id -> Nullable<Uuid>,
            direction -> CallDirection,
            status -> ActiveCallStatus,
            started_at -> Timestamptz,
            answered_at -> Nullable<Timestamptz>,
            is_monitored -> Bool,
            monitor_mode -> Nullable<MonitorMode>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.agent_scripts (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Text,
            description -> Nullable<Text>,
            content -> Text,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::AgentStatus;

        iiz.agent_state_log (id, changed_at) {
            id -> Uuid,
            account_id -> Uuid,
            agent_id -> Uuid,
            status -> AgentStatus,
            changed_at -> Timestamptz,
            duration_secs -> Nullable<Int4>,
            reason -> Nullable<Text>,
            created_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.api_log_entries (id, timestamp) {
            id -> Uuid,
            account_id -> Uuid,
            source -> Nullable<Text>,
            method -> Text,
            endpoint -> Text,
            request_headers -> Nullable<Jsonb>,
            request_body -> Nullable<Jsonb>,
            response_code -> Nullable<Int4>,
            response_body -> Nullable<Jsonb>,
            response_size_bytes -> Nullable<Int4>,
            duration_ms -> Nullable<Int4>,
            activity_description -> Nullable<Text>,
            error_message -> Nullable<Text>,
            timestamp -> Timestamptz,
            created_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.appointments (id) {
            id -> Uuid,
            account_id -> Uuid,
            call_id -> Nullable<Uuid>,
            scheduled_at -> Timestamptz,
            caller_name -> Nullable<Text>,
            caller_phone -> Nullable<Text>,
            source_id -> Nullable<Uuid>,
            agent_id -> Nullable<Uuid>,
            appointment_type -> Text,
            status -> Text,
            revenue -> Nullable<Numeric>,
            notes -> Nullable<Text>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.ask_ai_configs (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Text,
            preset -> Text,
            custom_prompt -> Nullable<Text>,
            tracking_number_id -> Nullable<Uuid>,
            delay -> Nullable<Text>,
            output_action -> Nullable<Text>,
            workflow_ids -> Nullable<Jsonb>,
            is_active -> Bool,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.authorized_contacts (id) {
            id -> Uuid,
            account_id -> Uuid,
            business_info_id -> Uuid,
            name -> Text,
            title -> Nullable<Text>,
            phone -> Nullable<Text>,
            email -> Nullable<Text>,
            is_primary -> Bool,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.blocked_numbers (id) {
            id -> Uuid,
            account_id -> Uuid,
            number -> Text,
            cnam -> Nullable<Text>,
            calls_blocked -> Int4,
            last_blocked_at -> Nullable<Timestamptz>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.bulk_messages (id) {
            id -> Uuid,
            account_id -> Uuid,
            label -> Nullable<Text>,
            sender_number_id -> Nullable<Uuid>,
            sender_phone -> Nullable<Text>,
            message_body -> Text,
            msg_type -> Text,
            contact_list_id -> Nullable<Uuid>,
            recipient_count -> Int4,
            sent_count -> Int4,
            delivered_count -> Int4,
            failed_count -> Int4,
            status -> Text,
            scheduled_at -> Nullable<Timestamptz>,
            started_at -> Nullable<Timestamptz>,
            completed_at -> Nullable<Timestamptz>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.business_info (id) {
            id -> Uuid,
            account_id -> Uuid,
            legal_business_name -> Nullable<Text>,
            dba -> Nullable<Text>,
            ein -> Nullable<Text>,
            industry -> Nullable<Text>,
            address_line1 -> Nullable<Text>,
            address_line2 -> Nullable<Text>,
            city -> Nullable<Text>,
            state -> Nullable<Text>,
            zip -> Nullable<Text>,
            country -> Text,
            phone -> Nullable<Text>,
            email -> Nullable<Text>,
            website -> Nullable<Text>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::SummaryType;

        iiz.call_ai_summaries (id, generated_at) {
            id -> Uuid,
            account_id -> Uuid,
            call_id -> Uuid,
            summary_type -> SummaryType,
            content -> Text,
            model -> Nullable<Text>,
            generated_at -> Timestamptz,
            created_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.call_annotations (call_id) {
            call_id -> Uuid,
            account_id -> Uuid,
            score -> Nullable<Int4>,
            converted -> Nullable<Bool>,
            outcome -> Nullable<Text>,
            reporting_tag -> Nullable<Text>,
            category -> Nullable<Text>,
            appointment_set -> Nullable<Bool>,
            notes -> Nullable<Text>,
            updated_at -> Timestamptz,
            updated_by_id -> Nullable<Uuid>,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.call_daily_summary (id) {
            id -> Uuid,
            account_id -> Uuid,
            summary_date -> Date,
            source_id -> Nullable<Uuid>,
            agent_id -> Nullable<Uuid>,
            queue_id -> Nullable<Uuid>,
            total_calls -> Int4,
            answered_calls -> Int4,
            missed_calls -> Int4,
            voicemail_calls -> Int4,
            total_duration_secs -> Int4,
            total_ring_duration_secs -> Int4,
            total_hold_duration_secs -> Int4,
            avg_duration_secs -> Nullable<Numeric>,
            avg_ring_duration_secs -> Nullable<Numeric>,
            unique_callers -> Int4,
            first_time_callers -> Int4,
            repeat_callers -> Int4,
            converted_calls -> Int4,
            appointments_set -> Int4,
            computed_at -> Timestamptz,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.call_flow_events (id, occurred_at) {
            id -> Uuid,
            account_id -> Uuid,
            call_id -> Uuid,
            event_type -> Text,
            occurred_at -> Timestamptz,
            detail -> Nullable<Text>,
            created_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::SpeakerType;

        iiz.call_keyword_hits (id) {
            id -> Uuid,
            account_id -> Uuid,
            call_id -> Uuid,
            keyword_id -> Nullable<Uuid>,
            timestamp_offset_secs -> Nullable<Float4>,
            speaker -> Nullable<SpeakerType>,
            created_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::CallDirection;
        use super::sql_types::CallStatus;

        iiz.call_records (id, started_at) {
            id -> Uuid,
            account_id -> Uuid,
            call_id -> Text,
            caller_phone -> Nullable<Text>,
            callee_phone -> Nullable<Text>,
            direction -> CallDirection,
            status -> CallStatus,
            source_id -> Nullable<Uuid>,
            source_number_id -> Nullable<Uuid>,
            agent_id -> Nullable<Uuid>,
            queue_id -> Nullable<Uuid>,
            started_at -> Timestamptz,
            answered_at -> Nullable<Timestamptz>,
            ended_at -> Nullable<Timestamptz>,
            duration_secs -> Int4,
            ring_duration_secs -> Int4,
            hold_duration_secs -> Int4,
            recording_url -> Nullable<Text>,
            has_audio -> Bool,
            is_first_time_caller -> Bool,
            location -> Nullable<Text>,
            automation_id -> Nullable<Uuid>,
            source_name -> Nullable<Text>,
            agent_name -> Nullable<Text>,
            queue_name -> Nullable<Text>,
            created_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.call_settings (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Text,
            is_default -> Bool,
            greeting_enabled -> Bool,
            whisper_enabled -> Bool,
            inbound_recording -> Bool,
            outbound_recording -> Bool,
            transcription_enabled -> Bool,
            caller_id_enabled -> Bool,
            enhanced_caller_id -> Bool,
            caller_id_override -> Bool,
            spam_filter_enabled -> Bool,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.call_tags (id) {
            id -> Uuid,
            account_id -> Uuid,
            call_id -> Uuid,
            tag_id -> Uuid,
            applied_at -> Timestamptz,
            applied_by_type -> Text,
            applied_by_id -> Nullable<Uuid>,
            trigger_id -> Nullable<Uuid>,
            created_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::SpeakerType;

        iiz.call_transcription_segments (id, created_at) {
            id -> Uuid,
            account_id -> Uuid,
            call_id -> Uuid,
            segment_index -> Int4,
            timestamp_offset_secs -> Nullable<Float4>,
            speaker -> SpeakerType,
            content -> Text,
            confidence -> Nullable<Float4>,
            created_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.call_visitor_sessions (id) {
            id -> Uuid,
            account_id -> Uuid,
            call_id -> Uuid,
            ip_address -> Nullable<Text>,
            device -> Nullable<Text>,
            browser -> Nullable<Text>,
            os -> Nullable<Text>,
            referrer -> Nullable<Text>,
            landing_page -> Nullable<Text>,
            keywords -> Nullable<Text>,
            campaign -> Nullable<Text>,
            utm_source -> Nullable<Text>,
            utm_medium -> Nullable<Text>,
            utm_content -> Nullable<Text>,
            utm_term -> Nullable<Text>,
            visit_duration_secs -> Nullable<Int4>,
            pages_viewed -> Nullable<Int4>,
            created_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.caller_id_cnam (id) {
            id -> Uuid,
            account_id -> Uuid,
            number -> Text,
            tracking_number_id -> Nullable<Uuid>,
            current_cnam -> Nullable<Text>,
            requested_cnam -> Nullable<Text>,
            status -> Text,
            last_updated_at -> Nullable<Timestamptz>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.chat_ai_agents (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Text,
            instructions -> Nullable<Text>,
            knowledge_bank_id -> Nullable<Uuid>,
            welcome_message -> Nullable<Text>,
            max_turns -> Int4,
            handoff_threshold -> Nullable<Text>,
            handoff_queue_id -> Nullable<Uuid>,
            is_active -> Bool,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.chat_ai_configs (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Text,
            knowledge_bank_id -> Nullable<Uuid>,
            instructions -> Nullable<Text>,
            max_turns -> Int4,
            handoff_threshold -> Nullable<Text>,
            crm_integration_enabled -> Bool,
            crm_type -> Nullable<Text>,
            crm_config -> Nullable<Jsonb>,
            is_active -> Bool,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::ChannelType;

        iiz.chat_records (id) {
            id -> Uuid,
            account_id -> Uuid,
            visitor_name -> Nullable<Text>,
            visitor_detail -> Nullable<Text>,
            channel -> Nullable<ChannelType>,
            message_count -> Int4,
            agent_id -> Nullable<Uuid>,
            widget_id -> Nullable<Uuid>,
            status -> Text,
            duration_secs -> Int4,
            started_at -> Timestamptz,
            ended_at -> Nullable<Timestamptz>,
            created_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.chat_widgets (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Text,
            website_url -> Nullable<Text>,
            tracking_number_id -> Nullable<Uuid>,
            routing_type -> Nullable<Text>,
            queue_id -> Nullable<Uuid>,
            agent_count -> Int4,
            custom_fields_count -> Int4,
            status -> Text,
            config_json -> Nullable<Jsonb>,
            chat_count -> Int4,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.compliance_addresses (id) {
            id -> Uuid,
            account_id -> Uuid,
            label -> Nullable<Text>,
            address_line1 -> Text,
            address_line2 -> Nullable<Text>,
            city -> Text,
            state -> Text,
            zip -> Text,
            country -> Text,
            is_verified -> Bool,
            verification_method -> Nullable<Text>,
            verified_at -> Nullable<Timestamptz>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::ComplianceStatus;

        iiz.compliance_applications (id) {
            id -> Uuid,
            account_id -> Uuid,
            application_name -> Text,
            application_type -> Nullable<Text>,
            country -> Text,
            status -> ComplianceStatus,
            submitted_at -> Nullable<Timestamptz>,
            reviewed_at -> Nullable<Timestamptz>,
            expires_at -> Nullable<Timestamptz>,
            rejection_reason -> Nullable<Text>,
            external_reference_id -> Nullable<Text>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::ComplianceStatus;

        iiz.compliance_requirements (id) {
            id -> Uuid,
            account_id -> Uuid,
            country -> Text,
            requirement_name -> Text,
            requirement_description -> Nullable<Text>,
            status -> ComplianceStatus,
            documentation_url -> Nullable<Text>,
            due_date -> Nullable<Date>,
            completed_at -> Nullable<Timestamptz>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.contact_list_members (id) {
            id -> Uuid,
            account_id -> Uuid,
            list_id -> Uuid,
            phone -> Text,
            contact_name -> Nullable<Text>,
            added_at -> Timestamptz,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.contact_lists (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Text,
            description -> Nullable<Text>,
            member_count -> Int4,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.custom_reports (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Text,
            report_type -> Nullable<Text>,
            #[sql_name = "columns"]
            columns_ -> Nullable<Jsonb>,
            filters -> Nullable<Jsonb>,
            date_range_type -> Text,
            custom_start_date -> Nullable<Date>,
            custom_end_date -> Nullable<Date>,
            sort_column -> Nullable<Text>,
            sort_direction -> Nullable<Text>,
            schedule -> Nullable<Text>,
            schedule_recipients -> Nullable<Jsonb>,
            last_run_at -> Nullable<Timestamptz>,
            created_by_id -> Nullable<Uuid>,
            is_shared -> Bool,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.dialogflow_configs (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Text,
            project_id -> Nullable<Text>,
            service_account_json -> Nullable<Text>,
            language -> Nullable<Text>,
            default_intent -> Nullable<Text>,
            fallback_message -> Nullable<Text>,
            connection_status -> Text,
            last_tested_at -> Nullable<Timestamptz>,
            is_active -> Bool,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.dnc_entries (id) {
            id -> Uuid,
            account_id -> Uuid,
            number -> Text,
            added_by_id -> Nullable<Uuid>,
            reason -> Nullable<Text>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.dnt_entries (id) {
            id -> Uuid,
            account_id -> Uuid,
            number -> Text,
            e164 -> Text,
            rejected_count -> Int4,
            last_rejected_at -> Nullable<Timestamptz>,
            added_by_id -> Nullable<Uuid>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::ExportFormat;

        iiz.export_records (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Nullable<Text>,
            export_type -> Nullable<Text>,
            format -> ExportFormat,
            date_range -> Nullable<Text>,
            record_count -> Int4,
            status -> Text,
            download_url -> Nullable<Text>,
            requested_by_id -> Nullable<Uuid>,
            filters_applied -> Nullable<Jsonb>,
            completed_at -> Nullable<Timestamptz>,
            created_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::CallDirection;

        iiz.fax_records (id) {
            id -> Uuid,
            account_id -> Uuid,
            from_number -> Nullable<Text>,
            to_number -> Nullable<Text>,
            direction -> CallDirection,
            pages -> Int4,
            status -> Text,
            document_url -> Nullable<Text>,
            sent_at -> Timestamptz,
            created_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.form_reactor_entries (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Text,
            form_fields -> Nullable<Text>,
            tracking_number_id -> Nullable<Uuid>,
            call_count -> Int4,
            status -> Text,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.form_records (id) {
            id -> Uuid,
            account_id -> Uuid,
            contact_name -> Nullable<Text>,
            contact_phone -> Nullable<Text>,
            contact_email -> Nullable<Text>,
            form_name -> Nullable<Text>,
            source -> Nullable<Text>,
            tracking_number -> Nullable<Text>,
            form_data -> Nullable<Jsonb>,
            status -> Text,
            submitted_at -> Timestamptz,
            created_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.frequency_limits (id) {
            id -> Uuid,
            account_id -> Uuid,
            policy_id -> Text,
            scope -> Text,
            limit_type -> Text,
            max_count -> Int4,
            current_count -> Int4,
            window_start -> Nullable<Timestamptz>,
            window_end -> Nullable<Timestamptz>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.geo_router_rules (id) {
            id -> Uuid,
            account_id -> Uuid,
            router_id -> Uuid,
            region -> Text,
            region_type -> Text,
            destination_type -> Nullable<Text>,
            destination_id -> Nullable<Uuid>,
            destination_number -> Nullable<Text>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.geo_routers (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Text,
            default_destination_type -> Nullable<Text>,
            default_destination_id -> Nullable<Uuid>,
            default_destination_number -> Nullable<Text>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.keyword_spotting_configs (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Text,
            sensitivity -> Text,
            apply_to_all_numbers -> Bool,
            is_active -> Bool,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.keyword_spotting_keywords (id) {
            id -> Uuid,
            account_id -> Uuid,
            config_id -> Uuid,
            keyword -> Text,
            category -> Text,
            score_weight -> Float4,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.keyword_spotting_numbers (id) {
            id -> Uuid,
            account_id -> Uuid,
            config_id -> Uuid,
            tracking_number_id -> Uuid,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.knowledge_bank_documents (id) {
            id -> Uuid,
            account_id -> Uuid,
            bank_id -> Uuid,
            filename -> Text,
            file_type -> Text,
            source_url -> Nullable<Text>,
            file_ref -> Nullable<Text>,
            content_hash -> Nullable<Text>,
            file_size_bytes -> Int8,
            page_count -> Nullable<Int4>,
            chunk_count -> Int4,
            embedding_status -> Text,
            embedding_model -> Nullable<Text>,
            error_message -> Nullable<Text>,
            indexed_at -> Nullable<Timestamptz>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::Vector;

        iiz.knowledge_bank_embeddings (id) {
            id -> Uuid,
            account_id -> Uuid,
            document_id -> Uuid,
            chunk_index -> Int4,
            chunk_text -> Text,
            embedding -> Nullable<Vector>,
            created_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.knowledge_banks (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Text,
            category -> Text,
            document_count -> Int4,
            total_size_bytes -> Int8,
            status -> Text,
            last_import_at -> Nullable<Timestamptz>,
            used_by -> Nullable<Text>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.lambda_env_vars (id) {
            id -> Uuid,
            account_id -> Uuid,
            lambda_id -> Uuid,
            key -> Text,
            value -> Nullable<Text>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.lambdas (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Text,
            runtime -> Text,
            code -> Text,
            handler -> Text,
            timeout_ms -> Int4,
            memory_mb -> Int4,
            last_invoked_at -> Nullable<Timestamptz>,
            invocation_count -> Int4,
            error_count -> Int4,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.lead_reactor_actions (id) {
            id -> Uuid,
            account_id -> Uuid,
            config_id -> Uuid,
            sort_order -> Int4,
            action_type -> Text,
            template_content -> Nullable<Text>,
            action_config -> Nullable<Jsonb>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.lead_reactor_configs (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Text,
            trigger_event -> Text,
            delay_minutes -> Int4,
            is_active -> Bool,
            working_hours_only -> Bool,
            max_retries -> Int4,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::SipTransport;

        iiz.locations (id) {
            id -> Uuid,
            account_id -> Nullable<Uuid>,
            aor -> Text,
            username -> Nullable<Text>,
            realm -> Nullable<Text>,
            destination -> Text,
            expires -> Timestamptz,
            user_agent -> Nullable<Text>,
            supports_webrtc -> Bool,
            source_ip -> Nullable<Text>,
            source_port -> Nullable<Int4>,
            transport -> Nullable<SipTransport>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::MonitorMode;

        iiz.monitoring_events (id) {
            id -> Uuid,
            account_id -> Uuid,
            session_id -> Nullable<Text>,
            call_id -> Nullable<Uuid>,
            monitor_user_id -> Uuid,
            monitored_agent_id -> Nullable<Uuid>,
            event_type -> Text,
            monitor_mode -> MonitorMode,
            started_at -> Timestamptz,
            ended_at -> Nullable<Timestamptz>,
            duration_secs -> Nullable<Int4>,
            created_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.notification_rules (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Text,
            metric -> Text,
            condition_operator -> Text,
            threshold_value -> Numeric,
            time_window_minutes -> Int4,
            notification_method -> Text,
            recipients -> Nullable<Jsonb>,
            cooldown_minutes -> Int4,
            is_active -> Bool,
            last_triggered_at -> Nullable<Timestamptz>,
            trigger_count -> Int4,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.notifications (id) {
            id -> Uuid,
            account_id -> Uuid,
            user_id -> Uuid,
            event_type -> Text,
            title -> Text,
            body -> Nullable<Text>,
            entity_type -> Nullable<Text>,
            entity_id -> Nullable<Uuid>,
            is_read -> Bool,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.number_pool_members (id) {
            id -> Uuid,
            account_id -> Uuid,
            pool_id -> Uuid,
            tracking_number_id -> Uuid,
            status -> Text,
            call_count -> Int4,
            added_at -> Timestamptz,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.number_pools (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Text,
            description -> Nullable<Text>,
            source_id -> Nullable<Uuid>,
            auto_manage -> Bool,
            target_accuracy -> Int4,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::ComplianceStatus;

        iiz.port_requests (id) {
            id -> Uuid,
            account_id -> Uuid,
            numbers_to_port -> Jsonb,
            first_name -> Nullable<Text>,
            last_name -> Nullable<Text>,
            email -> Nullable<Text>,
            phone -> Nullable<Text>,
            billing_address_line1 -> Nullable<Text>,
            billing_address_line2 -> Nullable<Text>,
            city -> Nullable<Text>,
            state -> Nullable<Text>,
            zip -> Nullable<Text>,
            authorized_signature -> Nullable<Text>,
            status -> ComplianceStatus,
            submitted_at -> Nullable<Timestamptz>,
            completed_at -> Nullable<Timestamptz>,
            rejection_reason -> Nullable<Text>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::AgentStatus;

        iiz.presence (identity) {
            identity -> Text,
            account_id -> Nullable<Uuid>,
            user_id -> Nullable<Uuid>,
            status -> AgentStatus,
            note -> Nullable<Text>,
            activity -> Nullable<Text>,
            current_call_id -> Nullable<Uuid>,
            last_updated -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.queue_agents (id) {
            id -> Uuid,
            account_id -> Uuid,
            queue_id -> Uuid,
            agent_id -> Uuid,
            priority -> Int4,
            is_active -> Bool,
            added_at -> Timestamptz,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::QueueStrategy;

        iiz.queues (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Text,
            description -> Nullable<Text>,
            strategy -> QueueStrategy,
            schedule_id -> Nullable<Uuid>,
            repeat_callers -> Bool,
            caller_id_display -> Nullable<Text>,
            max_wait_secs -> Int4,
            no_answer_destination_type -> Nullable<Text>,
            no_answer_destination_id -> Nullable<Uuid>,
            no_answer_destination_number -> Nullable<Text>,
            moh_audio_url -> Nullable<Text>,
            wrap_up_secs -> Int4,
            is_active -> Bool,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.receiving_numbers (id) {
            id -> Uuid,
            account_id -> Uuid,
            number -> Text,
            description -> Nullable<Text>,
            tracking_count -> Int4,
            total_calls -> Int4,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.reminders (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Nullable<Text>,
            timezone -> Nullable<Text>,
            remind_at -> Nullable<Timestamptz>,
            is_recurring -> Bool,
            recurrence_rule -> Nullable<Text>,
            contact_source -> Nullable<Text>,
            contact_phone -> Nullable<Text>,
            contact_list_id -> Nullable<Uuid>,
            delivery_method -> Text,
            recipient -> Nullable<Text>,
            message -> Nullable<Text>,
            status -> Text,
            call_id -> Nullable<Uuid>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.routing_table_routes (id) {
            id -> Uuid,
            account_id -> Uuid,
            table_id -> Uuid,
            priority -> Int4,
            match_pattern -> Nullable<Text>,
            destination_type -> Nullable<Text>,
            destination_id -> Nullable<Uuid>,
            destination_number -> Nullable<Text>,
            weight -> Int4,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.routing_tables (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Text,
            is_active -> Bool,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.schedule_holidays (id) {
            id -> Uuid,
            account_id -> Uuid,
            schedule_id -> Uuid,
            date -> Date,
            name -> Text,
            is_closed -> Bool,
            custom_open -> Nullable<Time>,
            custom_close -> Nullable<Time>,
            override_destination_type -> Nullable<Text>,
            override_destination_id -> Nullable<Uuid>,
            override_destination_number -> Nullable<Text>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.schedules (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Text,
            timezone -> Text,
            monday_open -> Nullable<Time>,
            monday_close -> Nullable<Time>,
            tuesday_open -> Nullable<Time>,
            tuesday_close -> Nullable<Time>,
            wednesday_open -> Nullable<Time>,
            wednesday_close -> Nullable<Time>,
            thursday_open -> Nullable<Time>,
            thursday_close -> Nullable<Time>,
            friday_open -> Nullable<Time>,
            friday_close -> Nullable<Time>,
            saturday_open -> Nullable<Time>,
            saturday_close -> Nullable<Time>,
            sunday_open -> Nullable<Time>,
            sunday_close -> Nullable<Time>,
            closed_destination_type -> Nullable<Text>,
            closed_destination_id -> Nullable<Uuid>,
            closed_destination_number -> Nullable<Text>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.scoring_configs (id) {
            id -> Uuid,
            account_id -> Uuid,
            answer_rate_weight -> Int4,
            talk_time_weight -> Int4,
            conversion_weight -> Int4,
            min_talk_time_secs -> Int4,
            target_answer_rate -> Int4,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.smart_dialer_configs (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Text,
            mode -> Text,
            max_concurrent -> Int4,
            ring_timeout_secs -> Int4,
            retry_attempts -> Int4,
            retry_interval_minutes -> Int4,
            outbound_number -> Nullable<Text>,
            outbound_cnam -> Nullable<Text>,
            start_time -> Nullable<Time>,
            end_time -> Nullable<Time>,
            timezone -> Nullable<Text>,
            active_days -> Int4,
            contact_list_id -> Nullable<Uuid>,
            agent_script_id -> Nullable<Uuid>,
            is_active -> Bool,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.smart_router_rules (id) {
            id -> Uuid,
            account_id -> Uuid,
            router_id -> Uuid,
            sort_order -> Int4,
            condition_field -> Text,
            condition_operator -> Text,
            condition_value -> Text,
            destination_type -> Nullable<Text>,
            destination_id -> Nullable<Uuid>,
            destination_number -> Nullable<Text>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.smart_routers (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Text,
            priority -> Int4,
            fallback_destination_type -> Nullable<Text>,
            fallback_destination_id -> Nullable<Uuid>,
            fallback_destination_number -> Nullable<Text>,
            is_active -> Bool,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.summary_configs (id) {
            id -> Uuid,
            account_id -> Uuid,
            phone_enabled -> Bool,
            video_enabled -> Bool,
            chat_enabled -> Bool,
            enabled_summary_types -> Nullable<Jsonb>,
            transcribe_all -> Bool,
            transcription_language -> Nullable<Text>,
            pii_redaction_enabled -> Bool,
            pii_redaction_rules -> Nullable<Text>,
            default_model -> Nullable<Text>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.tags (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Text,
            color -> Nullable<Text>,
            description -> Nullable<Text>,
            usage_count -> Int4,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.target_numbers (id) {
            id -> Uuid,
            account_id -> Uuid,
            number -> Text,
            name -> Text,
            description -> Nullable<Text>,
            target_type -> Text,
            priority -> Int4,
            concurrency_cap -> Nullable<Int4>,
            weight -> Int4,
            status -> Text,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::CallDirection;

        iiz.text_messages (id, sent_at) {
            id -> Uuid,
            account_id -> Uuid,
            contact_phone -> Nullable<Text>,
            tracking_number_id -> Nullable<Uuid>,
            call_id -> Nullable<Uuid>,
            direction -> CallDirection,
            body -> Text,
            status -> Text,
            sent_at -> Timestamptz,
            created_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.text_numbers (id) {
            id -> Uuid,
            account_id -> Uuid,
            number -> Text,
            name -> Nullable<Text>,
            is_assigned -> Bool,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::CallDirection;

        iiz.text_records (id) {
            id -> Uuid,
            account_id -> Uuid,
            contact_phone -> Nullable<Text>,
            tracking_number_id -> Nullable<Uuid>,
            direction -> CallDirection,
            preview -> Nullable<Text>,
            status -> Text,
            sent_at -> Timestamptz,
            created_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::ComplianceStatus;

        iiz.toll_free_registrations (id) {
            id -> Uuid,
            account_id -> Uuid,
            business_name -> Nullable<Text>,
            contact_name -> Nullable<Text>,
            contact_phone -> Nullable<Text>,
            contact_email -> Nullable<Text>,
            use_case -> Nullable<Text>,
            use_case_description -> Nullable<Text>,
            monthly_volume -> Nullable<Text>,
            toll_free_numbers -> Nullable<Jsonb>,
            status -> ComplianceStatus,
            rejection_reason -> Nullable<Text>,
            submitted_at -> Nullable<Timestamptz>,
            approved_at -> Nullable<Timestamptz>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::NumberType;
        use super::sql_types::NumberClass;

        iiz.tracking_numbers (id) {
            id -> Uuid,
            account_id -> Uuid,
            number -> Text,
            source_id -> Nullable<Uuid>,
            routing_description -> Nullable<Text>,
            routing_type -> Nullable<Text>,
            routing_target_type -> Nullable<Text>,
            routing_target_id -> Nullable<Uuid>,
            text_enabled -> Bool,
            receiving_number_id -> Nullable<Uuid>,
            number_type -> NumberType,
            number_class -> NumberClass,
            pool_id -> Nullable<Uuid>,
            billing_date -> Nullable<Int4>,
            is_active -> Bool,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.tracking_sources (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Text,
            source_type -> Nullable<Text>,
            position -> Int4,
            last_touch -> Bool,
            number_count -> Int4,
            call_count -> Int4,
            status -> Text,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.trigger_actions (id) {
            id -> Uuid,
            account_id -> Uuid,
            trigger_id -> Uuid,
            sort_order -> Int4,
            action_type -> Text,
            action_config -> Nullable<Jsonb>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.trigger_conditions (id) {
            id -> Uuid,
            account_id -> Uuid,
            trigger_id -> Uuid,
            sort_order -> Int4,
            field -> Text,
            operator -> Text,
            value -> Text,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.triggers (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Text,
            trigger_event -> Text,
            run_on -> Nullable<Text>,
            runs_7d -> Int4,
            status -> Text,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::UserRole;

        iiz.users (id) {
            id -> Uuid,
            account_id -> Uuid,
            username -> Text,
            email -> Text,
            password_hash -> Text,
            display_name -> Nullable<Text>,
            initials -> Nullable<Text>,
            avatar_color -> Nullable<Text>,
            role -> UserRole,
            phone -> Nullable<Text>,
            is_active -> Bool,
            reset_token -> Nullable<Text>,
            reset_token_expires -> Nullable<Timestamptz>,
            last_login_at -> Nullable<Timestamptz>,
            last_login_ip -> Nullable<Text>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.video_records (id) {
            id -> Uuid,
            account_id -> Uuid,
            participant_name -> Nullable<Text>,
            participant_email -> Nullable<Text>,
            host_agent_id -> Nullable<Uuid>,
            platform -> Nullable<Text>,
            has_recording -> Bool,
            recording_url -> Nullable<Text>,
            duration_secs -> Int4,
            started_at -> Timestamptz,
            ended_at -> Nullable<Timestamptz>,
            created_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.voice_ai_agents (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Text,
            welcome_message -> Nullable<Text>,
            instructions -> Nullable<Text>,
            voice -> Nullable<Text>,
            language -> Nullable<Text>,
            knowledge_bank_id -> Nullable<Uuid>,
            max_turns -> Int4,
            handoff_threshold -> Nullable<Text>,
            handoff_destination_type -> Nullable<Text>,
            handoff_destination_id -> Nullable<Uuid>,
            is_active -> Bool,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.voice_menu_options (id) {
            id -> Uuid,
            account_id -> Uuid,
            menu_id -> Uuid,
            dtmf_digit -> Text,
            description -> Nullable<Text>,
            destination_type -> Nullable<Text>,
            destination_id -> Nullable<Uuid>,
            destination_number -> Nullable<Text>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::GreetingType;

        iiz.voice_menus (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Text,
            greeting_type -> GreetingType,
            greeting_audio_url -> Nullable<Text>,
            greeting_text -> Nullable<Text>,
            speech_recognition -> Bool,
            speech_language -> Nullable<Text>,
            timeout_secs -> Int4,
            max_retries -> Int4,
            no_input_destination_type -> Nullable<Text>,
            no_input_destination_id -> Nullable<Uuid>,
            no_input_destination_number -> Nullable<Text>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.voice_registration_history (id) {
            id -> Uuid,
            account_id -> Uuid,
            registration_id -> Uuid,
            event_date -> Date,
            event_type -> Text,
            old_status -> Nullable<Text>,
            new_status -> Nullable<Text>,
            notes -> Nullable<Text>,
            created_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::ComplianceStatus;
        use super::sql_types::AttestationLevel;

        iiz.voice_registrations (id) {
            id -> Uuid,
            account_id -> Uuid,
            business_name -> Nullable<Text>,
            ein -> Nullable<Text>,
            address_line1 -> Nullable<Text>,
            address_line2 -> Nullable<Text>,
            city -> Nullable<Text>,
            state -> Nullable<Text>,
            zip -> Nullable<Text>,
            status -> ComplianceStatus,
            attestation_level -> Nullable<AttestationLevel>,
            last_verified_at -> Nullable<Timestamptz>,
            next_verification_due -> Nullable<Date>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::GreetingType;

        iiz.voicemail_boxes (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Text,
            max_message_length_secs -> Int4,
            greeting_type -> GreetingType,
            greeting_audio_url -> Nullable<Text>,
            transcription_enabled -> Bool,
            email_notification_enabled -> Bool,
            notification_email -> Nullable<Text>,
            max_messages -> Int4,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.voicemail_messages (id) {
            id -> Uuid,
            account_id -> Uuid,
            mailbox_id -> Uuid,
            call_id -> Nullable<Uuid>,
            caller_number -> Nullable<Text>,
            caller_name -> Nullable<Text>,
            duration_secs -> Int4,
            audio_url -> Nullable<Text>,
            transcription -> Nullable<Text>,
            is_read -> Bool,
            recorded_at -> Timestamptz,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.webhook_deliveries (id, delivered_at) {
            id -> Uuid,
            account_id -> Uuid,
            webhook_id -> Uuid,
            event_type -> Text,
            payload -> Nullable<Jsonb>,
            http_status_code -> Nullable<Int4>,
            response_body -> Nullable<Text>,
            status -> Text,
            attempt_number -> Int4,
            delivered_at -> Timestamptz,
            created_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.webhook_subscriptions (id) {
            id -> Uuid,
            account_id -> Uuid,
            webhook_id -> Uuid,
            event_type -> Text,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.webhooks (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Text,
            trigger_event -> Nullable<Text>,
            callback_url -> Text,
            method -> Text,
            body_type -> Text,
            headers -> Nullable<Jsonb>,
            secret -> Nullable<Text>,
            retry_count -> Int4,
            retry_delay_secs -> Int4,
            status -> Text,
            last_triggered_at -> Nullable<Timestamptz>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.workflow_edges (id) {
            id -> Uuid,
            account_id -> Uuid,
            workflow_id -> Uuid,
            from_node_id -> Uuid,
            to_node_id -> Uuid,
            label -> Nullable<Text>,
            sort_order -> Int4,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::WorkflowNodeType;

        iiz.workflow_nodes (id) {
            id -> Uuid,
            account_id -> Uuid,
            workflow_id -> Uuid,
            node_type -> WorkflowNodeType,
            event_type -> Nullable<Text>,
            action_type -> Nullable<Text>,
            condition_type -> Nullable<Text>,
            config_json -> Nullable<Jsonb>,
            label -> Nullable<Text>,
            position_x -> Nullable<Float4>,
            position_y -> Nullable<Float4>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        iiz.workflows (id) {
            id -> Uuid,
            account_id -> Uuid,
            name -> Text,
            canvas_json -> Nullable<Jsonb>,
            status -> Text,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            deleted_at -> Nullable<Timestamptz>,
        }
    }

    diesel::joinable!(a2p_campaigns -> accounts (account_id));
    diesel::joinable!(account_variables -> accounts (account_id));
    diesel::joinable!(agent_scripts -> accounts (account_id));
    diesel::joinable!(appointments -> accounts (account_id));
    diesel::joinable!(appointments -> users (agent_id));
    diesel::joinable!(ask_ai_configs -> accounts (account_id));
    diesel::joinable!(authorized_contacts -> accounts (account_id));
    diesel::joinable!(authorized_contacts -> business_info (business_info_id));
    diesel::joinable!(blocked_numbers -> accounts (account_id));
    diesel::joinable!(bulk_messages -> accounts (account_id));
    diesel::joinable!(bulk_messages -> contact_lists (contact_list_id));
    diesel::joinable!(business_info -> accounts (account_id));
    diesel::joinable!(call_daily_summary -> accounts (account_id));
    diesel::joinable!(call_settings -> accounts (account_id));
    diesel::joinable!(call_tags -> accounts (account_id));
    diesel::joinable!(call_tags -> tags (tag_id));
    diesel::joinable!(call_tags -> users (applied_by_id));
    diesel::joinable!(caller_id_cnam -> accounts (account_id));
    diesel::joinable!(chat_ai_agents -> accounts (account_id));
    diesel::joinable!(chat_ai_agents -> knowledge_banks (knowledge_bank_id));
    diesel::joinable!(chat_ai_configs -> accounts (account_id));
    diesel::joinable!(chat_ai_configs -> knowledge_banks (knowledge_bank_id));
    diesel::joinable!(chat_widgets -> accounts (account_id));
    diesel::joinable!(compliance_addresses -> accounts (account_id));
    diesel::joinable!(compliance_applications -> accounts (account_id));
    diesel::joinable!(compliance_requirements -> accounts (account_id));
    diesel::joinable!(contact_list_members -> accounts (account_id));
    diesel::joinable!(contact_list_members -> contact_lists (list_id));
    diesel::joinable!(contact_lists -> accounts (account_id));
    diesel::joinable!(custom_reports -> accounts (account_id));
    diesel::joinable!(custom_reports -> users (created_by_id));
    diesel::joinable!(dialogflow_configs -> accounts (account_id));
    diesel::joinable!(dnc_entries -> accounts (account_id));
    diesel::joinable!(dnc_entries -> users (added_by_id));
    diesel::joinable!(dnt_entries -> accounts (account_id));
    diesel::joinable!(dnt_entries -> users (added_by_id));
    diesel::joinable!(form_reactor_entries -> accounts (account_id));
    diesel::joinable!(geo_router_rules -> accounts (account_id));
    diesel::joinable!(geo_router_rules -> geo_routers (router_id));
    diesel::joinable!(geo_routers -> accounts (account_id));
    diesel::joinable!(keyword_spotting_configs -> accounts (account_id));
    diesel::joinable!(keyword_spotting_keywords -> accounts (account_id));
    diesel::joinable!(keyword_spotting_keywords -> keyword_spotting_configs (config_id));
    diesel::joinable!(keyword_spotting_numbers -> accounts (account_id));
    diesel::joinable!(keyword_spotting_numbers -> keyword_spotting_configs (config_id));
    diesel::joinable!(knowledge_bank_documents -> accounts (account_id));
    diesel::joinable!(knowledge_bank_documents -> knowledge_banks (bank_id));
    diesel::joinable!(knowledge_banks -> accounts (account_id));
    diesel::joinable!(lambda_env_vars -> accounts (account_id));
    diesel::joinable!(lambda_env_vars -> lambdas (lambda_id));
    diesel::joinable!(lambdas -> accounts (account_id));
    diesel::joinable!(lead_reactor_actions -> accounts (account_id));
    diesel::joinable!(lead_reactor_actions -> lead_reactor_configs (config_id));
    diesel::joinable!(lead_reactor_configs -> accounts (account_id));
    diesel::joinable!(notification_rules -> accounts (account_id));
    diesel::joinable!(number_pool_members -> accounts (account_id));
    diesel::joinable!(number_pool_members -> number_pools (pool_id));
    diesel::joinable!(number_pools -> accounts (account_id));
    diesel::joinable!(number_pools -> tracking_sources (source_id));
    diesel::joinable!(port_requests -> accounts (account_id));
    diesel::joinable!(queue_agents -> accounts (account_id));
    diesel::joinable!(queue_agents -> queues (queue_id));
    diesel::joinable!(queue_agents -> users (agent_id));
    diesel::joinable!(queues -> accounts (account_id));
    diesel::joinable!(queues -> schedules (schedule_id));
    diesel::joinable!(receiving_numbers -> accounts (account_id));
    diesel::joinable!(reminders -> accounts (account_id));
    diesel::joinable!(reminders -> contact_lists (contact_list_id));
    diesel::joinable!(routing_table_routes -> accounts (account_id));
    diesel::joinable!(routing_table_routes -> routing_tables (table_id));
    diesel::joinable!(routing_tables -> accounts (account_id));
    diesel::joinable!(schedule_holidays -> accounts (account_id));
    diesel::joinable!(schedule_holidays -> schedules (schedule_id));
    diesel::joinable!(schedules -> accounts (account_id));
    diesel::joinable!(scoring_configs -> accounts (account_id));
    diesel::joinable!(smart_dialer_configs -> accounts (account_id));
    diesel::joinable!(smart_dialer_configs -> agent_scripts (agent_script_id));
    diesel::joinable!(smart_dialer_configs -> contact_lists (contact_list_id));
    diesel::joinable!(smart_router_rules -> accounts (account_id));
    diesel::joinable!(smart_router_rules -> smart_routers (router_id));
    diesel::joinable!(smart_routers -> accounts (account_id));
    diesel::joinable!(summary_configs -> accounts (account_id));
    diesel::joinable!(tags -> accounts (account_id));
    diesel::joinable!(target_numbers -> accounts (account_id));
    diesel::joinable!(text_numbers -> accounts (account_id));
    diesel::joinable!(toll_free_registrations -> accounts (account_id));
    diesel::joinable!(tracking_numbers -> accounts (account_id));
    diesel::joinable!(tracking_numbers -> number_pools (pool_id));
    diesel::joinable!(tracking_numbers -> receiving_numbers (receiving_number_id));
    diesel::joinable!(tracking_numbers -> tracking_sources (source_id));
    diesel::joinable!(tracking_sources -> accounts (account_id));
    diesel::joinable!(trigger_actions -> accounts (account_id));
    diesel::joinable!(trigger_actions -> triggers (trigger_id));
    diesel::joinable!(trigger_conditions -> accounts (account_id));
    diesel::joinable!(trigger_conditions -> triggers (trigger_id));
    diesel::joinable!(triggers -> accounts (account_id));
    diesel::joinable!(users -> accounts (account_id));
    diesel::joinable!(voice_ai_agents -> accounts (account_id));
    diesel::joinable!(voice_ai_agents -> knowledge_banks (knowledge_bank_id));
    diesel::joinable!(voice_menu_options -> accounts (account_id));
    diesel::joinable!(voice_menu_options -> voice_menus (menu_id));
    diesel::joinable!(voice_menus -> accounts (account_id));
    diesel::joinable!(voice_registration_history -> accounts (account_id));
    diesel::joinable!(voice_registration_history -> voice_registrations (registration_id));
    diesel::joinable!(voice_registrations -> accounts (account_id));
    diesel::joinable!(voicemail_boxes -> accounts (account_id));
    diesel::joinable!(voicemail_messages -> accounts (account_id));
    diesel::joinable!(voicemail_messages -> voicemail_boxes (mailbox_id));
    diesel::joinable!(webhook_subscriptions -> accounts (account_id));
    diesel::joinable!(webhook_subscriptions -> webhooks (webhook_id));
    diesel::joinable!(webhooks -> accounts (account_id));
    diesel::joinable!(workflow_edges -> accounts (account_id));
    diesel::joinable!(workflow_edges -> workflows (workflow_id));
    diesel::joinable!(workflow_nodes -> accounts (account_id));
    diesel::joinable!(workflow_nodes -> workflows (workflow_id));
    diesel::joinable!(workflows -> accounts (account_id));

    diesel::allow_tables_to_appear_in_same_query!(
        a2p_campaigns,
        account_variables,
        accounts,
        active_calls,
        agent_scripts,
        agent_state_log,
        api_log_entries,
        appointments,
        ask_ai_configs,
        authorized_contacts,
        blocked_numbers,
        bulk_messages,
        business_info,
        call_ai_summaries,
        call_annotations,
        call_daily_summary,
        call_flow_events,
        call_keyword_hits,
        call_records,
        call_settings,
        call_tags,
        call_transcription_segments,
        call_visitor_sessions,
        caller_id_cnam,
        chat_ai_agents,
        chat_ai_configs,
        chat_records,
        chat_widgets,
        compliance_addresses,
        compliance_applications,
        compliance_requirements,
        contact_list_members,
        contact_lists,
        custom_reports,
        dialogflow_configs,
        dnc_entries,
        dnt_entries,
        export_records,
        fax_records,
        form_reactor_entries,
        form_records,
        frequency_limits,
        geo_router_rules,
        geo_routers,
        keyword_spotting_configs,
        keyword_spotting_keywords,
        keyword_spotting_numbers,
        knowledge_bank_documents,
        knowledge_bank_embeddings,
        knowledge_banks,
        lambda_env_vars,
        lambdas,
        lead_reactor_actions,
        lead_reactor_configs,
        locations,
        monitoring_events,
        notification_rules,
        notifications,
        number_pool_members,
        number_pools,
        port_requests,
        presence,
        queue_agents,
        queues,
        receiving_numbers,
        reminders,
        routing_table_routes,
        routing_tables,
        schedule_holidays,
        schedules,
        scoring_configs,
        smart_dialer_configs,
        smart_router_rules,
        smart_routers,
        summary_configs,
        tags,
        target_numbers,
        text_messages,
        text_numbers,
        text_records,
        toll_free_registrations,
        tracking_numbers,
        tracking_sources,
        trigger_actions,
        trigger_conditions,
        triggers,
        users,
        video_records,
        voice_ai_agents,
        voice_menu_options,
        voice_menus,
        voice_registration_history,
        voice_registrations,
        voicemail_boxes,
        voicemail_messages,
        webhook_deliveries,
        webhook_subscriptions,
        webhooks,
        workflow_edges,
        workflow_nodes,
        workflows,
    );
}
