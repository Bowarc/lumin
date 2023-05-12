#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Monitor {
    pub name: String,
    pub position: (i32, i32),
    pub size: (i32, i32),
}

impl Monitor {
    pub fn from_info(name: String, info: winapi::um::winuser::MONITORINFOEXW) -> Self {
        Monitor {
            name: name.replace(['\\', '.'], ""), //.replace(['\\', '.'], "").replace("DISPLAY", "")
            position: (info.rcMonitor.left, info.rcMonitor.top),
            size: (
                info.rcMonitor.right - info.rcMonitor.left,
                info.rcMonitor.bottom - info.rcMonitor.top,
            ),
        }
    }
    // Tries to create a direction to represend the position of the screen
    pub fn direction(&self) -> String {
        const ERROR_MARGIN: i32 = 10;

        let mut output = String::new();

        if self.position.0.abs() < ERROR_MARGIN {
            // Center x
            output.push_str("Center")
        } else if self.position.0.is_positive() {
            // Right
            output.push_str("Right")
        } else if self.position.0.is_negative() {
            // Left
            output.push_str("Left")
        } else {
            // Wtf ?
            output.push_str("Unknown")
        }

        output.push('•');

        if self.position.1.abs() < ERROR_MARGIN {
            // Center y
            output.push_str("Center")
        } else if self.position.1.is_positive() {
            // Top
            output.push_str("Top")
        } else if self.position.1.is_negative() {
            // Bottom
            output.push_str("Bottom")
        } else {
            // Wtf ?
            output.push_str("Unknown")
        }

        output
    }
}
