use accessibility_sys::*;
use core_foundation::{
    base::{CFType, TCFType},
    boolean::CFBoolean,
    dictionary::CFDictionary,
    string::CFString,
};

use super::{MAX_TEXT, SelectionError};

pub struct Source {
    process: i32,
}

fn check(error: AXError) -> Result<(), SelectionError> {
    if error == kAXErrorSuccess {
        return Ok(());
    }
    let code = match error {
        kAXErrorAPIDisabled => "permission",
        kAXErrorNoValue => "no_selection",
        kAXErrorAttributeUnsupported | kAXErrorNotImplemented => "unsupported",
        _ => "read_failed",
    };
    Err(SelectionError::new(
        code,
        format!(
            "Accessibility read failed: {} ({error})",
            error_string(error)
        ),
    ))
}

fn attribute(element: AXUIElementRef, name: &str) -> Result<CFType, SelectionError> {
    let key = CFString::new(name);
    let mut value = std::ptr::null();
    unsafe {
        check(AXUIElementCopyAttributeValue(
            element,
            key.as_concrete_TypeRef(),
            &mut value,
        ))?;
        if value.is_null() {
            return Err(SelectionError::new("no_selection", ""));
        }
        Ok(CFType::wrap_under_create_rule(value))
    }
}

fn element(value: &CFType) -> Result<AXUIElementRef, SelectionError> {
    if value.type_of() != unsafe { AXUIElementGetTypeID() } {
        return Err(SelectionError::new(
            "read_failed",
            "Accessibility returned an invalid element",
        ));
    }
    Ok(value.as_CFTypeRef() as AXUIElementRef)
}

fn focused_process() -> Result<i32, SelectionError> {
    unsafe {
        let system = AXUIElementCreateSystemWide();
        let _owner = CFType::wrap_under_create_rule(system.cast());
        check(AXUIElementSetMessagingTimeout(system, 1.0))?;
        let app = attribute(system, kAXFocusedApplicationAttribute)?;
        let mut process = 0;
        check(AXUIElementGetPid(element(&app)?, &mut process))?;
        Ok(process)
    }
}

impl Source {
    pub fn current() -> Result<Self, SelectionError> {
        unsafe {
            let prompt = CFType::wrap_under_get_rule(kAXTrustedCheckOptionPrompt.cast());
            let options =
                CFDictionary::from_CFType_pairs(&[(prompt, CFBoolean::true_value().as_CFType())]);
            if !AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef()) {
                return Err(SelectionError::new("permission", ""));
            }
            Ok(Self {
                process: focused_process()?,
            })
        }
    }

    fn check_focus(&self) -> Result<(), SelectionError> {
        if focused_process()? == self.process {
            Ok(())
        } else {
            Err(SelectionError::new(
                "read_failed",
                "Source application changed focus",
            ))
        }
    }

    pub fn read(self) -> Result<String, SelectionError> {
        unsafe {
            if !AXIsProcessTrusted() {
                return Err(SelectionError::new("permission", ""));
            }
            self.check_focus()?;
            let app = AXUIElementCreateApplication(self.process);
            let _owner = CFType::wrap_under_create_rule(app.cast());
            check(AXUIElementSetMessagingTimeout(app, 1.0))?;
            let focus = attribute(app, kAXFocusedUIElementAttribute)?;
            let focused = element(&focus)?;
            match attribute(focused, kAXSubroleAttribute) {
                Ok(role)
                    if role
                        .downcast::<CFString>()
                        .is_some_and(|value| value.to_string() == "AXSecureTextField") =>
                {
                    return Err(SelectionError::new(
                        "unsupported",
                        "Password fields are not read",
                    ));
                }
                Err(error) if error.code != "unsupported" && error.code != "no_selection" => {
                    return Err(error);
                }
                _ => {}
            }
            let selected = attribute(focused, kAXSelectedTextAttribute)?;
            let text = selected.downcast::<CFString>().ok_or_else(|| {
                SelectionError::new(
                    "read_failed",
                    "Accessibility returned an invalid selected-text value",
                )
            })?;
            self.check_focus()?;
            Ok(text.to_string().chars().take(MAX_TEXT).collect())
        }
    }
}
