use clap::Parser;
use iced::widget::canvas;
use iced::{
    mouse, Color, Element, Fill, Point, Rectangle, Renderer, Subscription, Theme, 
    time, Task
};
use iced_layershell::application;
use iced_layershell::reexport::{Anchor, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings};
use iced_layershell::to_layer_message;
use std::time::{Duration, Instant};
use rand::{thread_rng, Rng};
use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "GPU Accelerated Rain Wallpaper for Wayland")]
struct Config {
    #[arg(long, default_value = "basic")]
    mode: String,

    #[arg(long, default_value_t = false)]
    no_sound: bool,

    #[arg(long, default_value_t = 700)]
    rain_density: usize,

    #[arg(long, default_value_t = 1.0)]
    rain_speed: f32,

    #[arg(long, default_value_t = 0.3)]
    volume: f32,

    #[arg(long)]
    asset_path: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Mode { Basic, Thunderstorm }

pub fn main() -> Result<(), iced_layershell::Error> {
    let config = Config::parse();

    application(move || init(config.clone()), namespace, update, view)
        .style(style)
        .subscription(subscription)
        .settings(Settings {
            layer_settings: LayerShellSettings {
                layer: Layer::Bottom,
                anchor: Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right,
                size: None,
                exclusive_zone: -1,
                ..Default::default()
            },
            ..Default::default()
        })
        .run()
}

fn namespace() -> String { String::from("wrain") }

struct State {
    config: Config,
    mode: Mode,
    drops: Vec<RainDrop>,
    wind_force: f32,
    wind_target: f32,
    lightning: Option<LightningStrike>,

    asset_root: PathBuf,

    _audio_stream: Option<OutputStream>,
    stream_handle: Option<rodio::OutputStreamHandle>,
    rain_sink: Option<Sink>,
}

struct LightningStrike {
    path: Vec<Point>,
    opacity: f32,
    flash_intensity: f32,
    thunder_triggered: bool,
    thunder_delay_timer: f32,
}

struct RainDrop {
    x: f32, y: f32, vx: f32, vy: f32, mass: f32,
}

impl RainDrop {
    fn update(&mut self, width: f32, height: f32, wind: f32, mode: Mode) {
        let wind_mult = if mode == Mode::Thunderstorm { 0.12 } else { 0.03 };
        self.vx += (wind / self.mass) * wind_mult;
        self.vx *= 0.96;
        self.x += self.vx;
        self.y += self.vy;

        if self.x > width { self.x = 0.0; }
        if self.x < 0.0 { self.x = width; }
        if self.y > height {
            self.y = -20.0;
            self.x = thread_rng().gen_range(0.0..width);
            self.vx += thread_rng().gen_range(-1.0..1.0);
        }
    }
}

#[to_layer_message]
#[derive(Debug, Clone)]
enum Message {
    Tick(Instant),
}

fn init(config: Config) -> (State, Task<Message>) {
    let mode = if config.mode == "thunderstorm" { Mode::Thunderstorm } else { Mode::Basic };
    
    let asset_root = PathBuf::from(
        config.asset_path.clone()
            .or_else(|| std::env::var("WRAIN_ASSET_PATH").ok())
            .unwrap_or_else(|| "assets".to_string())
    );

    // Audio Setup
    let mut _audio_stream = None;
    let mut stream_handle = None;
    let mut rain_sink = None;

    if !config.no_sound {
        if let Ok((stream, handle)) = OutputStream::try_default() {
            let sink = Sink::try_new(&handle).unwrap();
            
            // Construct path dynamically
            let rain_file = asset_root.join("rain_loop.mp3");
            
            if let Ok(file) = File::open(rain_file) {
                let source = Decoder::new(BufReader::new(file)).unwrap().repeat_infinite();
                sink.append(source);
                sink.set_volume(if mode == Mode::Thunderstorm { config.volume * 1.5 } else { config.volume });
                sink.play();
            }
            _audio_stream = Some(stream);
            stream_handle = Some(handle);
            rain_sink = Some(sink);
        }
    }

    let drops = (0..config.rain_density).map(|_| RainDrop {
        x: thread_rng().gen_range(0.0..2000.0),
        y: thread_rng().gen_range(0.0..1100.0),
        vx: 0.0,
        vy: thread_rng().gen_range(12.0..25.0) * config.rain_speed,
        mass: thread_rng().gen_range(0.7..1.3),
    }).collect();

    (State { 
        config, mode, drops, wind_force: 0.0, wind_target: 0.0, lightning: None,
        asset_root,
        _audio_stream, stream_handle, rain_sink,
    }, Task::none())
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Tick(_) => {
            let mut rng = thread_rng();
            
            // 1. Organic Wind Logic
            let wind_limit = if state.mode == Mode::Thunderstorm { 6.0 } else { 1.5 };
            if rng.gen_bool(0.01) { 
                state.wind_target = rng.gen_range(-wind_limit..wind_limit); 
            }
            state.wind_force += (state.wind_target - state.wind_force) * 0.02;

            for drop in &mut state.drops {
                drop.update(2000.0, 1100.0, state.wind_force, state.mode); 
            }

            // 2. Thunderstorm logic
            if state.mode == Mode::Thunderstorm {
                if let Some(strike) = &mut state.lightning {
                    strike.opacity -= 0.04;
                    strike.flash_intensity *= 0.85;
                    strike.thunder_delay_timer -= 1.0;
                    
                    if !strike.thunder_triggered && strike.thunder_delay_timer <= 0.0 {
                        if let Some(handle) = &state.stream_handle {
                            play_thunder(handle, state.config.volume, &state.asset_root);
                        }
                        strike.thunder_triggered = true;
                    }

                    if strike.opacity <= 0.0 && strike.thunder_triggered { 
                        state.lightning = None; 
                    }
                } else if rng.gen_bool(0.004) {
                    state.lightning = Some(generate_lightning());
                }
            }
            Task::none()
        }
        _ => Task::none(), 
    }
}

