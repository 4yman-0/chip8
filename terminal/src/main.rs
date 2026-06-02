use chip8_terminal::{Chip8Result, Context, FONT_OCTO, OpFlag, Settings};
use crossterm::{cursor, event, execute, queue, style, terminal};
use std::io::{self, Write};
use std::time::Duration;

#[derive(clap::ValueEnum, Clone)]
enum SettingsPreset {
    Chip8,
    Schip,
    SchipModern,
    Xochip,
}

#[derive(clap::Parser)]
struct Cli {
    rom: std::path::PathBuf,

    #[arg(short, long)]
    settings: Option<SettingsPreset>,

    #[arg(long)]
    fps: Option<usize>,

    #[arg(long)]
    frame_ticks: Option<usize>,

    #[arg(long)]
    draw_wait: Option<bool>,
    /*#[arg(long)]
    shift_quirks: Option<bool>,

    #[arg(long)]
    vf_order_quirks: Option<bool>,

    #[arg(long)]
    logic_quirks: Option<bool>,*/
}

fn keycode_to_idx(code: event::KeyCode) -> Option<usize> {
    use event::KeyCode::Char;
    match code {
        Char('1') => Some(1),
        Char('2') => Some(2),
        Char('3') => Some(3),
        Char('4') => Some(0xc),
        Char('q') => Some(4),
        Char('w') => Some(5),
        Char('e') => Some(6),
        Char('r') => Some(0xd),
        Char('a') => Some(7),
        Char('s') => Some(8),
        Char('d') => Some(9),
        Char('f') => Some(0xe),
        Char('z') => Some(0xa),
        Char('x') => Some(0),
        Char('c') => Some(0xb),
        Char('v') => Some(0xf),
        _ => None,
    }
}

fn reset_stdout(stdout: &mut io::Stdout) {
    execute!(
        stdout,
        cursor::Show,
        terminal::EnableLineWrap,
        event::PopKeyboardEnhancementFlags,
        event::PopKeyboardEnhancementFlags,
    )
    .unwrap();
    terminal::disable_raw_mode().unwrap();
}

macro_rules! wprint {
	($stdout:expr, $fmt:literal$(, $arg:expr)*) => {
		write!($stdout, $fmt$(, $arg)*).unwrap();
	};
}

fn main() {
    app_main().unwrap_or_else(|err| {
        let mut stdout = io::stdout();
        reset_stdout(&mut stdout);
        eprintln!("Error: {err}");
    });
}

