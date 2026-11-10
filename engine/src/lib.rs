//! Witch Client (Rust Edition): liga o ClickGUI de `witchcore` aos hooks do Horizon.
//!
//! ATENCAO: este arquivo foi escrito lendo docs/native-mods.md e o codigo do Horizon, mas NAO foi
//! compilado (o trait `Hooks` e gerado no build do Horizon). Os nomes marcados com `// CONFERIR`
//! sao os que mais provavelmente precisam de ajuste quando o launcher mostrar o erro de compilacao.
use horizon_client::mod_api::*;
use std::sync::Mutex;
use witchcore::{Canvas, ClickGui};

const MOD_ID: &str = "witch";
/// Tecla que abre o ClickGUI (Right Shift), no nome que o Horizon usa (`key.keyboard.right.shift`).
const OPEN_KEY: &str = "key.keyboard.right.shift";

pub fn hooks() -> Box<dyn Hooks> {
    Box::new(Witch { gui: Mutex::new(ClickGui::new()), size: Mutex::new([427.0, 240.0]) })
}

struct Witch {
    gui: Mutex<ClickGui>,
    /// Tamanho da tela em pixels de GUI, anotado no ultimo desenho (os hooks de clique nao o trazem).
    size: Mutex<[f32; 2]>,
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
    // CONFERIR: tipo do evento e campos (`key`, `pressed`) do hook `native_key`.
    fn key(&self, host: &mut dyn Host, event: &mut KeyEvent<'_>) {
        if event.pressed && event.key == OPEN_KEY {
            host.mod_screen_open(MOD_ID);
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
}