fn play_thunder(handle: &rodio::OutputStreamHandle, base_vol: f32, asset_root: &std::path::Path) {
    let mut rng = thread_rng();
    let filenames = ["thunder1.mp3", "thunder2.mp3"];
    let chosen = filenames[rng.gen_range(0..filenames.len())];
    
    // Construct path dynamically
    let thunder_path = asset_root.join(chosen);

    if let Ok(file) = File::open(thunder_path) {
        if let Ok(source) = Decoder::new(BufReader::new(file)) {
            let sink = Sink::try_new(handle).unwrap();
            sink.set_speed(rng.gen_range(0.7..1.2));
            sink.set_volume(rng.gen_range(base_vol..base_vol * 2.5));
            sink.append(source);
            sink.detach();
        }
    }
}

fn generate_lightning() -> LightningStrike {
    let mut rng = thread_rng();
    let mut path = Vec::new();
    let mut curr = Point::new(rng.gen_range(200.0..1800.0), 0.0);
    path.push(curr);
    while curr.y < 1100.0 {
        curr = Point::new(curr.x + rng.gen_range(-100.0..100.0), curr.y + rng.gen_range(40.0..130.0));
        path.push(curr);
    }
    LightningStrike { 
        path, opacity: 1.0, flash_intensity: 0.3, 
        thunder_triggered: false, 
        thunder_delay_timer: rng.gen_range(10.0..50.0) 
    }
}

fn subscription(_state: &State) -> Subscription<Message> {
    time::every(Duration::from_millis(16)).map(Message::Tick)
}

fn view(state: &State) -> Element<'_, Message> {
    canvas(state).width(Fill).height(Fill).into()
}

impl<Message> canvas::Program<Message> for State {
    type State = ();
    fn draw(&self, _s: &(), renderer: &Renderer, _t: &Theme, bounds: Rectangle, _c: mouse::Cursor) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        if let Some(strike) = &self.lightning {
            let flash_rect = canvas::Path::rectangle(Point::ORIGIN, bounds.size());
            frame.fill(&flash_rect, Color::from_rgba(1.0, 1.0, 1.0, strike.flash_intensity));
            let mut builder = canvas::path::Builder::new();
            if let Some(first) = strike.path.first() {
                builder.move_to(scale_pt(*first, bounds));
                for pt in strike.path.iter().skip(1) { builder.line_to(scale_pt(*pt, bounds)); }
            }
            frame.stroke(&builder.build(), canvas::Stroke {
                style: canvas::Style::Solid(Color::from_rgba(0.9, 0.9, 1.0, strike.opacity)),
                width: 2.0, ..Default::default()
            });
        }
        let rain_alpha = if self.mode == Mode::Thunderstorm { 0.4 } else { 0.3 };
        for drop in &self.drops {
            let x = (drop.x / 2000.0) * bounds.width;
            let y = (drop.y / 1100.0) * bounds.height;
            let path = canvas::Path::line(Point::new(x, y), Point::new(x + drop.vx * 0.8, y + drop.vy * 0.5));
            frame.stroke(&path, canvas::Stroke {
                style: canvas::Style::Solid(Color::from_rgba(0.8, 0.9, 1.0, rain_alpha)),
                width: 0.8, ..Default::default()
            });
        }
        vec![frame.into_geometry()]
    }
}

fn scale_pt(p: Point, b: Rectangle) -> Point {
    Point::new((p.x / 2000.0) * b.width, (p.y / 1100.0) * b.height)
}

fn style(_s: &State, _t: &Theme) -> iced::theme::Style {
    iced::theme::Style { background_color: Color::TRANSPARENT, text_color: Color::WHITE }
}