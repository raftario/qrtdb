use chrono::{Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime};
use serde::Serialize;
use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    str::FromStr,
    thread,
};
use tokio::sync::broadcast;
use tracing::debug;

pub struct LogChannel(
    broadcast::Sender<LogMessage>,
    broadcast::Receiver<LogMessage>,
);

#[derive(Debug, Clone, Serialize)]
pub struct LogMessage {
    timestamp: NaiveDateTime,
    level: LogLevel,
    message: String,
}

impl LogChannel {
    pub fn new(tx: broadcast::Sender<LogMessage>) -> Self {
        let rx = tx.subscribe();
        Self(tx, rx)
    }

    pub fn rx(self) -> broadcast::Receiver<LogMessage> {
        self.1
    }
}

impl Clone for LogChannel {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.0.subscribe())
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
enum LogLevel {
    Verbose,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

#[tracing::instrument(level = "debug")]
pub fn init(logs: broadcast::Sender<LogMessage>) {
    debug!("starting log collector...");

    thread::Builder::new()
        .name(concat!(env!("CARGO_PKG_NAME"), "-log").to_string())
        .spawn(move || {
            let mut logcat = Command::new("logcat")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();

            let mut reader = BufReader::new(logcat.stdout.take().unwrap());
            let mut buf = String::new();

            loop {
                reader.read_line(&mut buf).unwrap();
                if let Some(message) = parse_line(&buf) {
                    logs.send(message).unwrap();
                }
                buf.clear();
            }
        })
        .unwrap();
}

fn parse_line(line: &str) -> Option<LogMessage> {
    let today = Local::today();

    let month = line[0..2].parse().ok()?;
    let day = line[3..5].parse().ok()?;
    let year = if today.month() >= month {
        today.year()
    } else {
        today.year() - 1
    };
    let date = NaiveDate::from_ymd(year, month, day);

    let hours = line[6..8].parse().ok()?;
    let minutes = line[9..11].parse().ok()?;
    let seconds = line[12..14].parse().ok()?;
    let milliseconds = line[15..18].parse().ok()?;
    let time = NaiveTime::from_hms_milli(hours, minutes, seconds, milliseconds);

    let timestamp = NaiveDateTime::new(date, time);
    let level = line[31..32].parse().ok()?;
    let message = line[33..].to_string();

    Some(LogMessage {
        timestamp,
        level,
        message,
    })
}

impl FromStr for LogLevel {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "V" => Ok(Self::Verbose),
            "D" => Ok(Self::Debug),
            "I" => Ok(Self::Info),
            "W" => Ok(Self::Warn),
            "E" => Ok(Self::Error),
            "F" => Ok(Self::Fatal),
            _ => Err(()),
        }
    }
}
