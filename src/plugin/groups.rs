use super::{GameApp, Plugin, PluginGroup};

impl<P: Plugin> PluginGroup for P {
    fn build(self, app: &mut GameApp) {
        app.add_plugin(self);
    }
}

impl<A: PluginGroup, B: PluginGroup> PluginGroup for (A, B) {
    fn build(self, app: &mut GameApp) {
        self.0.build(app);
        self.1.build(app);
    }
}

impl<A: PluginGroup, B: PluginGroup, C: PluginGroup> PluginGroup for (A, B, C) {
    fn build(self, app: &mut GameApp) {
        self.0.build(app);
        self.1.build(app);
        self.2.build(app);
    }
}
