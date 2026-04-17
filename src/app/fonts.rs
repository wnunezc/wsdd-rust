/// Registers fonts embedded in the binary plus Windows fallbacks when present.
pub(super) fn setup_fonts(ctx: &egui::Context) {
    const JETBRAINS_MONO: &[u8] = include_bytes!("../../assets/JetBrainsMono-Regular.ttf");
    const NOTO_SYMBOLS: &[u8] = include_bytes!("../../assets/NotoSansSymbols2-Regular.ttf");

    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "jetbrains_mono".to_owned(),
        egui::FontData::from_static(JETBRAINS_MONO),
    );
    fonts.font_data.insert(
        "noto_symbols".to_owned(),
        egui::FontData::from_static(NOTO_SYMBOLS),
    );
    add_windows_font_if_exists(
        &mut fonts,
        "fira_code",
        r"C:\Windows\Fonts\FiraCode-Regular.ttf",
        0,
    );
    add_user_font_if_exists(
        &mut fonts,
        "fira_code_user",
        r"Microsoft\Windows\Fonts\FiraCode-Regular.ttf",
        0,
    );
    add_windows_font_if_exists(&mut fonts, "windows_cjk", r"C:\Windows\Fonts\msyh.ttc", 0);
    add_windows_font_if_exists(
        &mut fonts,
        "windows_indic",
        r"C:\Windows\Fonts\Nirmala.ttc",
        0,
    );
    add_windows_font_if_exists(&mut fonts, "windows_ui", r"C:\Windows\Fonts\segoeui.ttf", 0);

    insert_if_present(&mut fonts, egui::FontFamily::Monospace, "jetbrains_mono");
    insert_if_present(&mut fonts, egui::FontFamily::Monospace, "fira_code");
    insert_if_present(&mut fonts, egui::FontFamily::Monospace, "fira_code_user");
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("noto_symbols".to_owned());
    push_if_present(&mut fonts, egui::FontFamily::Monospace, "windows_cjk");
    push_if_present(&mut fonts, egui::FontFamily::Monospace, "windows_indic");

    if fonts.font_data.contains_key("windows_ui") {
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "windows_ui".to_owned());
    }
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .push("noto_symbols".to_owned());
    push_if_present(&mut fonts, egui::FontFamily::Proportional, "windows_cjk");
    push_if_present(&mut fonts, egui::FontFamily::Proportional, "windows_indic");

    ctx.set_fonts(fonts);
}

fn add_windows_font_if_exists(
    fonts: &mut egui::FontDefinitions,
    name: &str,
    path: &str,
    index: u32,
) {
    if let Ok(bytes) = std::fs::read(path) {
        fonts.font_data.insert(
            name.to_owned(),
            egui::FontData {
                font: bytes.into(),
                index,
                tweak: Default::default(),
            },
        );
    }
}

fn add_user_font_if_exists(
    fonts: &mut egui::FontDefinitions,
    name: &str,
    relative_path: &str,
    index: u32,
) {
    let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") else {
        return;
    };
    let path = std::path::PathBuf::from(local_app_data).join(relative_path);
    if let Ok(bytes) = std::fs::read(path) {
        fonts.font_data.insert(
            name.to_owned(),
            egui::FontData {
                font: bytes.into(),
                index,
                tweak: Default::default(),
            },
        );
    }
}

fn push_if_present(fonts: &mut egui::FontDefinitions, family: egui::FontFamily, name: &str) {
    if fonts.font_data.contains_key(name) {
        fonts
            .families
            .entry(family)
            .or_default()
            .push(name.to_owned());
    }
}

fn insert_if_present(fonts: &mut egui::FontDefinitions, family: egui::FontFamily, name: &str) {
    if fonts.font_data.contains_key(name) {
        fonts
            .families
            .entry(family)
            .or_default()
            .insert(0, name.to_owned());
    }
}
