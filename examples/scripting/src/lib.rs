use fyrox::{
    core::{
        algebra::{UnitQuaternion, Vector3},
        inspect::{Inspect, PropertyInfo},
        pool::Handle,
        uuid::Uuid,
        visitor::prelude::*,
    },
    event::{DeviceEvent, ElementState, Event, VirtualKeyCode, WindowEvent},
    gui::inspector::{FieldKind, PropertyChanged},
    plugin::{Plugin, PluginContext},
    scene::{
        node::{Node, TypeUuidProvider},
        rigidbody::RigidBody,
    },
    script::{ScriptContext, ScriptTrait},
};
use std::str::FromStr;

#[derive(Visit, Inspect, Default)]
struct GamePlugin {}

impl GamePlugin {
    pub fn new() -> Self {
        Self {}
    }
}

impl TypeUuidProvider for GamePlugin {
    fn type_uuid() -> Uuid {
        Uuid::from_str("a9507fb2-0945-4fc1-91ce-115ae7c8a615").unwrap()
    }
}

impl Plugin for GamePlugin {
    fn on_init(&mut self, engine: &mut PluginContext) {
        engine
            .serialization_context
            .script_constructors
            .add::<GamePlugin, Player, &str>("Player");

        engine
            .serialization_context
            .script_constructors
            .add::<GamePlugin, Jumper, &str>("Jumper");
    }

    fn on_unload(&mut self, _context: &mut PluginContext) {}

    fn update(&mut self, _context: &mut PluginContext) {}

    fn id(&self) -> Uuid {
        Self::type_uuid()
    }
}

#[derive(Default, Debug, Clone)]
pub struct InputController {
    walk_forward: bool,
    walk_backward: bool,
    walk_left: bool,
    walk_right: bool,
    jump: bool,
}

#[derive(Visit, Inspect, Debug, Clone)]
struct Player {
    speed: f32,
    yaw: f32,

    #[visit(optional)]
    pitch: f32,

    #[visit(optional)]
    camera: Handle<Node>,

    #[visit(skip)]
    #[inspect(skip)]
    controller: InputController,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            speed: 0.1,
            yaw: 0.0,
            pitch: 0.0,
            camera: Default::default(),
            controller: Default::default(),
        }
    }
}

impl TypeUuidProvider for Player {
    fn type_uuid() -> Uuid {
        Uuid::from_str("4aa165aa-011b-479f-bc10-b90b2c4b5060").unwrap()
    }
}

impl ScriptTrait for Player {
    fn on_property_changed(&mut self, args: &PropertyChanged) {
        if let FieldKind::Object(ref value) = args.value {
            match args.name.as_ref() {
                Self::SPEED => self.speed = value.cast_clone().unwrap(),
                Self::YAW => self.yaw = value.cast_clone().unwrap(),
                Self::CAMERA => self.camera = value.cast_clone().unwrap(),
                _ => (),
            }
        }
    }

    fn on_init(&mut self, context: ScriptContext) {
        let ScriptContext { node, scene, .. } = context;

        for &child in node.children() {
            if scene.graph[child].name() == "Camera" {
                self.camera = child;
                break;
            }
        }
    }

