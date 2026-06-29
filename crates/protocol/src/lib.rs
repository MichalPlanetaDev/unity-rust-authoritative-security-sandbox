use serde::{Deserialize, Serialize};

pub type SequenceNumber = u64;
pub type Milliseconds = u64;
pub type ConnectionId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PlayerId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn distance(self, other: Self) -> f32 {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
        .length()
    }

    pub fn normalized(self) -> Self {
        let length = self.length();

        if length <= f32::EPSILON {
            Self::ZERO
        } else {
            Self {
                x: self.x / length,
                y: self.y / length,
            }
        }
    }

    pub fn scaled(self, factor: f32) -> Self {
        Self {
            x: self.x * factor,
            y: self.y * factor,
        }
    }

    pub fn add_vector(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputCommand {
    pub player_id: PlayerId,
    pub sequence: SequenceNumber,
    pub client_time_ms: Milliseconds,
    pub movement: Vec2,
    pub fire: bool,
    pub claimed_position: Option<Vec2>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ClientMessage {
    Join { player_id: PlayerId },
    Input(InputCommand),
    Ping { client_time_ms: Milliseconds },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerSnapshot {
    pub player_id: PlayerId,
    pub position: Vec2,
    pub health: i32,
    pub alive: bool,
    pub server_time_ms: Milliseconds,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerMessage {
    Welcome {
        player_id: PlayerId,
    },
    Snapshot(PlayerSnapshot),
    Rejected {
        reason: String,
    },
    Pong {
        client_time_ms: Milliseconds,
        server_time_ms: Milliseconds,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SuspicionKind {
    SpeedHack,
    FireRateViolation,
    InvalidStateTransition,
    PacketSequenceViolation,
    ClientTimeViolation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SuspicionReport {
    pub player_id: PlayerId,
    pub sequence: SequenceNumber,
    pub kind: SuspicionKind,
    pub reason: String,
    pub observed_value: f32,
    pub expected_limit: f32,
    pub server_time_ms: Milliseconds,
}

impl SuspicionReport {
    pub fn new(
        player_id: PlayerId,
        sequence: SequenceNumber,
        kind: SuspicionKind,
        reason: impl Into<String>,
        observed_value: f32,
        expected_limit: f32,
        server_time_ms: Milliseconds,
    ) -> Self {
        Self {
            player_id,
            sequence,
            kind,
            reason: reason.into(),
            observed_value,
            expected_limit,
            server_time_ms,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum TelemetryEvent {
    ClientConnected {
        connection_id: ConnectionId,
        player_id: PlayerId,
        server_time_ms: Milliseconds,
    },
    ClientDisconnected {
        connection_id: ConnectionId,
        player_id: Option<PlayerId>,
        server_time_ms: Milliseconds,
    },
    CommandAccepted {
        command: InputCommand,
        server_time_ms: Milliseconds,
    },
    PlayerSnapshot(PlayerSnapshot),
    Suspicion(SuspicionReport),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_distance() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(3.0, 4.0);

        assert_eq!(a.distance(b), 5.0);
    }

    #[test]
    fn normalizes_non_zero_vector() {
        let vector = Vec2::new(10.0, 0.0).normalized();

        assert_eq!(vector, Vec2::new(1.0, 0.0));
    }

    #[test]
    fn keeps_zero_vector_zero_when_normalized() {
        assert_eq!(Vec2::ZERO.normalized(), Vec2::ZERO);
    }

    #[test]
    fn creates_suspicion_report() {
        let report = SuspicionReport::new(
            PlayerId(7),
            42,
            SuspicionKind::SpeedHack,
            "movement exceeded server budget",
            20.0,
            1.15,
            500,
        );

        assert_eq!(report.player_id, PlayerId(7));
        assert_eq!(report.sequence, 42);
        assert_eq!(report.kind, SuspicionKind::SpeedHack);
        assert_eq!(report.server_time_ms, 500);
    }

    #[test]
    fn creates_client_time_suspicion_report() {
        let report = SuspicionReport::new(
            PlayerId(4),
            3,
            SuspicionKind::ClientTimeViolation,
            "client timestamp jumped too far forward",
            5_000.0,
            250.0,
            750,
        );

        assert_eq!(report.player_id, PlayerId(4));
        assert_eq!(report.sequence, 3);
        assert_eq!(report.kind, SuspicionKind::ClientTimeViolation);
    }

    #[test]
    fn serializes_client_message_with_type_field() {
        let message = ClientMessage::Join {
            player_id: PlayerId(1),
        };

        let json = serde_json::to_string(&message).expect("message should serialize");

        assert!(json.contains("\"type\":\"Join\""));
        assert!(json.contains("\"player_id\":1"));
    }

    #[test]
    fn serializes_input_message_with_type_field() {
        let message = ClientMessage::Input(InputCommand {
            player_id: PlayerId(1),
            sequence: 10,
            client_time_ms: 1000,
            movement: Vec2::new(1.0, 0.0),
            fire: false,
            claimed_position: Some(Vec2::new(2.0, 0.0)),
        });

        let json = serde_json::to_string(&message).expect("message should serialize");

        assert!(json.contains("\"type\":\"Input\""));
        assert!(json.contains("\"sequence\":10"));
        assert!(json.contains("\"claimed_position\""));
    }

    #[test]
    fn deserializes_client_message_with_type_field() {
        let json = r#"{"type":"Join","data":{"player_id":1}}"#;

        let message: ClientMessage =
            serde_json::from_str(json).expect("message should deserialize");

        assert_eq!(
            message,
            ClientMessage::Join {
                player_id: PlayerId(1)
            }
        );
    }

    #[test]
    fn serializes_connection_telemetry_with_type_field() {
        let event = TelemetryEvent::ClientConnected {
            connection_id: 99,
            player_id: PlayerId(7),
            server_time_ms: 1234,
        };

        let json = serde_json::to_string(&event).expect("event should serialize");

        assert!(json.contains("\"type\":\"ClientConnected\""));
        assert!(json.contains("\"connection_id\":99"));
        assert!(json.contains("\"player_id\":7"));
    }

    #[test]
    fn serializes_command_accepted_with_server_time() {
        let event = TelemetryEvent::CommandAccepted {
            command: InputCommand {
                player_id: PlayerId(3),
                sequence: 8,
                client_time_ms: 700,
                movement: Vec2::new(1.0, 0.0),
                fire: false,
                claimed_position: Some(Vec2::new(4.0, 0.0)),
            },
            server_time_ms: 900,
        };

        let json = serde_json::to_string(&event).expect("event should serialize");

        assert!(json.contains("\"type\":\"CommandAccepted\""));
        assert!(json.contains("\"server_time_ms\":900"));
        assert!(json.contains("\"sequence\":8"));
    }
}
