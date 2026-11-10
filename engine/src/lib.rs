//! Witch Client (Rust Edition): liga o ClickGUI de `witchcore` aos hooks do Horizon.
//!
//! ATENCAO: este arquivo foi escrito lendo docs/native-mods.md e o codigo do Horizon, mas NAO foi
//! compilado (o trait `Hooks` e gerado no build do Horizon). Os nomes marcados com `// CONFERIR`
//! sao os que mais provavelmente precisam de ajuste quando o launcher mostrar o erro de compilacao.
use horizon_client::mod_api::*;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use witchcore::blocks::{self, World};
use witchcore::fly::{self, FlyInput};
use witchcore::math::{self, V3};
use witchcore::trigger::{self, Target};
use witchcore::{Canvas, ClickGui};

const MOD_ID: &str = "witch";
/// Tecla que abre o ClickGUI (Right Shift), no nome que o Horizon usa (`key.keyboard.right.shift`).
/// Moldura do menu (assets/frame.png) como pixels RGBA crus: 640x356, sem depender de crate de PNG.
const FRAME_RGBA: &[u8] = include_bytes!("../assets/frame.rgba");
const FRAME_W: u32 = 640;
const FRAME_H: u32 = 356;
/// Nome do sprite onde `mod_image` coloca a imagem (MOD_IMAGE_SPRITE do Horizon).
const IMAGE_SPRITE: &str = "horizon/modimage";
/// Nomes de tecla do Horizon (`native_key`).
const K_FWD: &str = "key.keyboard.w";
const K_BACK: &str = "key.keyboard.s";
const K_LEFT: &str = "key.keyboard.a";
const K_RIGHT: &str = "key.keyboard.d";
const K_UP: &str = "key.keyboard.space";
const K_DOWN: &str = "key.keyboard.left.shift";
const REACH: f32 = 3.0;
const SCAN_RADIUS: i32 = 16;
const SCAN_EVERY: u32 = 20;
const OPEN_KEY: &str = "key.keyboard.right.shift";

pub fn hooks() -> Box<dyn Hooks> {
    Box::new(Witch { gui: Mutex::new(ClickGui::new()), size: Mutex::new([427.0, 240.0]), uploaded: AtomicBool::new(false), live: Mutex::new(Live::default()) })
}

struct Witch {
    gui: Mutex<ClickGui>,
    /// Tamanho da tela em pixels de GUI, anotado no ultimo desenho (os hooks de clique nao o trazem).
    size: Mutex<[f32; 2]>,
    /// A moldura ja foi enviada ao atlas do HUD.
    uploaded: AtomicBool,
    live: Mutex<Live>,
}

/// Estado dos modulos que roda a cada tick.
#[derive(Default)]
struct Live {
    keys: FlyInput,
    last_pos: Option<V3>,
    ticks: u32,
    trigger: trigger::Timer,
    ores: Vec<([i32; 3], u32)>,
}

struct Pen<'a> {
    host: &'a dyn Host,
    batch: &'a mut HudBatch,
}

impl Canvas for Pen<'_> {
    fn fill(&mut self, x: f32, y: f32, w: f32, h: f32, argb: u32) {
        self.host.fill(&mut *self.batch, [x, y, w, h], argb);
    }
    fn text(&mut self, x: f32, y: f32, text: &str, rgb: u32, shadow: bool) {
        self.host.text(&mut *self.batch, text, [x, y], rgb, shadow);
    }
    fn text_width(&self, text: &str) -> f32 {
        self.host.text_width(text)
    }
    fn item_icon(&mut self, name: &str, x: f32, y: f32) -> bool {
        self.host.item(&mut *self.batch, name, [x, y], 16.0)
    }
    fn sprite(&mut self, src: [f32; 4], dst: [f32; 4]) -> bool {
        // CONFERIR: nome/assinatura de `Host::sprite` (native_sprite: batch, name, src, dst).
        self.host.sprite(&mut *self.batch, IMAGE_SPRITE, src, dst)
    }
}

impl Witch {
    fn mine(&self, host: &dyn Host) -> bool {
        host.mod_screen().is_some_and(|id| id == MOD_ID)
    }
    fn size(&self) -> [f32; 2] {
        *self.size.lock().unwrap()
    }
}