    fn on_update(&mut self, context: ScriptContext) {
        let ScriptContext {
            dt, node, scene, ..
        } = context;

        node.local_transform_mut()
            .set_rotation(UnitQuaternion::from_axis_angle(
                &Vector3::y_axis(),
                self.yaw,
            ));

        if let Some(body) = node.cast_mut::<RigidBody>() {
            let look_vector = body
                .look_vector()
                .try_normalize(f32::EPSILON)
                .unwrap_or_else(Vector3::z);

            let side_vector = body
                .side_vector()
                .try_normalize(f32::EPSILON)
                .unwrap_or_else(Vector3::x);

            let mut velocity = Vector3::default();

            if self.controller.walk_right {
                velocity -= side_vector;
            }
            if self.controller.walk_left {
                velocity += side_vector;
            }
            if self.controller.walk_forward {
                velocity += look_vector;
            }
            if self.controller.walk_backward {
                velocity -= look_vector;
            }

            let speed = 2.0 * dt;
            let velocity = velocity
                .try_normalize(f32::EPSILON)
                .map(|v| v.scale(speed))
                .unwrap_or_default();

            body.set_ang_vel(Default::default());
            body.set_lin_vel(Vector3::new(
                velocity.x / dt,
                body.lin_vel().y,
                velocity.z / dt,
            ));
        }

        if let Some(camera) = scene.graph.try_get_mut(self.camera) {
            camera
                .local_transform_mut()
                .set_rotation(UnitQuaternion::from_axis_angle(
                    &Vector3::x_axis(),
                    self.pitch,
                ));
        }
    }

    fn on_os_event(&mut self, event: &Event<()>, _context: ScriptContext) {
        match event {
            Event::DeviceEvent { event, .. } => {
                if let DeviceEvent::MouseMotion { delta } = event {
                    let mouse_sens = 0.025;

                    self.yaw -= mouse_sens * delta.0 as f32;
                    self.pitch = (self.pitch + (delta.1 as f32) * mouse_sens)
                        .max(-90.0f32.to_radians())
                        .min(90.0f32.to_radians());
                }
            }
            Event::WindowEvent { event, .. } => {
                if let WindowEvent::KeyboardInput { input, .. } = event {
                    if let Some(key_code) = input.virtual_keycode {
                        match key_code {
                            VirtualKeyCode::W => {
                                self.controller.walk_forward = input.state == ElementState::Pressed
                            }
                            VirtualKeyCode::S => {
                                self.controller.walk_backward = input.state == ElementState::Pressed
                            }
                            VirtualKeyCode::A => {
                                self.controller.walk_left = input.state == ElementState::Pressed
                            }
                            VirtualKeyCode::D => {
                                self.controller.walk_right = input.state == ElementState::Pressed
                            }
                            VirtualKeyCode::Space => {
                                self.controller.jump = input.state == ElementState::Pressed
                            }
                            _ => (),
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn id(&self) -> Uuid {
        Self::type_uuid()
    }

    fn plugin_uuid(&self) -> Uuid {
        GamePlugin::type_uuid()
    }
}

#[derive(Visit, Inspect, Debug, Clone, Default)]
struct Jumper {
    timer: f32,

    #[visit(optional)]
    period: f32,
}

impl TypeUuidProvider for Jumper {
    fn type_uuid() -> Uuid {
        Uuid::from_str("942e9f5b-e036-4357-b514-91060d4059f5").unwrap()
    }
}

impl ScriptTrait for Jumper {
    fn on_property_changed(&mut self, args: &PropertyChanged) {
        if let FieldKind::Object(ref value) = args.value {
            match args.name.as_ref() {
                Self::TIMER => self.timer = value.cast_clone().unwrap(),
                Self::PERIOD => self.period = value.cast_clone().unwrap(),
                _ => (),
            }
        }
    }

    fn on_init(&mut self, _context: ScriptContext) {}

    fn on_update(&mut self, context: ScriptContext) {
        if let Some(rigid_body) = context.node.cast_mut::<RigidBody>() {
            if self.timer > 0.6 {
                rigid_body.apply_force(Vector3::new(0.0, 200.0, 0.0));
                self.timer = 0.0;
            }

            self.timer += context.dt;
        }
    }

    fn id(&self) -> Uuid {
        Self::type_uuid()
    }

    fn plugin_uuid(&self) -> Uuid {
        GamePlugin::type_uuid()
    }
}

// Script entry point.
#[no_mangle]
pub extern "C" fn fyrox_main() -> Box<Box<dyn Plugin>> {
    Box::new(Box::new(GamePlugin::new()))
}
