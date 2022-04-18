use bevy::math::Rect;
use bevy::ui::Val;

pub trait RectExt {
    fn new_2(v_topbottom: Val, v_leftright: Val) -> Self;
}

impl RectExt for Rect<Val> {
    fn new_2(v_topbottom: Val, v_leftright: Val) -> Self {
        Rect {
            left: v_leftright,
            right: v_leftright,
            top: v_topbottom,
            bottom: v_topbottom,
        }
    }
}
