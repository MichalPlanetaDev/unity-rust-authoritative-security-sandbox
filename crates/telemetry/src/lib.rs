use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::Path,
};

use protocol::TelemetryEvent;

pub struct TelemetryWriter {
    writer: BufWriter<File>,
}

impl TelemetryWriter {
    pub fn create(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = File::create(path)?;

        Ok(Self {
            writer: BufWriter::new(file),
        })
    }

    pub fn append(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new().create(true).append(true).open(path)?;

        Ok(Self {
            writer: BufWriter::new(file),
        })
    }

    pub fn write_event(&mut self, event: &TelemetryEvent) -> io::Result<()> {
        serde_json::to_writer(&mut self.writer, event).map_err(to_invalid_data)?;
        self.writer.write_all(b"\n")?;
        self.writer.flush()
    }
}

pub fn read_events(path: impl AsRef<Path>) -> io::Result<Vec<TelemetryEvent>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut events = Vec::new();

    for (line_index, line) in reader.lines().enumerate() {
        let line = line?;

        if line.trim().is_empty() {
            continue;
        }

        let event = serde_json::from_str(&line).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "failed to parse telemetry line {}: {}",
                    line_index + 1,
                    error
                ),
            )
        })?;

        events.push(event);
    }

    Ok(events)
}

fn to_invalid_data(error: serde_json::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use protocol::{PlayerId, PlayerSnapshot, TelemetryEvent, Vec2};

    #[test]
    fn writes_and_reads_telemetry_jsonl() {
        let path = std::env::temp_dir().join(format!(
            "security-sandbox-telemetry-test-{}.jsonl",
            std::process::id()
        ));

        let events = vec![TelemetryEvent::PlayerSnapshot(PlayerSnapshot {
            player_id: PlayerId(1),
            position: Vec2::new(3.0, 4.0),
            health: 100,
            alive: true,
            server_time_ms: 250,
        })];

        {
            let mut writer = TelemetryWriter::create(&path).expect("failed to create writer");
            writer
                .write_event(&events[0])
                .expect("failed to write event");
        }

        let loaded = read_events(&path).expect("failed to read telemetry");

        assert_eq!(loaded, events);

        let _ = std::fs::remove_file(path);
    }
}
