use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use flow2_core::backlight::{self, Backlight, Mode};
use flow2_core::client::ViaClient;
use flow2_core::hid::{self, DeviceInfo, HidTransport};
use flow2_core::keycodes::{legend, Variant};
use flow2_core::layout::ModelDef;
use flow2_core::{battery, keymap, models, profile, via, Error, Result};

#[derive(Parser)]
#[command(
    name = "flow2ctl",
    about = "Configure Lofree Flow 2 keyboards over USB"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Clone, Copy, ValueEnum)]
enum LightArg {
    Off,
    On,
    Breathing,
}

#[derive(Subcommand)]
enum Cmd {
    /// Check the connection and print protocol version, layers and model.
    Probe,
    /// Print one layer as key legends.
    Keys {
        #[arg(long, default_value_t = 0)]
        layer: u8,
    },
    /// Set one key. `code` is a hex keycode such as 0x0029. The write is read back and verified.
    SetKey {
        layer: u8,
        row: u8,
        col: u8,
        #[arg(value_parser = parse_hex)]
        code: u16,
    },
    /// Backlight mode.
    Light { mode: LightArg },
    /// Backlight brightness, 0 to 100.
    Brightness {
        #[arg(value_parser = clap::value_parser!(u8).range(0..=100))]
        percent: u8,
    },
    /// Battery level (Bluetooth, macOS only for now).
    Battery,
    /// Save the keyboard's current state to a profile file.
    Backup { file: PathBuf },
    /// Write a profile file to the keyboard.
    Apply { file: PathBuf },
    /// Diagnostic: list VIA custom channels that answer with data.
    Scan,
}

fn parse_hex(s: &str) -> std::result::Result<u16, String> {
    u16::from_str_radix(s.trim_start_matches("0x"), 16).map_err(|e| e.to_string())
}

struct Session {
    client: ViaClient<HidTransport>,
    model_id: &'static str,
    def: ModelDef,
    info: DeviceInfo,
}

fn connect() -> Result<Session> {
    let (transport, info) = hid::find()?;
    let (model, def) = models::by_product_id(info.product_id).ok_or_else(|| {
        Error::BadReply(format!(
            "unsupported model, product id {:#06x} ({}). Please open an issue at github.com/kills1nt/openlofree",
            info.product_id, info.product
        ))
    })?;
    if !model.verified {
        eprintln!("note: {} is not verified on real hardware", model.label);
    }
    Ok(Session {
        client: ViaClient::new(transport),
        model_id: model.id,
        def,
        info,
    })
}

impl Session {
    /// Call before the first write of a run. Creates the factory backup once.
    fn guard_write(&mut self) -> Result<()> {
        let Some(path) = profile::factory_backup_path() else {
            return Ok(());
        };
        if profile::ensure_factory_backup(&mut self.client, self.model_id, &self.def, &path)? {
            println!("saved factory backup to {}", path.display());
        }
        Ok(())
    }
}

fn run(cli: Cli) -> Result<()> {
    if let Cmd::Battery = cli.cmd {
        match battery::read() {
            Some(pct) => println!("battery: {pct}%"),
            None => println!("battery: unknown (needs a Bluetooth connection, macOS only for now)"),
        }
        return Ok(());
    }
    let mut s = connect()?;
    match cli.cmd {
        Cmd::Battery => unreachable!(),
        Cmd::Probe => {
            println!(
                "device: {} ({:04x}:{:04x})",
                s.info.product, s.info.vendor_id, s.info.product_id
            );
            println!("model: {}", s.model_id);
            println!("via protocol: {}", s.client.protocol_version()?);
            println!("layers: {}", s.client.layer_count()?);
        }
        Cmd::Keys { layer } => {
            let km = keymap::read(&mut s.client, s.def.rows, s.def.cols)?;
            let keys = km
                .layers
                .get(layer as usize)
                .ok_or_else(|| Error::Profile(format!("no layer {layer}")))?;
            for k in &s.def.keys {
                let code = keys[k.row as usize * s.def.cols as usize + k.col as usize];
                println!(
                    "{},{}\t{:#06x}\t{}",
                    k.row,
                    k.col,
                    code,
                    legend(code, Variant::Mac)
                );
            }
        }
        Cmd::SetKey {
            layer,
            row,
            col,
            code,
        } => {
            let layers = s.client.layer_count()?;
            keymap::check_position(s.def.rows, s.def.cols, layers, layer, row, col)?;
            s.guard_write()?;
            keymap::set_key_verified(&mut s.client, layer, row, col, code)?;
            println!(
                "ok: layer {layer} key {row},{col} = {code:#06x} ({})",
                legend(code, Variant::Mac)
            );
        }
        Cmd::Light { mode } => {
            s.guard_write()?;
            let current = backlight::read(&mut s.client)?;
            let brightness = if current.brightness == 0 {
                255
            } else {
                current.brightness
            };
            let mode = match mode {
                LightArg::Off => Mode::Off,
                LightArg::On => Mode::Steady,
                LightArg::Breathing => Mode::Breathing,
            };
            backlight::apply(&mut s.client, Backlight { mode, brightness })?;
            println!("ok");
        }
        Cmd::Brightness { percent } => {
            s.guard_write()?;
            let current = backlight::read(&mut s.client)?;
            let mode = if percent == 0 {
                Mode::Off
            } else if current.mode == Mode::Off {
                Mode::Steady
            } else {
                current.mode
            };
            backlight::apply(
                &mut s.client,
                Backlight {
                    mode,
                    brightness: backlight::level_from_percent(percent),
                },
            )?;
            println!("ok");
        }
        Cmd::Backup { file } => {
            profile::capture(&mut s.client, s.model_id, &s.def, "Backup")?.save(&file)?;
            println!("saved {}", file.display());
        }
        Cmd::Apply { file } => {
            let p = profile::Profile::load(&file)?;
            if p.model != s.model_id {
                return Err(Error::Profile(format!(
                    "profile is for {}, keyboard is {}",
                    p.model, s.model_id
                )));
            }
            s.guard_write()?;
            println!(
                "ok: {} keys changed",
                profile::apply(&mut s.client, &s.def, &p)?
            );
        }
        Cmd::Scan => {
            for channel in 0u8..=5 {
                for value in 1u8..=15 {
                    let mut r = via::backlight_get(value);
                    r[1] = channel;
                    if let Ok(reply) = s.client.raw(&r) {
                        if reply[0] == via::CMD_CUSTOM_GET_VALUE
                            && reply[3..].iter().any(|&b| b != 0)
                        {
                            println!("channel {channel} value {value}: {:02X?}", &reply[..8]);
                        }
                    }
                }
            }
            println!("scan done (channels 0-5, values 1-15, only answers with data are listed)");
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            if matches!(e, Error::Timeout) {
                eprintln!(
                    "The keyboard was found but did not answer. Wake it with a key press first. Then check that its switch is in wired (USB) mode, \
                     close anything else using it (VIA in a browser, Lofree's configurator) and try another cable or port."
                );
            }
            ExitCode::FAILURE
        }
    }
}
