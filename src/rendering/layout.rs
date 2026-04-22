/// Layout constants for node rendering.
/// All values are unscaled — multiply by zoom_level before converting to pixels.
///
/// Node height formula:
///   HEADER_H + SEP_H + BODY_PAD*2 + max_pins * PIN_ROW_H + (max_pins-1) * PIN_GAP

pub const HEADER_H: f32 = 27.0;
pub const SEP_H: f32    = 1.0;
pub const BODY_PAD: f32 = 8.0;
pub const PIN_ROW_H: f32 = 16.0;
pub const PIN_GAP: f32  = 4.0;
pub const PIN_SIZE: f32 = 12.0;

pub const NODE_BASE_H: f32 = HEADER_H + SEP_H + BODY_PAD * 2.0;

pub fn node_height_for_pin_rows(pin_rows: usize) -> f32 {
    let rows = pin_rows.max(1) as f32;
    NODE_BASE_H + rows * PIN_ROW_H + ((rows - 1.0).max(0.0)) * PIN_GAP
}
