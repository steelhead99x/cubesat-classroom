use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use anyhow::{Context, Result, ensure};
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Request, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use moq_net::{Pattern, Patterns, Timestamp, origin, track};
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, watch};

use super::{Link, Sample, SensorInputs, connect, moq_net, start_relay, unix_ms};

const COMMANDS: &str = "classroom/commands";
const REPLIES: &str = "classroom/payload";
const TRACK: &str = "exchange.v1";
const FRAME_COUNT: usize = 8;
const FRAME_INTERVAL: Duration = Duration::from_millis(500);
const REQUEST_LIFETIME_MS: u64 = 5_000;
const DEADLINE: Duration = Duration::from_secs(9);

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Action {
    Sensors,
    Video,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Command {
    id: u64,
    issued_ms: u64,
    action: Action,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Reply {
    id: u64,
    sample: Option<Sample>,
    error: Option<String>,
}

#[derive(Clone, Serialize, Default)]
struct Display {
    sample: Option<Sample>,
    frame_id: u64,
    frame_count: usize,
    video_received: bool,
    loss_tried: bool,
    recovery_seen: bool,
    event: String,
    #[serde(skip)]
    jpeg: Bytes,
}

struct Station {
    commands: track::Producer,
    origin: origin::Consumer,
    contact: watch::Sender<bool>,
    sequence: Mutex<u64>,
    display: RwLock<Display>,
}

fn endpoint(relay: &super::LocalRelay) -> Link {
    Link {
        relay: relay.url.clone(),
        fingerprint: Some(relay.pin.clone()),
        broadcast: String::new(),
    }
}

fn bounded(value: &[u8], size: usize) -> Result<()> {
    ensure!(
        !value.is_empty() && value.len() <= size,
        "unexpected object size"
    );
    Ok(())
}

fn publish(
    origin: &origin::Producer,
    name: &str,
) -> Result<(moq_net::broadcast::Producer, track::Producer)> {
    let broadcast = origin.create_broadcast(name)?;
    let track = broadcast.create_track(TRACK, None)?;
    broadcast.announce(Default::default())?;
    Ok((broadcast, track))
}

async fn subscribe(origin: &origin::Consumer, name: &str) -> Result<track::Subscriber> {
    let mut announces = origin.announced();
    while let Some(update) = announces.next().await {
        if update.prefix.as_str() == name && update.kind.is_active() {
            let broadcast = origin.request_broadcast(name).await?;
            return Ok(broadcast.track(TRACK)?.subscribe(None).await?);
        }
    }
    anyhow::bail!("broadcast withdrawn")
}

async fn bridge(
    spacecraft: origin::Producer,
    ground: Link,
    mut contact: watch::Receiver<bool>,
) -> Result<()> {
    let replies = spacecraft.scope("", &Patterns::from(Pattern::subtree(REPLIES)?))?;
    let commands = spacecraft.scope("", &Patterns::from(Pattern::subtree(COMMANDS)?))?;
    let client = connect(&ground)?
        .with_publisher(replies.consume())
        .with_subscriber(commands);
    loop {
        contact.wait_for(|active| *active).await?;
        let session = client.connect(ground.relay.clone());
        tokio::select! {
            changed = async { contact.wait_for(|active| !*active).await.map(|_| ()) } => { changed?; }
            _ = session.closed() => tokio::time::sleep(Duration::from_millis(250)).await,
        }
    }
}

fn test_frame(frame: usize) -> Result<Vec<u8>> {
    let palette = [
        [225, 67, 70],
        [255, 196, 67],
        [60, 164, 121],
        [50, 113, 202],
        [48, 53, 84],
        [243, 232, 208],
    ];
    let grid = image::RgbImage::from_fn(320, 240, |x, y| {
        let index = (x / 54) as usize;
        let marker = (frame as u32 * 33 + 24, 140);
        let dx = x.abs_diff(marker.0);
        let dy = y.abs_diff(marker.1);
        let color = if dx.max(dy) <= 17 {
            [255, 255, 255]
        } else if dx.max(dy) <= 21 {
            [18, 28, 46]
        } else if y > 190 {
            palette[palette.len() - 1 - index]
        } else {
            palette[index]
        };
        image::Rgb(color)
    });
    let mut jpeg = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, 75).encode_image(&grid)?;
    Ok(jpeg)
}

async fn payload(
    relay: Link,
    inputs: Option<SensorInputs>,
    mut contact: watch::Receiver<bool>,
) -> Result<()> {
    let origin = moq_tokio::origin::spawn();
    let (_broadcast, replies) = publish(&origin, REPLIES)?;
    let client = connect(&relay)?
        .with_publisher(&origin)
        .with_subscriber(origin.clone());
    let _session = client.connect(relay.relay);
    let consumer = origin.consume();
    let mut recent = VecDeque::with_capacity(16);
    let mut readings = 0_u64;
    loop {
        contact.wait_for(|active| *active).await?;
        let run = async {
            let mut track = subscribe(&consumer, COMMANDS).await?;
            while let Some(mut group) = track.recv_group().await? {
                let Some(frame) = group.read_frame().await? else {
                    continue;
                };
                if bounded(&frame.payload, 512).is_err() {
                    continue;
                }
                let Ok(command) = serde_json::from_slice::<Command>(&frame.payload) else {
                    continue;
                };
                let now = unix_ms()?;
                if !now
                    .checked_sub(command.issued_ms)
                    .is_some_and(|age| age <= REQUEST_LIFETIME_MS)
                    || recent.contains(&command.id)
                {
                    continue;
                }
                if recent.len() == 16 {
                    recent.pop_front();
                }
                recent.push_back(command.id);
                let mut reply = Reply {
                    id: command.id,
                    sample: None,
                    error: None,
                };
                if command.action == Action::Sensors {
                    readings += 1;
                    let reading = match &inputs {
                        Some(inputs) => inputs.sample(readings),
                        None => Sample::synthetic(readings),
                    };
                    match reading {
                        Ok(sample) => reply.sample = Some(sample),
                        Err(error) => {
                            eprintln!("Sensor reading failed: {error:#}");
                            reply.error = Some("A sensor input is unavailable.".into());
                        }
                    }
                }
                let mut response = replies.append_group()?;
                response.write_frame(Timestamp::now(), serde_json::to_vec(&reply)?)?;
                if command.action == Action::Video {
                    for frame in 0..FRAME_COUNT {
                        if frame > 0 {
                            tokio::time::sleep(FRAME_INTERVAL).await;
                        }
                        response.write_frame(Timestamp::now(), test_frame(frame)?)?;
                    }
                }
                response.finish()?;
            }
            Ok::<(), anyhow::Error>(())
        };
        tokio::select! {
            result = run => {
                if let Err(error) = result {
                    eprintln!("Payload stopped answering requests, restarting: {error:#}");
                }
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
            changed = async { contact.wait_for(|active| !*active).await.map(|_| ()) } => { changed?; }
        }
    }
}

impl Station {
    async fn exchange(&self, command: Command) -> Result<()> {
        let mut replies = subscribe(&self.origin, REPLIES).await?;
        let mut group = self.commands.append_group()?;
        group.write_frame(Timestamp::now(), serde_json::to_vec(&command)?)?;
        group.finish()?;

        while let Some(mut group) = replies.recv_group().await? {
            let Some(first) = group.read_frame().await? else {
                continue;
            };
            bounded(&first.payload, 4096)?;
            let reply: Reply = serde_json::from_slice(&first.payload)?;
            if reply.id != command.id {
                continue;
            }
            if let Some(error) = reply.error {
                anyhow::bail!("{error}");
            }
            match command.action {
                Action::Sensors => {
                    let sample = reply.sample.context("missing sensor readings")?;
                    sample.age_ms(unix_ms()?)?;
                    self.display.write().expect("display lock").sample = Some(sample);
                }
                Action::Video => {
                    for index in 1..=FRAME_COUNT {
                        let frame = group.read_frame().await?.context("video ended early")?;
                        bounded(&frame.payload, 64 * 1024)?;
                        ensure!(frame.payload.starts_with(&[0xff, 0xd8]), "invalid JPEG");
                        let mut display = self.display.write().expect("display lock");
                        display.jpeg = frame.payload;
                        display.frame_id += 1;
                        display.frame_count = index;
                    }
                    ensure!(group.read_frame().await?.is_none(), "too many video frames");
                    self.display.write().expect("display lock").video_received = true;
                }
            }
            return Ok(());
        }
        anyhow::bail!("payload went offline")
    }
}

async fn action(State(station): State<Arc<Station>>, Json(action): Json<Action>) -> Response {
    let Ok(mut sequence) = station.sequence.try_lock() else {
        return (StatusCode::CONFLICT, "One request is already running.").into_response();
    };
    if !*station.contact.borrow() {
        return (
            StatusCode::CONFLICT,
            "Restore contact before sending a request.",
        )
            .into_response();
    }
    *sequence += 1;
    let Ok(issued_ms) = unix_ms() else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "System clock unavailable.",
        )
            .into_response();
    };
    {
        let mut display = station.display.write().expect("display lock");
        display.event = match action {
            Action::Sensors => "Waiting for a new sensor reading.",
            Action::Video => {
                display.frame_count = 0;
                display.jpeg = Bytes::new();
                "Waiting for the test camera."
            }
        }
        .into();
    }
    let result = tokio::time::timeout(
        DEADLINE,
        station.exchange(Command {
            id: *sequence,
            issued_ms,
            action,
        }),
    )
    .await;
    let mut display = station.display.write().expect("display lock");
    match result {
        Ok(Ok(())) => {
            display.recovery_seen |= display.loss_tried;
            display.event = match action {
                Action::Sensors => "Received a fresh sensor reading.",
                Action::Video => "Received all eight frames.",
            }
            .into();
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(Err(error)) => {
            display.event = error.to_string();
            (StatusCode::BAD_GATEWAY, display.event.clone()).into_response()
        }
        Err(_) => {
            display.event = "No complete reply. Check contact and try again.".into();
            (StatusCode::GATEWAY_TIMEOUT, display.event.clone()).into_response()
        }
    }
}

async fn contact(State(station): State<Arc<Station>>, Json(online): Json<bool>) -> StatusCode {
    station.contact.send_replace(online);
    let mut display = station.display.write().expect("display lock");
    display.loss_tried |= !online;
    display.event = if online {
        "Contact restored. Try a new request."
    } else {
        "Contact paused. The last readings are historical."
    }
    .into();
    StatusCode::NO_CONTENT
}

async fn status(State(station): State<Arc<Station>>) -> impl IntoResponse {
    let display = station.display.read().expect("display lock").clone();
    let online = *station.contact.borrow();
    let fresh = online
        && display
            .sample
            .as_ref()
            .is_some_and(|sample| unix_ms().and_then(|now| sample.age_ms(now)).is_ok());
    Json(serde_json::json!({"online": online, "fresh": fresh, "display": display}))
}

async fn jpeg(State(station): State<Arc<Station>>) -> Response {
    let jpeg = station.display.read().expect("display lock").jpeg.clone();
    if jpeg.is_empty() {
        return StatusCode::NOT_FOUND.into_response();
    }
    (
        [
            (header::CONTENT_TYPE, "image/jpeg"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        jpeg,
    )
        .into_response()
}

/// Accept only the loopback host names this server is reachable at. Rejecting other
/// Host values blocks DNS-rebinding; checking Origin on POST blocks cross-site requests.
fn allowed_host(port: u16, host: &str) -> bool {
    host == format!("127.0.0.1:{port}") || host == format!("localhost:{port}")
}

async fn local_only(State(port): State<u16>, request: Request, next: Next) -> Response {
    let heads: &HeaderMap = request.headers();
    let host = heads.get(header::HOST).and_then(|v| v.to_str().ok());
    let Some(host) = host.filter(|host| allowed_host(port, host)) else {
        return (
            StatusCode::FORBIDDEN,
            format!("Open the station at http://127.0.0.1:{port}"),
        )
            .into_response();
    };
    let expected_origin = format!("http://{host}");
    if request.method() == axum::http::Method::POST
        && heads.get(header::ORIGIN).and_then(|v| v.to_str().ok()) != Some(expected_origin.as_str())
    {
        return StatusCode::FORBIDDEN.into_response();
    }
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-store"),
    );
    response.headers_mut().insert(
        header::CONTENT_SECURITY_POLICY,
        header::HeaderValue::from_static(
            "default-src 'self'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'",
        ),
    );
    response
}

async fn report(name: &str, task: impl Future<Output = Result<()>>) {
    if let Err(error) = task.await {
        eprintln!("{name} stopped: {error:#}. Restart the station.");
    }
}

pub(crate) async fn run(port: u16, sensor_dir: Option<PathBuf>) -> Result<()> {
    let ground = start_relay(0).await?;
    let spacecraft = start_relay(0).await?;
    let ground_link = endpoint(&ground);
    let payload_link = endpoint(&spacecraft);
    let (control, channel) = watch::channel(true);
    // Report a task that stops, so a failure shows its cause instead of only timeouts.
    let _bridge = tokio::spawn(report(
        "Radio link simulation",
        bridge(
            spacecraft.origin.clone(),
            endpoint(&ground),
            channel.clone(),
        ),
    ));
    let _payload = tokio::spawn(report(
        "Payload simulation",
        payload(
            payload_link,
            sensor_dir.map(|dir| SensorInputs {
                temperature_mc: dir.join("temperature_mc"),
                voltage_mv: dir.join("voltage_mv"),
                light_lux: dir.join("light_lux"),
            }),
            channel,
        ),
    ));

    let origin = moq_tokio::origin::spawn();
    let (_broadcast, commands) = publish(&origin, COMMANDS)?;
    let ground_client = connect(&ground_link)?
        .with_publisher(&origin)
        .with_subscriber(origin.clone());
    let _session = ground_client.connect(ground_link.relay);
    let state = Arc::new(Station {
        commands,
        origin: origin.consume(),
        contact: control,
        sequence: Mutex::new(unix_ms()?),
        display: RwLock::new(Display {
            event: "Ready for your first request.".into(),
            ..Display::default()
        }),
    });

    let listener =
        tokio::net::TcpListener::bind(std::net::SocketAddr::from(([127, 0, 0, 1], port))).await?;
    let local = listener.local_addr()?;
    let addr = local.to_string();
    let app = Router::new()
        .route(
            "/",
            get(|| async { Html(include_str!("../web/index.html")) }),
        )
        .route(
            "/style.css",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
                    include_str!("../web/style.css"),
                )
            }),
        )
        .route(
            "/station.js",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
                    include_str!("../web/station.js"),
                )
            }),
        )
        .route("/api/state", get(status))
        .route("/api/frame.jpg", get(jpeg))
        .route("/api/request", post(action))
        .route("/api/contact", post(contact))
        .layer(DefaultBodyLimit::max(1024))
        .layer(middleware::from_fn_with_state(local.port(), local_only))
        .with_state(state);
    println!("Classroom ground station: http://{addr}");
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_loopback_host_names() {
        assert!(allowed_host(8080, "127.0.0.1:8080"));
        assert!(allowed_host(8080, "localhost:8080"));
        assert!(!allowed_host(8080, "localhost:8081"));
        assert!(!allowed_host(8080, "evil.example:8080"));
        assert!(!allowed_host(8080, "127.0.0.1"));
    }

    #[test]
    fn test_frames_are_jpeg_and_bounded() {
        for frame in 0..FRAME_COUNT {
            let jpeg = test_frame(frame).unwrap();
            assert!(jpeg.starts_with(&[0xff, 0xd8]));
            assert!(bounded(&jpeg, 64 * 1024).is_ok());
        }
    }
}
