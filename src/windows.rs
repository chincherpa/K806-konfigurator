//! Zeigergeschwindigkeit und Scrollzeilen werden – wie im Original – nicht in
//! der Maus, sondern systemweit in Windows gesetzt.

use anyhow::{bail, Result};

use crate::konfig::WindowsEinstellungen;

#[cfg(windows)]
pub fn anwenden(w: &WindowsEinstellungen) -> Result<()> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SystemParametersInfoW, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE, SPI_SETMOUSESPEED, SPI_SETWHEELSCROLLLINES,
    };
    let flags = SPIF_UPDATEINIFILE | SPIF_SENDCHANGE;
    if let Some(v) = w.zeigergeschwindigkeit {
        if !(1..=20).contains(&v) {
            bail!("windows.zeigergeschwindigkeit muss 1–20 sein");
        }
        // SAFETY: SPI_SETMOUSESPEED erwartet den Wert direkt im Zeigerargument.
        if unsafe { SystemParametersInfoW(SPI_SETMOUSESPEED, 0, v as usize as *mut _, flags) } == 0 {
            bail!("Zeigergeschwindigkeit konnte nicht gesetzt werden");
        }
        println!("Windows-Zeigergeschwindigkeit = {v}");
    }
    if let Some(z) = w.scrollzeilen {
        // SAFETY: SPI_SETWHEELSCROLLLINES liest nur uiParam.
        if unsafe { SystemParametersInfoW(SPI_SETWHEELSCROLLLINES, z, std::ptr::null_mut(), flags) } == 0 {
            bail!("Scrollzeilen konnten nicht gesetzt werden");
        }
        println!("Windows-Scrollzeilen = {z}");
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn anwenden(w: &WindowsEinstellungen) -> Result<()> {
    if w.zeigergeschwindigkeit.is_some() || w.scrollzeilen.is_some() {
        eprintln!("Hinweis: [windows]-Einstellungen werden nur unter Windows angewendet.");
    }
    Ok(())
}
