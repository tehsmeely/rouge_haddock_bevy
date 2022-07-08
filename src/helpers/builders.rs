use bevy::ecs::system::EntityCommands;

pub trait WithSelf {
    fn with_self(&mut self, f: impl FnOnce(&Self)) -> &mut Self;
    fn with_self_mut(&mut self, f: impl FnOnce(&mut Self)) -> &mut Self;
}

impl<'w, 's, 'a> WithSelf for EntityCommands<'w, 's, 'a> {
    fn with_self(&mut self, f: impl FnOnce(&Self)) -> &mut Self {
        f(self);
        self
    }

    fn with_self_mut(&mut self, f: impl FnOnce(&mut Self)) -> &mut Self {
        f(self);
        self
    }
}
