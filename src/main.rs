#![feature(assert_matches)]

use std::{
    fs,
    path::{Path, PathBuf},
};
#[macro_use]
extern crate anyhow;

#[derive(Debug)]
struct Device {
    file: PathBuf,
    name: String,
    max: u32,
    current: u32,
}

impl std::fmt::Display for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:15} {}% ({}/{})", self.name, self.percent(), self.current, self.max)
    }
}

impl Device {
    pub fn percent(&self) -> f32 {
        self.current as f32 / self.max as f32 * 100.0
    }

    pub fn change(&mut self, percent: f32) {
        let counts = percent / 100.0 * self.max as f32;
        let counts = counts + self.current as f32;
        self.current = match counts {
            c if c < 0.0 => 0,
            c if c >= self.max as f32 => self.max,
            c => c as u32
        };
    }

    pub fn set(&mut self, percent: f32) {
        let percent = percent.min(100.0);
        let counts = percent / 100.0 * self.max as f32;
        self.current = match counts {
            c if c < 0.0 => 0,
            c if c >= self.max as f32 => self.max,
            c => c as u32
        };
    }

    pub fn apply(&mut self, change: Change) {
        match change {
            Change::Absolute(p) => self.set(p),
            Change::Relative(p) => self.change(p),
        }
    }

    pub fn write(&self) -> anyhow::Result<()> {
        Ok(fs::write(self.file.join("brightness"), &format!("{}", self.current))?)
    }
}

fn read_device(path: impl AsRef<Path>) -> anyhow::Result<Device> {
    let path = path.as_ref();
    let name = path.file_name().ok_or(anyhow!("no device name"))?;
    let name = name.to_string_lossy().to_string();

    let max = fs::read_to_string(path.join("max_brightness"))?;
    let current = fs::read_to_string(path.join("brightness"))?;

    let max: u32 = max.trim().parse()?;
    let current: u32 = current.trim().parse()?;

    Ok(Device {
        file: path.to_path_buf(),
        name,
        max,
        current,
    })
}

fn get_devices(path: impl AsRef<Path>) -> anyhow::Result<Vec<Device>> {
    let path: &Path = path.as_ref();
    let path = path.to_string_lossy();

    glob::glob(&format!("{}/*", path))?
        .filter_map(Result::ok)
        .filter(|path| fs::metadata(&path).map(|m| m.is_dir()).unwrap_or(false))
        .map(read_device)
        .collect()
}

#[derive(Debug)]
pub enum Change {
    Absolute(f32),
    Relative(f32),
}

fn parse_change(ch: &str) -> anyhow::Result<Change> {
    match ch {
        ch if ch.starts_with('+') => Ok(Change::Relative(ch[1..].parse()?)),
        ch if ch.starts_with('-') => Ok(Change::Relative(-ch[1..].parse()?)),
        ch => Ok(Change::Absolute(ch.parse()?)),
    }
}

fn usage() {
    println!("Change backlight brightness");
    println!();
    println!("brack [device] [change]");
    println!();
    println!("device (optional) the device to change backlight on (see /sys/class/backlight)");
    println!("change (optional) absolute value in percent or change in percent prefixed with +/-.");
    println!();
    println!("no change will display current value.");
    println!("no device will try to automatically determine device.");
    println!();
    println!("Examples:");
    println!();
    println!("brack +10 # increase brightness with 10%");
    println!("brack -10 # decrease brightness with 10%");
    println!("brack 50 # set brightness to 50%");
    println!("brack intel_backlight +10 # increase brightness with 10% on the intel_backlight device");
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    {
        if args.iter().any(|s| *s == "-h" || *s == "--help") {
            usage();
            return Ok(());
        }
    }

    let (device, change) = if args.len() == 1 {
        (None, None)
    } else if args.len() == 2 {
        let arg = args[1].clone();
        if arg.starts_with('+') || arg.starts_with('-') || arg.parse::<f32>().is_ok() {
            (None, Some(arg))
        } else {
            (Some(arg), None)
        }
    } else if args.len() == 3 {
        (Some(args[1].clone()), Some(args[2].clone()))
    } else {
        usage();
        return Ok(());
    };

    let mut devices = get_devices("/sys/class/backlight")?;

    match (device, change) {
        (None, None) => {
            // List devices
            for dev in devices {
                println!("{}", dev);
            }
        }
        (Some(dev), None) => match devices.iter().find(|d| d.name == dev) {
            Some(dev) => println!("{}", dev),
            _ => (),
        },
        (Some(dev), Some(change)) => match devices.iter_mut().find(|d| d.name == dev) {
            Some(dev) => {
                let change = parse_change(&change)?;
                dev.apply(change);
                dev.write()?;
                println!("{}", dev)
            }
            _ => (),
        },
        (None, Some(change)) => match devices.iter_mut().find(|d| d.name == "intel_backlight" || d.name == "radeon_backlight" ) {
            Some(dev) => {
                let change = parse_change(&change)?;
                dev.apply(change);
                dev.write()?;
                println!("{}", dev)
            }
            _ => (),
        },
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_intel_backlight() {
        let _dev = read_device("tests/backlight/intel_backlight").unwrap();
    }

    #[test]
    fn dev_percent() {
        let dev = read_device("tests/backlight/intel_backlight").unwrap();
        println!("{}", dev.percent());
        assert!(dev.percent() == 27.95389);
    }

    #[test]
    fn increase_percent() {
        let mut dev = read_device("tests/backlight/intel_backlight").unwrap();
        assert!(dev.percent() == 27.95389);
        dev.change(10.0);
        println!("{}", dev.percent());
        assert!(dev.percent() == 37.896255);
    }

    #[test]
    fn decrease_percent() {
        let mut dev = read_device("tests/backlight/intel_backlight").unwrap();
        assert!(dev.percent() == 27.95389);
        dev.change(-10.0);
        println!("{}", dev.percent());
        assert!(dev.percent() == 17.939482);
    }

    #[test]
    fn read_all_devices() {
        let devs = get_devices("tests/backlight").unwrap();
        assert!(devs.len() == 1);
    }

    #[test]
    fn inc_change() {
        let c = parse_change("+10").unwrap();
        assert_matches!(c, Change::Relative(v) if v == 10.0);

        let c = parse_change("-10").unwrap();
        assert_matches!(c, Change::Relative(v) if v == -10.0);
    }
}
