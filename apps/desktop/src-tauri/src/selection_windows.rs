use windows::Win32::{
    System::Com::{
        CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
        CoUninitialize,
    },
    UI::{
        Accessibility::{
            CUIAutomation, IUIAutomation, IUIAutomationTextPattern, UIA_TextPatternId,
        },
        WindowsAndMessaging::GetForegroundWindow,
    },
};
use windows::core::Interface;

use super::{MAX_TEXT, SelectionError};

pub struct Source {
    window: usize,
}

struct Apartment;
impl Drop for Apartment {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}

fn failure(error: windows::core::Error) -> SelectionError {
    SelectionError::new(
        "read_failed",
        format!("Windows selection read failed: {error}"),
    )
}

impl Source {
    fn check_focus(&self) -> Result<(), SelectionError> {
        if unsafe { GetForegroundWindow().0 as usize } == self.window {
            Ok(())
        } else {
            Err(SelectionError::new(
                "read_failed",
                "Source application focus changed during capture",
            ))
        }
    }

    pub fn current() -> Result<Self, SelectionError> {
        let window = unsafe { GetForegroundWindow().0 as usize };
        if window == 0 {
            return Err(SelectionError::new(
                "unsupported",
                "No foreground application",
            ));
        }
        Ok(Self { window })
    }

    pub fn read(self) -> Result<String, SelectionError> {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED)
                .ok()
                .map_err(failure)?;
            let _apartment = Apartment;
            let automation: IUIAutomation =
                CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER).map_err(failure)?;
            let element = automation.GetFocusedElement().map_err(failure)?;
            self.check_focus()?;
            if element.CurrentIsPassword().map_err(failure)?.as_bool() {
                return Err(SelectionError::new(
                    "unsupported",
                    "Password fields are not read",
                ));
            }
            let pattern: IUIAutomationTextPattern = element
                .GetCurrentPattern(UIA_TextPatternId)
                .and_then(|value| value.cast())
                .map_err(|error| {
                    SelectionError::new(
                        "unsupported",
                        format!("Source does not expose text selection: {error}"),
                    )
                })?;
            let ranges = pattern.GetSelection().map_err(failure)?;
            let mut text = String::new();
            for index in 0..ranges.Length().map_err(failure)? {
                if index > 0 {
                    text.push('\n');
                }
                let remaining = MAX_TEXT.saturating_sub(text.chars().count());
                if remaining == 0 {
                    break;
                }
                let range = ranges.GetElement(index).map_err(failure)?;
                let part = range.GetText((remaining * 2 + 1) as i32).map_err(failure)?;
                text.extend(part.to_string().chars().take(remaining));
                if text.chars().count() == MAX_TEXT {
                    break;
                }
            }
            self.check_focus()?;
            Ok(text)
        }
    }
}
