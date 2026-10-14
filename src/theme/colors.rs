//! trippy's colour names and theme items, mapped onto the window's slots.

use super::Slot;
use anyhow::anyhow;
use egui::Color32;

/// trippy's named colours: the 16 ANSI colours (with xterm's values, since a
/// window has no terminal palette) and the CSS colour names trippy accepts.
const NAMED: &[(&str, u32)] = &[
    ("black", 0x000000),
    ("red", 0xcd0000),
    ("green", 0x00cd00),
    ("yellow", 0xcdcd00),
    ("blue", 0x0000ee),
    ("magenta", 0xcd00cd),
    ("cyan", 0x00cdcd),
    ("gray", 0xe5e5e5),
    ("darkgray", 0x7f7f7f),
    ("lightred", 0xff0000),
    ("lightgreen", 0x00ff00),
    ("lightyellow", 0xffff00),
    ("lightblue", 0x5c5cff),
    ("lightmagenta", 0xff00ff),
    ("lightcyan", 0x00ffff),
    ("white", 0xffffff),
    ("aliceblue", 0xf0f8ff),
    ("antiquewhite", 0xfaebd7),
    ("aqua", 0x00ffff),
    ("aquamarine", 0x7fffd4),
    ("azure", 0xf0ffff),
    ("beige", 0xf5f5dc),
    ("bisque", 0xffe4c4),
    ("blanchedalmond", 0xffebcd),
    ("blueviolet", 0x8a2be2),
    ("brown", 0xa52a2a),
    ("burlywood", 0xdeb887),
    ("cadetblue", 0x5f9ea0),
    ("chartreuse", 0x7fff00),
    ("chocolate", 0xd2691e),
    ("coral", 0xff7f50),
    ("cornflowerblue", 0x6495ed),
    ("cornsilk", 0xfff8dc),
    ("crimson", 0xdc143c),
    ("darkblue", 0x00008b),
    ("darkcyan", 0x008b8b),
    ("darkgoldenrod", 0xb8860b),
    ("darkgreen", 0x006400),
    ("darkkhaki", 0xbdb76b),
    ("darkmagenta", 0x8b008b),
    ("darkolivegreen", 0x556b2f),
    ("darkorange", 0xff8c00),
    ("darkorchid", 0x9932cc),
    ("darkred", 0x8b0000),
    ("darksalmon", 0xe9967a),
    ("darkseagreen", 0x8fbc8f),
    ("darkslateblue", 0x483d8b),
    ("darkslategray", 0x2f4f4f),
    ("darkturquoise", 0x00ced1),
    ("darkviolet", 0x9400d3),
    ("deeppink", 0xff1493),
    ("deepskyblue", 0x00bfff),
    ("dimgray", 0x696969),
    ("dodgerblue", 0x1e90ff),
    ("firebrick", 0xb22222),
    ("floralwhite", 0xfffaf0),
    ("forestgreen", 0x228b22),
    ("fuchsia", 0xff00ff),
    ("gainsboro", 0xdcdcdc),
    ("ghostwhite", 0xf8f8ff),
    ("gold", 0xffd700),
    ("goldenrod", 0xdaa520),
    ("greenyellow", 0xadff2f),
    ("honeydew", 0xf0fff0),
    ("hotpink", 0xff69b4),
    ("indianred", 0xcd5c5c),
    ("indigo", 0x4b0082),
    ("ivory", 0xfffff0),
    ("khaki", 0xf0e68c),
    ("lavender", 0xe6e6fa),
    ("lavenderblush", 0xfff0f5),
    ("lawngreen", 0x7cfc00),
    ("lemonchiffon", 0xfffacd),
    ("lightcoral", 0xf08080),
    ("lightgoldenrodyellow", 0xfafad2),
    ("lightgray", 0xd3d3d3),
    ("lightpink", 0xffb6c1),
    ("lightsalmon", 0xffa07a),
    ("lightseagreen", 0x20b2aa),
    ("lightskyblue", 0x87cefa),
    ("lightslategray", 0x778899),
    ("lightsteelblue", 0xb0c4de),
    ("lime", 0x00ff00),
    ("limegreen", 0x32cd32),
    ("linen", 0xfaf0e6),
    ("maroon", 0x800000),
    ("mediumaquamarine", 0x66cdaa),
    ("mediumblue", 0x0000cd),
    ("mediumorchid", 0xba55d3),
    ("mediumpurple", 0x9370db),
    ("mediumseagreen", 0x3cb371),
    ("mediumslateblue", 0x7b68ee),
    ("mediumspringgreen", 0x00fa9a),
    ("mediumturquoise", 0x48d1cc),
    ("mediumvioletred", 0xc71585),
    ("midnightblue", 0x191970),
    ("mintcream", 0xf5fffa),
    ("mistyrose", 0xffe4e1),
    ("moccasin", 0xffe4b5),
    ("navajowhite", 0xffdead),
    ("navy", 0x000080),
    ("oldlace", 0xfdf5e6),
    ("olive", 0x808000),
    ("olivedrab", 0x6b8e23),
    ("orange", 0xffa500),
    ("orangered", 0xff4500),
    ("orchid", 0xda70d6),
    ("palegoldenrod", 0xeee8aa),
    ("palegreen", 0x98fb98),
    ("paleturquoise", 0xafeeee),
    ("palevioletred", 0xdb7093),
    ("papayawhip", 0xffefd5),
    ("peachpuff", 0xffdab9),
    ("peru", 0xcd853f),
    ("pink", 0xffc0cb),
    ("plum", 0xdda0dd),
    ("powderblue", 0xb0e0e6),
    ("purple", 0x800080),
    ("rebeccapurple", 0x663399),
    ("rosybrown", 0xbc8f8f),
    ("royalblue", 0x4169e1),
    ("saddlebrown", 0x8b4513),
    ("salmon", 0xfa8072),
    ("sandybrown", 0xf4a460),
    ("seagreen", 0x2e8b57),
    ("seashell", 0xfff5ee),
    ("sienna", 0xa0522d),
    ("silver", 0xc0c0c0),
    ("skyblue", 0x87ceeb),
    ("slateblue", 0x6a5acd),
    ("slategray", 0x708090),
    ("snow", 0xfffafa),
    ("springgreen", 0x00ff7f),
    ("steelblue", 0x4682b4),
    ("tan", 0xd2b48c),
    ("teal", 0x008080),
    ("thistle", 0xd8bfd8),
    ("tomato", 0xff6347),
    ("turquoise", 0x40e0d0),
    ("violet", 0xee82ee),
    ("wheat", 0xf5deb3),
    ("whitesmoke", 0xf5f5f5),
    ("yellowgreen", 0x9acd32),
];

