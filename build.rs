#[cfg(not(target_os = "windows"))]
fn main() {}

#[cfg(target_os = "windows")]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon_with_id("graphics/sonde.ico", "100");
    res.compile().unwrap();
}
