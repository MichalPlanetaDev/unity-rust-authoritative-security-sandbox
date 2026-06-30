use protocol::{
    InputCommand, Milliseconds, PlayerId, SuspicionKind, SuspicionReport, TelemetryEvent, Vec2,
};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValidationPolicy {
    pub max_speed_units_per_second: f32,
    pub movement_tolerance_units: f32,
    pub fixed_tick_ms: Milliseconds,
    pub fire_cooldown_ms: Milliseconds,
    pub max_client_time_step_ms: Milliseconds,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerValidationState {
    pub position: Vec2,
    pub alive: bool,
    pub last_sequence: u64,
    pub last_client_time_ms: Option<Milliseconds>,
    pub next_allowed_fire_time_ms: Milliseconds,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidationDecision {
    pub accepted: bool,
    pub rejection_reason: Option<String>,
    pub reports: Vec<SuspicionReport>,
}

impl ValidationDecision {
    fn accepted() -> Self {
        Self {
            accepted: true,
            rejection_reason: None,
            reports: Vec::new(),
        }
    }

    fn rejected(reason: impl Into<String>, report: SuspicionReport) -> Self {
        Self {
            accepted: false,
            rejection_reason: Some(reason.into()),
            reports: vec![report],
        }
    }

    pub fn has_kind(&self, kind: SuspicionKind) -> bool {
        self.reports.iter().any(|report| report.kind == kind)
    }
}

pub fn validate_input(
    command: &InputCommand,
    state: PlayerValidationState,
    policy: ValidationPolicy,
    server_time_ms: Milliseconds,
) -> ValidationDecision {
    if command.sequence <= state.last_sequence {
        return ValidationDecision::rejected(
            "command sequence number did not increase",
            SuspicionReport::new(
                command.player_id,
                command.sequence,
                SuspicionKind::PacketSequenceViolation,
                "command sequence number did not increase",
                command.sequence as f32,
                (state.last_sequence + 1) as f32,
                server_time_ms,
            ),
        );
    }

    if !state.alive {
        return ValidationDecision::rejected(
            "dead player cannot send input",
            SuspicionReport::new(
                command.player_id,
                command.sequence,
                SuspicionKind::InvalidStateTransition,
                "dead player attempted to send input",
                1.0,
                0.0,
                server_time_ms,
            ),
        );
    }

    let mut decision = ValidationDecision::accepted();

    inspect_client_time(
        command,
        state.last_client_time_ms,
        policy.max_client_time_step_ms,
        server_time_ms,
        &mut decision.reports,
    );

    inspect_movement_claim(
        command,
        state.position,
        policy.max_speed_units_per_second,
        policy.fixed_tick_ms,
        policy.movement_tolerance_units,
        server_time_ms,
        &mut decision.reports,
    );

    inspect_fire_rate(
        command,
        state.next_allowed_fire_time_ms,
        server_time_ms,
        &mut decision.reports,
    );

    decision
}

fn inspect_client_time(
    command: &InputCommand,
    previous_client_time_ms: Option<Milliseconds>,
    max_client_time_step_ms: Milliseconds,
    server_time_ms: Milliseconds,
    reports: &mut Vec<SuspicionReport>,
) {
    let Some(previous_client_time_ms) = previous_client_time_ms else {
        return;
    };

    if command.client_time_ms <= previous_client_time_ms {
        reports.push(SuspicionReport::new(
            command.player_id,
            command.sequence,
            SuspicionKind::ClientTimeViolation,
            "client timestamp did not increase",
            command.client_time_ms as f32,
            (previous_client_time_ms + 1) as f32,
            server_time_ms,
        ));
        return;
    }

    let observed_step = command.client_time_ms - previous_client_time_ms;

    if observed_step > max_client_time_step_ms {
        reports.push(SuspicionReport::new(
            command.player_id,
            command.sequence,
            SuspicionKind::ClientTimeViolation,
            "client timestamp jumped too far forward",
            observed_step as f32,
            max_client_time_step_ms as f32,
            server_time_ms,
        ));
    }
}

fn inspect_movement_claim(
    command: &InputCommand,
    current_position: Vec2,
    max_speed_units_per_second: f32,
    fixed_tick_ms: Milliseconds,
    movement_tolerance_units: f32,
    server_time_ms: Milliseconds,
    reports: &mut Vec<SuspicionReport>,
) {
    let Some(claimed_position) = command.claimed_position else {
        return;
    };

    let observed_distance = current_position.distance(claimed_position);
    let allowed_distance =
        max_speed_units_per_second * (fixed_tick_ms as f32 / 1000.0) + movement_tolerance_units;

    if observed_distance > allowed_distance {
        reports.push(SuspicionReport::new(
            command.player_id,
            command.sequence,
            SuspicionKind::SpeedHack,
            "claimed position exceeded server movement budget",
            observed_distance,
            allowed_distance,
            server_time_ms,
        ));
    }
}

fn inspect_fire_rate(
    command: &InputCommand,
    next_allowed_fire_time_ms: Milliseconds,
    server_time_ms: Milliseconds,
    reports: &mut Vec<SuspicionReport>,
) {
    if !command.fire {
        return;
    }

    if server_time_ms < next_allowed_fire_time_ms {
        reports.push(SuspicionReport::new(
            command.player_id,
            command.sequence,
            SuspicionKind::FireRateViolation,
            "fire input arrived before cooldown expired",
            server_time_ms as f32,
            next_allowed_fire_time_ms as f32,
            server_time_ms,
        ));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum ViolationCode {
    SpeedHack,
    FireRateViolation,
    InvalidStateTransition,
    PacketSequenceViolation,
    ClientTimeViolation,
    ProtocolViolation,
    RateLimitViolation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EvidenceRecord {
    pub player_id: PlayerId,
    pub sequence: u64,
    pub violation_code: ViolationCode,
    pub severity: ViolationSeverity,
    pub reason: String,
    pub observed_value: f32,
    pub expected_limit: f32,
    pub server_time_ms: Milliseconds,
}

impl EvidenceRecord {
    pub fn from_suspicion(report: &SuspicionReport) -> Self {
        let violation_code = violation_code(report.kind);

        Self {
            player_id: report.player_id,
            sequence: report.sequence,
            violation_code,
            severity: severity(violation_code),
            reason: report.reason.clone(),
            observed_value: report.observed_value,
            expected_limit: report.expected_limit,
            server_time_ms: report.server_time_ms,
        }
    }
}

pub fn evidence_records_from_events(events: &[TelemetryEvent]) -> Vec<EvidenceRecord> {
    events
        .iter()
        .filter_map(|event| match event {
            TelemetryEvent::Suspicion(report) => Some(EvidenceRecord::from_suspicion(report)),
            _ => None,
        })
        .collect()
}

fn violation_code(kind: SuspicionKind) -> ViolationCode {
    match kind {
        SuspicionKind::SpeedHack => ViolationCode::SpeedHack,
        SuspicionKind::FireRateViolation => ViolationCode::FireRateViolation,
        SuspicionKind::InvalidStateTransition => ViolationCode::InvalidStateTransition,
        SuspicionKind::PacketSequenceViolation => ViolationCode::PacketSequenceViolation,
        SuspicionKind::ClientTimeViolation => ViolationCode::ClientTimeViolation,
        SuspicionKind::ProtocolViolation => ViolationCode::ProtocolViolation,
        SuspicionKind::RateLimitViolation => ViolationCode::RateLimitViolation,
    }
}

fn severity(code: ViolationCode) -> ViolationSeverity {
    match code {
        ViolationCode::SpeedHack => ViolationSeverity::High,
        ViolationCode::FireRateViolation => ViolationSeverity::Medium,
        ViolationCode::InvalidStateTransition => ViolationSeverity::High,
        ViolationCode::PacketSequenceViolation => ViolationSeverity::Medium,
        ViolationCode::ClientTimeViolation => ViolationSeverity::Medium,
        ViolationCode::ProtocolViolation => ViolationSeverity::High,
        ViolationCode::RateLimitViolation => ViolationSeverity::Medium,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> ValidationPolicy {
        ValidationPolicy {
            max_speed_units_per_second: 10.0,
            movement_tolerance_units: 0.15,
            fixed_tick_ms: 100,
            fire_cooldown_ms: 500,
            max_client_time_step_ms: 250,
        }
    }

    fn state() -> PlayerValidationState {
        PlayerValidationState {
            position: Vec2::ZERO,
            alive: true,
            last_sequence: 0,
            last_client_time_ms: None,
            next_allowed_fire_time_ms: 0,
        }
    }

    fn input(sequence: u64, client_time_ms: u64, claimed_position: Vec2) -> InputCommand {
        InputCommand {
            player_id: PlayerId(1),
            sequence,
            client_time_ms,
            movement: Vec2::new(1.0, 0.0),
            fire: false,
            claimed_position: Some(claimed_position),
        }
    }

    #[test]
    fn accepts_normal_input() {
        let decision = validate_input(&input(1, 100, Vec2::new(1.0, 0.0)), state(), policy(), 100);

        assert!(decision.accepted);
        assert!(decision.reports.is_empty());
    }

    #[test]
    fn flags_speed_violation() {
        let decision = validate_input(&input(1, 100, Vec2::new(25.0, 0.0)), state(), policy(), 100);

        assert!(decision.accepted);
        assert!(decision.has_kind(SuspicionKind::SpeedHack));
    }

    #[test]
    fn rejects_repeated_sequence() {
        let state = PlayerValidationState {
            last_sequence: 5,
            ..state()
        };

        let decision = validate_input(&input(5, 100, Vec2::new(1.0, 0.0)), state, policy(), 100);

        assert!(!decision.accepted);
        assert!(decision.has_kind(SuspicionKind::PacketSequenceViolation));
    }

    #[test]
    fn rejects_dead_player_input() {
        let state = PlayerValidationState {
            alive: false,
            ..state()
        };

        let decision = validate_input(&input(1, 100, Vec2::new(1.0, 0.0)), state, policy(), 100);

        assert!(!decision.accepted);
        assert!(decision.has_kind(SuspicionKind::InvalidStateTransition));
    }

    #[test]
    fn flags_backwards_client_time() {
        let state = PlayerValidationState {
            last_client_time_ms: Some(200),
            ..state()
        };

        let decision = validate_input(&input(1, 100, Vec2::new(1.0, 0.0)), state, policy(), 100);

        assert!(decision.accepted);
        assert!(decision.has_kind(SuspicionKind::ClientTimeViolation));
    }

    #[test]
    fn flags_large_client_time_jump() {
        let state = PlayerValidationState {
            last_client_time_ms: Some(100),
            ..state()
        };

        let decision = validate_input(&input(1, 5_000, Vec2::new(1.0, 0.0)), state, policy(), 100);

        assert!(decision.accepted);
        assert!(decision.has_kind(SuspicionKind::ClientTimeViolation));
    }

    #[test]
    fn flags_fire_rate_violation() {
        let state = PlayerValidationState {
            next_allowed_fire_time_ms: 1000,
            ..state()
        };

        let mut command = input(1, 100, Vec2::new(1.0, 0.0));
        command.fire = true;

        let decision = validate_input(&command, state, policy(), 500);

        assert!(decision.accepted);
        assert!(decision.has_kind(SuspicionKind::FireRateViolation));
    }

    #[test]
    fn builds_evidence_from_suspicion() {
        let report = SuspicionReport::new(
            PlayerId(2),
            7,
            SuspicionKind::SpeedHack,
            "test speed violation",
            20.0,
            1.15,
            900,
        );

        let evidence = EvidenceRecord::from_suspicion(&report);

        assert_eq!(evidence.player_id, PlayerId(2));
        assert_eq!(evidence.sequence, 7);
        assert_eq!(evidence.violation_code, ViolationCode::SpeedHack);
        assert_eq!(evidence.severity, ViolationSeverity::High);
    }
}