/// Parses a colour as trippy does: a name (case and dashes ignored) or a
/// `#rrggbb` / `rrggbb` hex triplet.
pub fn parse(text: &str) -> anyhow::Result<Color32> {
    let normalized = text.to_ascii_lowercase().replace('-', "");
    if let Some((_, rgb)) = NAMED.iter().find(|(name, _)| *name == normalized) {
        return Ok(from_u32(*rgb));
    }
    let hex = normalized.strip_prefix('#').unwrap_or(&normalized);
    if hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Ok(from_u32(u32::from_str_radix(hex, 16)?));
    }
    Err(anyhow!("unknown color: {text}"))
}

const fn from_u32(rgb: u32) -> Color32 {
    Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

/// The slot a trippy theme item colours, or `None` for an item the window has
/// no counterpart for; an unknown item is an error.
pub fn slot_for_item(item: &str) -> anyhow::Result<Option<Slot>> {
    let slot = match item {
        "bg-color" => Some(Slot::bg),
        "border-color" => Some(Slot::border),
        "text-color" => Some(Slot::text),
        "hops-table-header-bg-color" => Some(Slot::header_bg),
        "hops-table-header-text-color" => Some(Slot::header_text),
        "hops-table-row-active-text-color" => Some(Slot::text),
        "hops-table-row-inactive-text-color" => Some(Slot::weak),
        "hops-table-row-active-selected-bg-color" => Some(Slot::accent),
        "hops-table-row-active-selected-text-color" => Some(Slot::on_accent),
        "hops-chart-selected-color" => Some(Slot::chart_selected),
        "hops-chart-unselected-color" => Some(Slot::chart_unselected),
        "hops-chart-axis-color" => Some(Slot::chart_axis),
        "frequency-chart-bar-color" => Some(Slot::freq_bar),
        "frequency-chart-text-color" => Some(Slot::freq_text),
        "flows-chart-bar-selected-color" => Some(Slot::flows_selected),
        "flows-chart-bar-unselected-color" => Some(Slot::flows_unselected),
        "flows-chart-text-current-color" => Some(Slot::flows_current),
        "flows-chart-text-non-current-color" => Some(Slot::flows_noncurrent),
        "samples-chart-color" => Some(Slot::samples),
        "samples-chart-lost-color" => Some(Slot::samples_lost),
        "help-dialog-bg-color" => Some(Slot::help_bg),
        "help-dialog-text-color" => Some(Slot::help_text),
        "settings-dialog-bg-color" => Some(Slot::settings_bg),
        "map-world-color" => Some(Slot::map_world),
        "map-radius-color" => Some(Slot::map_radius),
        "map-selected-color" => Some(Slot::map_selected),
        "info-bar-bg-color" => Some(Slot::info_bar_bg),
        "info-bar-text-color" => Some(Slot::info_bar_text),
        other if crate::config::file::THEME_ITEMS.contains(&other) => None,
        other => return Err(anyhow!("unknown theme item: {other}")),
    };
    Ok(slot)
}

/// Checks every `item=colour` pair without applying it.
pub fn validate(items: &[(String, String)]) -> anyhow::Result<()> {
    for (item, color) in items {
        slot_for_item(item)?;
        parse(color)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_names_and_hex() {
        assert_eq!(parse("red").unwrap(), Color32::from_rgb(0xcd, 0, 0));
        assert_eq!(parse("Light-Green").unwrap(), Color32::from_rgb(0, 0xff, 0));
        assert_eq!(
            parse("CornflowerBlue").unwrap(),
            Color32::from_rgb(0x64, 0x95, 0xed)
        );
        assert_eq!(
            parse("#102030").unwrap(),
            Color32::from_rgb(0x10, 0x20, 0x30)
        );
        assert_eq!(
            parse("102030").unwrap(),
            Color32::from_rgb(0x10, 0x20, 0x30)
        );
        assert!(parse("#12345").is_err());
        assert!(parse("octarine").is_err());
    }

    #[test]
    fn maps_items_onto_slots() {
        assert_eq!(
            slot_for_item("samples-chart-color").unwrap(),
            Some(Slot::samples)
        );
        assert_eq!(slot_for_item("settings-tab-text-color").unwrap(), None);
        assert!(slot_for_item("no-such-item").is_err());
        for item in crate::config::file::THEME_ITEMS {
            assert!(slot_for_item(item).is_ok(), "{item}");
        }
    }
}
