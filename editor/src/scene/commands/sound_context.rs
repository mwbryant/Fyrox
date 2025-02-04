use crate::{Command, SceneContext};
use fyrox::scene::sound::{context::SoundContext, DistanceModel, Renderer};

macro_rules! define_sound_context_command {
    ($($name:ident($human_readable_name:expr, $value_type:ty, $get:ident, $set:ident); )*) => {
        $(
            #[derive(Debug)]
            pub struct $name {
                value: $value_type,
            }

            impl $name {
                pub fn new(value: $value_type) -> Self {
                    Self { value }
                }

                fn swap(&mut self, sound_context: &mut SoundContext) {
                    let old = sound_context.state().$get();
                    sound_context.state().$set(self.value.clone());
                    self.value = old;
                }
            }

            impl Command for $name {
                fn name(&mut self, _context: &SceneContext) -> String {
                    $human_readable_name.to_owned()
                }

                fn execute(&mut self, context: &mut SceneContext) {
                    self.swap(&mut context.scene.graph.sound_context);
                }

                fn revert(&mut self, context: &mut SceneContext) {
                    self.swap(&mut context.scene.graph.sound_context);
                }
            }
        )*
    };
}

define_sound_context_command! {
    SetDistanceModelCommand("Set Distance Model", DistanceModel, distance_model, set_distance_model);
    SetRendererCommand("Set Renderer", Renderer, renderer, set_renderer);
}
