use serde::de::DeserializeOwned;
use serde::Serialize;

fn get_clipboard_text() -> Result<String, String> {
    arboard::Clipboard::new()
        .and_then(|mut clipboard| clipboard.get_text())
        .map_err(|e| e.to_string())
}

fn set_clipboard_text(runes: &String) -> Option<()> {
    arboard::Clipboard::new()
        .and_then(|mut clipboard| clipboard.set_text(runes.clone()))
        .ok()
}

#[cfg(feature = "futhark")]
pub fn retrieve_from_runes<T: DeserializeOwned>() -> Result<T, String> {
    get_clipboard_text()
        .map(|text| {
            let futhark = crate::parse_runes(&text, crate::FUTHARK);
            if futhark.len() > 0 {
                futhark
            } else {
                crate::parse_runes(&text, crate::ALPHA_NUM)
            }
        })
        .and_then(|bytes| postcard::from_bytes(&bytes).map_err(|e| e.to_string()))
}

#[cfg(feature = "futhark")]
pub fn store_in_runes<T: Serialize>(t: &T) -> Option<()> {
    let runes = crate::create_runes(t, crate::FUTHARK);
    set_clipboard_text(&runes)
}

#[cfg(feature = "cursed")]
pub fn retrieve_cursed<T: DeserializeOwned>() -> Option<T> {
    get_clipboard_text()
        .ok()
        .and_then(|text| crate::read_from_curse(&text))
}

#[cfg(feature = "cursed")]
pub fn retrieve_cursed_bytes() -> Option<Vec<u8>> {
    get_clipboard_text()
        .ok()
        .map(|text| crate::bytes_from_curse(&text))
}

#[cfg(feature = "cursed")]
impl crate::CursedConfig {
    pub fn store_cursed<T: Serialize>(&self, t: &T, text: &str) -> Option<()> {
        let curse = crate::create_curse(t, &self, text);
        set_clipboard_text(&curse)
    }

    pub fn store_cursed_bytes(&self, bytes: &[u8], text: &str) -> Option<()> {
        let curse = self.generate_curse(text, bytes);
        set_clipboard_text(&curse)
    }
}