fn app_main() -> Chip8Result<()> {
    let cli = <Cli as clap::Parser>::parse();
    let rom = std::fs::read(cli.rom).expect("ROM error");
    let rng = rand::rng();
    let mut ctx = Context::new(&rom, rng);
    ctx.write_font(FONT_OCTO);

    if let Some(s) = cli.settings {
        ctx.settings = match s {
            SettingsPreset::Chip8 => Settings::chip8(),
            SettingsPreset::Schip => Settings::schip(),
            SettingsPreset::SchipModern => Settings::schip_modern(),
            SettingsPreset::Xochip => Settings::xochip(),
        };
    }

    if let Some(a) = cli.draw_wait {
        ctx.settings.draw_wait = a;
    }
    /*if let Some(a) = cli.shift_quirks {
        ctx.settings.shift_quirks = a;
    }
    if let Some(a) = cli.vf_order_quirks {
        ctx.settings.vf_order_quirks = a;
    }
    if let Some(a) = cli.logic_quirks {
        ctx.settings.logic_quirks = a;
    }*/

    let fps = cli.fps.unwrap_or(30);
    let frame_time = Duration::from_millis(1000 / fps as u64);
    let frame_ticks = cli.frame_ticks.unwrap_or(32);

    let mut paused = false;

    let mut stdout = io::stdout();
    terminal::enable_raw_mode().unwrap();
    execute!(
        stdout,
        cursor::Hide,
        terminal::DisableLineWrap,
        event::DisableFocusChange,
        event::DisableMouseCapture,
        event::PushKeyboardEnhancementFlags(event::KeyboardEnhancementFlags::REPORT_EVENT_TYPES),
    )
    .unwrap();

    'main_loop: loop {
        'input: while let Ok(true) = event::poll(Duration::ZERO)
            && let Ok(event) = event::read()
        {
            use event::{Event, KeyCode, KeyEventKind};
			//println!("evt: {event:?}");
            if let Event::Key(event) = event {
                if let KeyEventKind::Repeat = event.kind {
                    continue 'input;
                }
                if let KeyEventKind::Press = event.kind {
                    match event.code {
                        KeyCode::Esc => {
                            reset_stdout(&mut stdout);
                            std::process::exit(0);
                        }
                        KeyCode::Backspace => {
                            ctx.reset(&rom);
                        }
                        KeyCode::Char('p') => {
                            paused ^= true;
                        }
                        KeyCode::Char('o') if paused => {
                            ctx.emulate_cycle()?;
                            queue!(stdout, cursor::MoveTo(0, 16 + 4)).unwrap();
                            for (i, x) in ctx.gpr.iter().enumerate() {
                                wprint!(stdout, "v{i:X} = {x:X}; ");
                            }
                            queue!(stdout, cursor::MoveToNextLine(1)).unwrap();
                            for (i, x) in ctx.stack.iter().take(ctx.sp as usize).enumerate() {
                                wprint!(stdout, "s{i:X} = {x:X};");
                            }
                            queue!(stdout, cursor::MoveToNextLine(1)).unwrap();
                            wprint!(stdout, "pc = {:X}; ", ctx.pc);
                            wprint!(stdout, "idx = {:X}; ", ctx.idx);
                        }
                        _ => {},
                    }
                }
                if let Some(idx) = keycode_to_idx(event.code) {
                    ctx.keys[idx] = match event.kind {
                        KeyEventKind::Press => true,
                        KeyEventKind::Release => false,
                        _ => unreachable!(),
                    };
                    ctx.keys_modified[idx] = true;
                }
            }
        }

        if !paused {
            ctx.update_timers();

            'emulate: for _ in 0..frame_ticks {
                match ctx.emulate_cycle()? {
                    OpFlag::WaitVblank | OpFlag::WaitInput => break 'emulate,
                    OpFlag::Exit => break 'main_loop,
                    OpFlag::None => {}
                }
            }
        }

        ctx.update_input();

        queue!(
            stdout,
            terminal::BeginSynchronizedUpdate,
            cursor::MoveTo(0, 0),
            style::SetAttribute(style::Attribute::Bold),
        )
        .unwrap();

        {
            let mut lines = ctx.screen.as_slice().chunks_exact(ctx.screen.width());
            while let (Some(top_line), Some(bottom_line)) = (lines.next(), lines.next()) {
                for (top, bottom) in top_line.iter().zip(bottom_line) {
                    let bg = if *top { "\x1b[47m" } else { "\x1b[40m" };
                    let fg = if *bottom { "\x1b[37m" } else { "\x1b[30m" };

                    wprint!(stdout, "{}{}▄", fg, bg);
                }
                wprint!(stdout, "\x1b[0m");
                queue!(stdout, cursor::MoveToNextLine(1)).unwrap();
            }
        }

        queue!(stdout, style::SetAttribute(style::Attribute::Reset)).unwrap();

        if ctx.sound_timer > 1 {
            queue!(stdout, style::Print('🔔')).unwrap();
            let level = (ctx.sound_timer as u16 * 4) / 256;
            for _ in 0..level {
                queue!(stdout, style::Print('>')).unwrap();
            }
        } else {
            queue!(stdout, terminal::Clear(terminal::ClearType::CurrentLine)).unwrap();
        }

        /*println!(
            "op: 0x{:04X} => 0x{:02X}{:02X}",
            ctx.pc,
            ctx.memory[ctx.pc as usize + 1],
            ctx.memory[ctx.pc as usize],
        );*/

        queue!(stdout, terminal::EndSynchronizedUpdate).unwrap();

        stdout.flush().unwrap();

        std::thread::sleep(frame_time);
    }

    #[allow(unreachable_code)]
    reset_stdout(&mut stdout);

    Ok(())
}
