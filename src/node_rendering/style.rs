use gpui::{linear_color_stop, linear_gradient, px, Hsla, Pixels};

pub fn body_bg() -> Hsla {
    // Opaque near-black to keep the body neutral and never color-bleed.
    Hsla {
        h: 0.0,
        s: 0.0,
        l: 0.065,
        a: 1.0,
    }
}

pub fn title_bg(node_color: Hsla) -> Hsla {
    Hsla {
        h: node_color.h,
        s: (node_color.s * 0.90).min(1.0),
        l: (node_color.l * 0.65).clamp(0.14, 0.44),
        a: 1.0,
    }
}

pub fn idle_border() -> Hsla {
    Hsla {
        h: 0.0,
        s: 0.0,
        l: 0.22,
        a: 1.0,
    }
}

pub fn separator_bg() -> Hsla {
    // Keep separator neutral-dark so the body remains visually dark.
    Hsla {
        h: 0.0,
        s: 0.0,
        l: 0.10,
        a: 1.0,
    }
}

pub fn label_color() -> Hsla {
    Hsla {
        h: 0.0,
        s: 0.0,
        l: 0.86,
        a: 1.0,
    }
}

pub fn corner_radius(z: f32) -> Pixels {
    px(7.0 * z)
}

pub fn title_pill_bg() -> Hsla {
    Hsla {
        h: 0.0,
        s: 0.0,
        l: 0.0,
        a: 0.30,
    }
}

pub fn header_shadow_gradient() -> gpui::Background {
    let transparent = Hsla {
        h: 0.0,
        s: 0.0,
        l: 0.0,
        a: 0.0,
    };
    let shadow_dark = Hsla {
        h: 0.0,
        s: 0.0,
        l: 0.0,
        a: 0.50,
    };

    linear_gradient(
        135.0,
        linear_color_stop(transparent, 0.0),
        linear_color_stop(shadow_dark, 1.0),
    )
}
