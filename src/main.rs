use anyhow::Result;
use image::codecs::gif::GifDecoder;
use image::{AnimationDecoder, ImageDecoder};
use pixel_loop::canvas::{Canvas, InMemoryCanvas, RenderableCanvas};
use pixel_loop::color::Color;
use pixel_loop::input::{KeyboardKey, KeyboardState};
use pixel_loop::{canvas::CrosstermCanvas, input::CrosstermInputState};
use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, Instant};

struct AnimationFrame {
    canvas: InMemoryCanvas,
    delay: Duration,
}

fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        panic!(
            "Not enough arguments, usage: {} <animated.gif>",
            std::env::current_exe()?.to_str().unwrap()
        );
    }

    let pathname = args[1].clone();
    eprintln!("Showing {}", pathname);

    let reader = BufReader::new(File::open(pathname)?);
    let decoder = GifDecoder::new(reader)?;

    let (width, height) = decoder.dimensions();
    let frames = decoder.into_frames();
    let mut animation_frames: Vec<AnimationFrame> = vec![];
    for frame in frames {
        let frame = frame.unwrap();
        let delay: Duration = frame.delay().into();
        let buffer = frame.buffer().as_raw();
        let mut canvas = InMemoryCanvas::new(width, height, &Color::from_rgba(0, 0, 0, 0));
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;
                canvas.set(
                    x,
                    y,
                    &Color::from_rgba(
                        buffer[idx + 0],
                        buffer[idx + 2],
                        buffer[idx + 1],
                        buffer[idx + 3],
                    ),
                );
            }
        }

        animation_frames.push(AnimationFrame { canvas, delay });
    }

    eprintln!(
        "Loaded gif with {} frames and dimensions {}x{}",
        animation_frames.len(),
        width,
        height
    );

    struct State {
        animation_frames: Vec<AnimationFrame>,
        current_frame: usize,
        last_frame_change: Instant,
    }

    let mut canvas = CrosstermCanvas::new(
        width.clamp(0, u16::MAX as u32) as u16,
        height.clamp(0, u16::MAX as u32) as u16,
    );
    canvas.set_refresh_limit(120);

    let state = State {
        animation_frames,
        current_frame: 0,
        last_frame_change: Instant::now(),
    };
    let input = CrosstermInputState::new();

    fn update(
        _env: &mut pixel_loop::EngineEnvironment,
        state: &mut State,
        input: &CrosstermInputState,
        _canvas: &mut CrosstermCanvas,
    ) -> Result<()> {
        if input.is_key_pressed(KeyboardKey::Q) {
            std::process::exit(0);
        }

        let current = &state.animation_frames[state.current_frame];
        if state.last_frame_change.elapsed() >= current.delay {
            state.current_frame += 1;
            if state.current_frame >= state.animation_frames.len() {
                state.current_frame = 0;
            }
            state.last_frame_change = Instant::now();
        }

        Ok(())
    }

    fn render(
        _env: &mut pixel_loop::EngineEnvironment,
        state: &mut State,
        _input: &CrosstermInputState,
        canvas: &mut CrosstermCanvas,
        _dt: Duration,
    ) -> Result<()> {
        canvas.blit(
            &state.animation_frames[state.current_frame].canvas,
            0,
            0,
            None,
        );
        canvas.render()?;
        Ok(())
    }

    pixel_loop::run(60, state, input, canvas, update, render)?;

    Ok(())
}
