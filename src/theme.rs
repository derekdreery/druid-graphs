use druid::{Color, Env, Key};

/// Used in a number of places to give small spacing.
pub const MARGIN: Key<f64> = Key::new("org.derekdreery.druid-graphs.theme.margin");
pub const BAR_SPACING: Key<f64> = Key::new("org.derekdreery.druid-graphs.theme.bar_spacing");
pub const AXES_COLOR: Key<Color> = Key::new("org.derekdreery.druid-graphs.theme.axes_color");

/// Important: call this before doing anything else.
pub fn add_to_env(env: &mut Env) {
    env.set(MARGIN, 6.);
    env.set(BAR_SPACING, 10.);
    env.set(AXES_COLOR, Color::grey(0.8));
}
