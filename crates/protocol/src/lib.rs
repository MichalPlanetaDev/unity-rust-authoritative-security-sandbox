use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 1;

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
    Join {
        player_id: PlayerId,
        #[serde(default)]
        protocol_version: Option<u32>,
    },
    Input(InputCommand),
    Ping {
        client_time_ms: Milliseconds,
    },
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
        protocol_version: u32,
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
    ProtocolViolation,
    RateLimitViolation,
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
        assert_eq!(Vec2::new(0.0, 0.0).distance(Vec2::new(3.0, 4.0)), 5.0);
    }

    #[test]
    fn normalizes_non_zero_vector() {
        assert_eq!(Vec2::new(10.0, 0.0).normalized(), Vec2::new(1.0, 0.0));
    }

    #[test]
    fn keeps_zero_vector_zero_when_normalized() {
        assert_eq!(Vec2::ZERO.normalized(), Vec2::ZERO);
    }

    #[test]
    fn serializes_join_with_protocol_version() {
        let message = ClientMessage::Join {
            player_id: PlayerId(1),
            protocol_version: Some(PROTOCOL_VERSION),
        };

        let json = serde_json::to_string(&message).expect("message should serialize");

        assert!(json.contains("\"type\":\"Join\""));
        assert!(json.contains("\"player_id\":1"));
        assert!(json.contains("\"protocol_version\":1"));
    }

    #[test]
    fn deserializes_legacy_join_without_protocol_version() {
        let json = r#"{"type":"Join","data":{"player_id":1}}"#;

        let message: ClientMessage =
            serde_json::from_str(json).expect("legacy join should deserialize");

        assert_eq!(
            message,
            ClientMessage::Join {
                player_id: PlayerId(1),
                protocol_version: None
            }
        );
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
    fn creates_protocol_suspicion_report() {
        let report = SuspicionReport::new(
            PlayerId(7),
            0,
            SuspicionKind::ProtocolViolation,
            "unsupported protocol version",
            999.0,
            PROTOCOL_VERSION as f32,
            500,
        );

        assert_eq!(report.kind, SuspicionKind::ProtocolViolation);
        assert_eq!(report.expected_limit, 1.0);
    }
}
