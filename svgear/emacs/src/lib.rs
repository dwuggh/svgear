use emacs::{defun, Env, Result, UnibyteString, Value};
use raw_value::RawValue;
use std::{io::Write, sync::OnceLock};
use svgear::PaintType;

mod async_gear;
mod raw_value;
use async_gear::AsyncGear;

static GEAR: OnceLock<AsyncGear> = OnceLock::new();
emacs::plugin_is_GPL_compatible!();

#[defun]
fn render_math_to_png(
    callback: Value,
    content: String,
    inline: u8,
    width: Option<u32>,
    height: Option<u32>,
) -> Result<()> {
    let ty = if inline == 1 {
        PaintType::InlineTeX
    } else {
        PaintType::Equation
    };
    let gear = GEAR.get().unwrap();
    let mut raw = RawValue::from(callback);
    raw.make_global();
    gear.render_input(raw, ty, content, width, height)?;
    println!("calling render-math-to-png with callback: {:?}", callback);
    Ok(())
}

const SERVER: &[u8] = include_bytes!("../../../mathjax-svg-server/server");

#[emacs::module(name = "svgear-dyn", defun_prefix = "svgear")]
fn init(env: &Env) -> Result<Value<'_>> {
    let file_path = "/tmp/server";
    let mut f = std::fs::File::create(file_path)?;
    f.write_all(SERVER)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        f.set_permissions(std::fs::Permissions::from_mode(0o755))?;
    }
    let gear = AsyncGear::new(file_path.to_string())?;
    GEAR.get_or_init(move || gear);
    env.message("svgear loaded")
}

#[defun]
fn resolve(env: &Env) -> Result<()> {
    let gear = &GEAR.get().unwrap();
    let rt = &gear.runtime;
    rt.block_on(async move { gear.join_all(env).await })?;
    Ok(())
}

#[defun]
fn resolve_one(env: &Env) -> Result<()> {
    let gear = &GEAR.get().unwrap();
    let rt = &gear.runtime;
    rt.block_on(async move { gear.join_one(env).await })?;
    Ok(())
}

#[defun]
fn test1() -> Result<UnibyteString> {
    let png_raw = include_bytes!("../1.png");
    let mut png = Vec::new();
    png.extend_from_slice(png_raw);
    let png_str = UnibyteString::new(png);
    Ok(png_str)
}
