//! Generic analog input support.

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// The minimum value of an analog input.
pub const ANALOG_MIN: f32 = -1.0;
/// The maximum value of an analog input.
pub const ANALOG_MAX: f32 = 1.0;

/// Wrapper around `f32` for analog inputs.
#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub(crate) struct AnalogInputValue(f32);

impl From<i16> for AnalogInputValue {
    fn from(value: i16) -> Self {
        let analog_value = value as f32 / i16::MAX as f32;
        Self(analog_value.clamp(ANALOG_MIN, ANALOG_MAX))
    }
}

impl From<f32> for AnalogInputValue {
    fn from(value: f32) -> Self {
        if value.is_finite() {
            Self(value.clamp(ANALOG_MIN, ANALOG_MAX))
        } else {
            Self(0.0)
        }
    }
}

impl From<AnalogInputValue> for f32 {
    fn from(value: AnalogInputValue) -> Self {
        value.0
    }
}

/// Wrapper around `f32` for deadzones.
#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Deadzone(f32);

impl From<AnalogInputValue> for Deadzone {
    fn from(value: AnalogInputValue) -> Self {
        Self(value.0.abs())
    }
}

impl From<Deadzone> for f32 {
    fn from(value: Deadzone) -> Self {
        value.0
    }
}

/// Container for analog inputs.
#[derive(Debug)]
pub struct AnalogInput<T> {
    inputs: HashMap<T, AnalogInputValue>,

    just_activated: HashSet<T>,
    just_deactivated: HashSet<T>,
    deadzone: Deadzone,

    just_activated_digital: HashSet<T>,
    just_deactivated_digital: HashSet<T>,
    deadzone_digital: Deadzone,
}

impl<T> AnalogInput<T>
where
    T: Hash + Eq,
{
    /// Gets the value of an analog input.
    ///
    /// Returns `0.0` if the input is within the analog deadzone, or if it has not been read yet.
    pub fn value(&self, input: T) -> f32 {
        match self.inputs.get(&input) {
            Some(&value) if Deadzone::from(value) > self.deadzone => f32::from(value),
            _ => 0.0,
        }
    }

    /// Checks if an analog input just left the analog deadzone.
    pub fn just_activated(&self, input: T) -> Option<f32> {
        if self.just_activated.contains(&input) {
            Some(self.value(input))
        } else {
            None
        }
    }

    /// Checks if an analog input just entered the analog deadzone.
    pub fn just_deactivated(&self, input: T) -> bool {
        self.just_deactivated.contains(&input)
    }

    /// Converts an analog input to a digital value.
    ///
    /// Returns either `ANALOG_MIN` or `ANALOG_MAX` when the input is outside the digital deadzone,
    /// and `0.0` otherwise.
    pub fn value_digital(&self, input: T) -> f32 {
        match self.inputs.get(&input) {
            Some(&value) if Deadzone::from(value) > self.deadzone_digital => {
                if f32::from(value) < 0.0 {
                    ANALOG_MIN
                } else {
                    ANALOG_MAX
                }
            }
            _ => 0.0,
        }
    }

    /// Checks if an analog input just left the digital deadzone.
    pub fn just_activated_digital(&self, input: T) -> Option<f32> {
        if self.just_activated_digital.contains(&input) {
            Some(self.value_digital(input))
        } else {
            None
        }
    }

    /// Checks if an analog input just entered the digital deadzone.
    pub fn just_deactivated_digital(&self, input: T) -> bool {
        self.just_deactivated_digital.contains(&input)
    }
}

impl<T> AnalogInput<T>
where
    T: Hash + Copy + Eq,
{
    pub(crate) fn set(&mut self, input: T, value: AnalogInputValue) {
        let old_value = self.inputs.insert(input, value);
        let value = f32::from(value);
        let deadzone = f32::from(self.deadzone);
        let deadzone_digital = f32::from(self.deadzone_digital);

        if let Some(old_value) = old_value {
            let old_value = f32::from(old_value);

            if value.abs() < deadzone {
                self.just_activated.remove(&input);
                if old_value.abs() >= deadzone {
                    self.just_deactivated.insert(input);
                }
            } else {
                self.just_deactivated.remove(&input);
                // It is possible for an analog input to completely pass through the deadzone
                // between updates. In that case, both the old and new values would exceed the
                // deadzone, but they would have opposite signs.
                if old_value.abs() < deadzone || value.signum() != old_value.signum() {
                    self.just_activated.insert(input);
                }
            }

            if value.abs() < deadzone_digital {
                self.just_activated_digital.remove(&input);
                if old_value.abs() >= deadzone_digital {
                    self.just_deactivated_digital.insert(input);
                }
            } else {
                self.just_deactivated_digital.remove(&input);
                if old_value.abs() < deadzone_digital || value.signum() != old_value.signum() {
                    self.just_activated_digital.insert(input);
                }
            }
        } else {
            if value.abs() >= deadzone {
                self.just_activated.insert(input);
                self.just_deactivated.remove(&input);
            }
            if value.abs() >= deadzone_digital {
                self.just_activated_digital.insert(input);
                self.just_deactivated_digital.remove(&input);
            }
        }
    }

    pub(crate) fn update(&mut self) {
        self.just_activated.clear();
        self.just_deactivated.clear();
        self.just_activated_digital.clear();
        self.just_deactivated_digital.clear();
    }
}

impl<T> Default for AnalogInput<T> {
    fn default() -> Self {
        Self {
            inputs: Default::default(),

            just_activated: Default::default(),
            just_deactivated: Default::default(),
            deadzone: DEFAULT_DEADZONE,

            just_activated_digital: Default::default(),
            just_deactivated_digital: Default::default(),
            deadzone_digital: DEFAULT_DEADZONE_DIGITAL,
        }
    }
}

const DEFAULT_DEADZONE: Deadzone = Deadzone(0.1);
const DEFAULT_DEADZONE_DIGITAL: Deadzone = Deadzone(0.5);