impl Hooks for Witch {
    // CONFERIR: `mod_image` precisa de `&mut dyn Host`; o hook `hud` da (como no exemplo native_gui_overlay_probe).
    fn hud(&self, host: &mut dyn Host, _: &mut HudEvent<'_>) {
        if !self.uploaded.load(Ordering::Relaxed) && host.mod_image(FRAME_W, FRAME_H, FRAME_RGBA) {
            self.uploaded.store(true, Ordering::Relaxed);
        }
    }

    // CONFERIR: tipo do evento e campos (`key`, `pressed`) do hook `native_key`.
    fn key(&self, host: &mut dyn Host, event: &mut KeyEvent<'_>) {
        if event.pressed && event.key == OPEN_KEY {
            host.mod_screen_open(MOD_ID);
        }
        let mut live = self.live.lock().unwrap();
        let down = event.pressed;
        match event.key {
            K_FWD => live.keys.forward = down,
            K_BACK => live.keys.back = down,
            K_LEFT => live.keys.left = down,
            K_RIGHT => live.keys.right = down,
            K_UP => live.keys.up = down,
            K_DOWN => live.keys.down = down,
            _ => {}
        }
    }

    fn screen_render_post(&self, host: &dyn Host, event: &mut ScreenRenderPostEvent<'_>) {
        if event.screen.id != "mod" || !self.mine(host) {
            return;
        }
        let screen = [event.batch.width, event.batch.height];
        *self.size.lock().unwrap() = screen;
        let mouse = event.screen.mouse;
        let mut pen = Pen { host, batch: &mut *event.batch };
        self.gui.lock().unwrap().draw(&mut pen, mouse, screen);
    }

    // CONFERIR: tipo do evento; `canceled` deve ser `&mut bool` (parametro `canceled` do hook).
    fn screen_mouse_button_pressed_pre(
        &self,
        host: &dyn Host,
        event: &mut ScreenMouseButtonPressedPreEvent<'_>,
    ) {
        if event.screen.id != "mod" || !self.mine(host) {
            return;
        }
        let used = self.gui.lock().unwrap().click(event.screen.mouse, self.size(), event.button);
        if used {
            *event.canceled = true;
        }
    }

    // CONFERIR: tipo do evento; `scroll` e [f64; 2] (x, y).
    fn screen_mouse_scrolled_pre(&self, host: &dyn Host, event: &mut ScreenMouseScrolledPreEvent<'_>) {
        if event.screen.id != "mod" || !self.mine(host) {
            return;
        }
        let used = self.gui.lock().unwrap().scroll(event.screen.mouse, self.size(), event.scroll[1] as f32);
        if used {
            *event.canceled = true;
        }
    }

