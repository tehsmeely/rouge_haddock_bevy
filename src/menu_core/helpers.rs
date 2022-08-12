use bevy::ui::{UiRect, Val};

pub trait RectExt {
    fn new_2(v_topbottom: Val, v_leftright: Val) -> Self;
}

impl RectExt for UiRect<Val> {
    fn new_2(v_topbottom: Val, v_leftright: Val) -> Self {
        UiRect {
            left: v_leftright,
            right: v_leftright,
            top: v_topbottom,
            bottom: v_topbottom,
        }
    }
}