    fn tick(&self, host: &mut dyn Host) {
        // This hook also runs on the integrated server; these modules are client-only.
        if !host.window() {
            return;
        }
        let (fly_on, trigger_on, block_on) = {
            let g = self.gui.lock().unwrap();
            (g.enabled("Fly"), g.enabled("Trigger Bot"), g.enabled("Block ESP"))
        };
        let Some(me) = host.player() else { return };
        let pos: V3 = [me.x as f32, me.y as f32, me.z as f32];
        let mut live = self.live.lock().unwrap();
        live.ticks = live.ticks.wrapping_add(1);
        let vel = live.last_pos.map(|p| [pos[0] - p[0], pos[1] - p[1], pos[2] - p[2]]).unwrap_or([0.0; 3]);
        live.last_pos = Some(pos);

        if fly_on && host.mod_screen().is_none() {
            live.keys.yaw_deg = me.yaw;
            // CONFERIR: `Host::push_player` (native_push_player), em blocos por tick.
            host.push_player(fly::push(&live.keys, vel, 0.45));
        }

        let aiming = if trigger_on && host.mod_screen().is_none() {
            let eye = [pos[0], pos[1] + 1.62, pos[2]];
            let dir = math::look_dir(me.yaw, me.pitch);
            // CONFERIR: `Host::client_entities` (native_client_entities); `position` como [f64; 3].
            let targets: Vec<Target> = host
                .client_entities()
                .iter()
                .filter(|e| e.alive && !e.invisible && !e.spectator && e.player_uuid.is_some())
                .map(|e| {
                    let p = e.position;
                    let (x, y, z) = (p[0] as f32, p[1] as f32, p[2] as f32);
                    let h = e.height as f32;
                    Target { id: e.id, min: [x - 0.3, y, z - 0.3], max: [x + 0.3, y + h, z + 0.3] }
                })
                .collect();
            trigger::pick(eye, dir, &targets, REACH)
        } else {
            None
        };
        // ~12 ticks de cooldown (espada): ataca no ritmo, nao a cada tick
        // Trigger Bot: a API do Horizon nao tem ataque do cliente; o pacote Attack exigiria depender de
        // horizon-protocol (fora do Cargo.lock permitido). Por isso so detecta o alvo, nao ataca.
        let _ = live.trigger.tick(aiming.is_some(), 12);

        if block_on && live.ticks % SCAN_EVERY == 0 {
            let sel = self.gui.lock().unwrap().blocks.clone();
            let c = [pos[0].floor() as i32, pos[1].floor() as i32, pos[2].floor() as i32];
            let r = SCAN_RADIUS;
            struct W<'a>(&'a dyn Host);
            impl World for W<'_> {
                fn name_at(&self, p: [i32; 3]) -> Option<String> {
                    let name = self.0.block(p);
                    (name != "minecraft:air").then_some(name)
                }
            }
            live.ores = blocks::scan(&W(&*host), c, r, &sel, 400);
        } else if !block_on {
            live.ores.clear();
        }
    }

    fn gui_render_post(&self, host: &dyn Host, event: &mut GuiRenderPostEvent<'_>) {
        let (esp, tracers, ores_on) = {
            let g = self.gui.lock().unwrap();
            (g.enabled("Player ESP") || g.enabled("ESP + Tracers"), g.enabled("Tracers") || g.enabled("ESP + Tracers"), g.enabled("Block ESP"))
        };
        if !(esp || tracers || ores_on) || event.gui_hidden {
            return;
        }
        let Some(cam) = host.camera() else { return };
        let screen = [event.batch.width, event.batch.height];
        let cp = cam.position;
        let eye = [cp[0] as f32, cp[1] as f32 + cam.eye_height, cp[2] as f32];
        // CONFERIR: `position` em `NativeCamera` e o pe ou o olho? Aqui: pe + eye_height.
        let rel = |p: V3| [p[0] - eye[0], p[1] - eye[1], p[2] - eye[2]];
        let proj = |p: V3| math::project(&cam.view, &cam.projection, rel(p), screen);
        let mut lines: Vec<([f32; 2], [f32; 2], u32)> = Vec::new();
        let origin = [screen[0] * 0.5, screen[1]];

        if esp || tracers {
            for e in host.client_entities().iter().filter(|e| e.alive && !e.invisible && !e.spectator && e.player_uuid.is_some()) {
                let p = e.position;
                let (x, y, z) = (p[0] as f32, p[1] as f32, p[2] as f32);
                let (min, max) = ([x - 0.3, y, z - 0.3], [x + 0.3, y + e.height as f32, z + 0.3]);
                if esp {
                    for (a, b) in math::box_edges(min, max) {
                        if let (Some(a), Some(b)) = (proj(a), proj(b)) {
                            lines.push((a, b, 0xFFB7ACF0));
                        }
                    }
                }
                if tracers {
                    if let Some(t) = proj([x, y + e.height as f32 * 0.5, z]) {
                        lines.push((origin, t, 0xFF8B7FD6));
                    }
                }
            }
        }
        if ores_on {
            for (pos, color) in self.live.lock().unwrap().ores.iter() {
                let (min, max) = blocks::box_of(*pos);
                for (a, b) in math::box_edges(min, max) {
                    if let (Some(a), Some(b)) = (proj(a), proj(b)) {
                        lines.push((a, b, *color));
                    }
                }
            }
        }
        for (a, b, color) in lines {
            // CONFERIR: `Host::quad` (native_quad: batch, points, argb).
            host.quad(&mut *event.batch, math::line_quad(a, b, 1.2), color);
        }
    }
}
