//! Nucleo do Witch (Rust Edition): ClickGUI sem nenhuma dependencia do Horizon.
//! Desenha por um `Canvas` e recebe mouse/scroll em pixels de GUI, entao da para testar com `cargo test`.

pub const ACCENT: u32 = 0xFF8B7FD6;
const BG: u32 = 0xF2101018;
const PANEL: u32 = 0xFF181822;
const ROW: u32 = 0xFF20202C;
const ROW_HOVER: u32 = 0xFF2A2A3A;
const OFF: u32 = 0xFF3A3A4A;
const TEXT: u32 = 0xFFE8E8F0;
const DIM: u32 = 0xFF9A94B8;

pub trait Canvas {
    fn fill(&mut self, x: f32, y: f32, w: f32, h: f32, argb: u32);
    fn text(&mut self, x: f32, y: f32, text: &str, rgb: u32, shadow: bool);
    fn text_width(&self, text: &str) -> f32;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Combat,
    Movement,
    Visuals,
    World,
    Utility,
    Hud,
}

impl Category {
    pub const ALL: [Category; 6] = [
        Category::Combat,
        Category::Movement,
        Category::Visuals,
        Category::World,
        Category::Utility,
        Category::Hud,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Category::Combat => "Combat",
            Category::Movement => "Movement",
            Category::Visuals => "Visuals",
            Category::World => "World",
            Category::Utility => "Utility",
            Category::Hud => "HUD",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Module {
    pub name: &'static str,
    pub category: Category,
    pub on: bool,
}

/// Modulos planejados. Por enquanto so ligam/desligam o estado: nenhum faz efeito no jogo ainda.
pub fn default_modules() -> Vec<Module> {
    use Category::*;
    let m = |name, category| Module { name, category, on: false };
    vec![
        m("Radar", Visuals),
        m("ESP", Visuals),
        m("Tracers", Visuals),
        m("Fullbright", Visuals),
        m("Zoom", Visuals),
        m("FPS", Hud),
        m("Coordinates", Hud),
        m("Keystrokes", Hud),
        m("Module List", Hud),
        m("Auto Sprint", Movement),
        m("Auto Walk", Movement),
        m("Auto Respawn", Utility),
        m("Chat Macro", Utility),
        m("Item Counter", Utility),
        m("Waypoints", World),
        m("Block Info", World),
    ]
}

const W: f32 = 360.0;
const H: f32 = 220.0;
const HEAD: f32 = 26.0;
const TABS: f32 = 84.0;
const TAB_H: f32 = 24.0;
const ROW_H: f32 = 24.0;
const PAD: f32 = 6.0;

#[derive(Clone, Copy, Debug)]
struct Rect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl Rect {
    fn has(&self, p: [f32; 2]) -> bool {
        p[0] >= self.x && p[0] < self.x + self.w && p[1] >= self.y && p[1] < self.y + self.h
    }
}

pub struct ClickGui {
    pub modules: Vec<Module>,
    pub category: Category,
    scroll: f32,
}

impl Default for ClickGui {
    fn default() -> Self {
        Self::new()
    }
}

impl ClickGui {
    pub fn new() -> Self {
        Self { modules: default_modules(), category: Category::Visuals, scroll: 0.0 }
    }

    pub fn enabled(&self, name: &str) -> bool {
        self.modules.iter().any(|m| m.name == name && m.on)
    }

    fn window(screen: [f32; 2]) -> Rect {
        let w = W.min(screen[0] - 20.0).max(200.0);
        let h = H.min(screen[1] - 20.0).max(120.0);
        Rect { x: ((screen[0] - w) / 2.0).floor(), y: ((screen[1] - h) / 2.0).floor(), w, h }
    }

    fn list_area(win: Rect) -> Rect {
        Rect { x: win.x + TABS, y: win.y + HEAD, w: win.w - TABS, h: win.h - HEAD }
    }

    fn visible(&self) -> Vec<usize> {
        (0..self.modules.len()).filter(|&i| self.modules[i].category == self.category).collect()
    }

    fn max_scroll(&self, area: Rect) -> f32 {
        (self.visible().len() as f32 * ROW_H + PAD * 2.0 - area.h).max(0.0)
    }

    fn row_rect(area: Rect, scroll: f32, n: usize) -> Rect {
        Rect { x: area.x + PAD, y: area.y + PAD + n as f32 * ROW_H - scroll, w: area.w - PAD * 2.0, h: ROW_H - 4.0 }
    }

    pub fn draw(&self, c: &mut dyn Canvas, mouse: [f32; 2], screen: [f32; 2]) {
        let win = Self::window(screen);
        c.fill(win.x, win.y, win.w, win.h, BG);
        c.fill(win.x, win.y, win.w, 2.0, ACCENT);
        c.text(win.x + 8.0, win.y + 9.0, "Witch Client", TEXT, true);
        let ver = "Rust Edition";
        let vw = c.text_width(ver);
        c.text(win.x + win.w - vw - 8.0, win.y + 9.0, ver, DIM, false);

        c.fill(win.x, win.y + HEAD, TABS, win.h - HEAD, PANEL);
        for (i, cat) in Category::ALL.iter().enumerate() {
            let r = Rect { x: win.x + 4.0, y: win.y + HEAD + 4.0 + i as f32 * TAB_H, w: TABS - 8.0, h: TAB_H - 2.0 };
            let sel = *cat == self.category;
            if sel {
                c.fill(r.x, r.y, r.w, r.h, ROW_HOVER);
                c.fill(r.x, r.y, 2.0, r.h, ACCENT);
            } else if r.has(mouse) {
                c.fill(r.x, r.y, r.w, r.h, ROW);
            }
            c.text(r.x + 8.0, r.y + 7.0, cat.name(), if sel { TEXT } else { DIM }, false);
        }

        let area = Self::list_area(win);
        let vis = self.visible();
        if vis.is_empty() {
            c.text(area.x + 10.0, area.y + 10.0, "Nenhum modulo aqui ainda", DIM, false);
        }
        for (n, &i) in vis.iter().enumerate() {
            let r = Self::row_rect(area, self.scroll, n);
            // so desenha linhas inteiras dentro da area (sem scissor no Canvas)
            if r.y < area.y || r.y + r.h > area.y + area.h {
                continue;
            }
            let m = &self.modules[i];
            c.fill(r.x, r.y, r.w, r.h, if r.has(mouse) { ROW_HOVER } else { ROW });
            c.text(r.x + 8.0, r.y + 8.0, m.name, TEXT, false);
            let (px, py) = (r.x + r.w - 30.0, r.y + 5.0);
            c.fill(px, py, 22.0, 12.0, if m.on { ACCENT } else { OFF });
            let kx = if m.on { px + 12.0 } else { px + 2.0 };
            c.fill(kx, py + 2.0, 8.0, 8.0, TEXT);
        }
        if self.max_scroll(area) > 0.0 {
            let total = vis.len() as f32 * ROW_H + PAD * 2.0;
            let bh = (area.h * area.h / total).max(12.0);
            let by = area.y + (area.h - bh) * (self.scroll / self.max_scroll(area));
            c.fill(win.x + win.w - 3.0, by, 2.0, bh, ACCENT);
        }
    }

    /// Clique esquerdo. Devolve true se a GUI usou o clique (dentro da janela).
    pub fn click(&mut self, mouse: [f32; 2], screen: [f32; 2], button: i32) -> bool {
        let win = Self::window(screen);
        if !win.has(mouse) {
            return false;
        }
        if button != 0 {
            return true;
        }
        for (i, cat) in Category::ALL.iter().enumerate() {
            let r = Rect { x: win.x + 4.0, y: win.y + HEAD + 4.0 + i as f32 * TAB_H, w: TABS - 8.0, h: TAB_H - 2.0 };
            if r.has(mouse) {
                if self.category != *cat {
                    self.category = *cat;
                    self.scroll = 0.0;
                }
                return true;
            }
        }
        let area = Self::list_area(win);
        if area.has(mouse) {
            for (n, i) in self.visible().into_iter().enumerate() {
                let r = Self::row_rect(area, self.scroll, n);
                if r.y >= area.y && r.y + r.h <= area.y + area.h && r.has(mouse) {
                    self.modules[i].on = !self.modules[i].on;
                    break;
                }
            }
        }
        true
    }

    /// Roda do mouse / arrastar com o dedo: `dy` positivo rola para cima (como a roda do mouse).
    pub fn scroll(&mut self, mouse: [f32; 2], screen: [f32; 2], dy: f32) -> bool {
        let win = Self::window(screen);
        if !win.has(mouse) {
            return false;
        }
        let area = Self::list_area(win);
        self.scroll = (self.scroll - dy * ROW_H).clamp(0.0, self.max_scroll(area));
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Null;
    impl Canvas for Null {
        fn fill(&mut self, _: f32, _: f32, _: f32, _: f32, _: u32) {}
        fn text(&mut self, _: f32, _: f32, _: &str, _: u32, _: bool) {}
        fn text_width(&self, t: &str) -> f32 {
            t.len() as f32 * 6.0
        }
    }

    const SCREEN: [f32; 2] = [427.0, 240.0];

    fn tab_center(i: usize) -> [f32; 2] {
        let win = ClickGui::window(SCREEN);
        [win.x + 10.0, win.y + HEAD + 4.0 + i as f32 * TAB_H + 5.0]
    }

    fn row_center(g: &ClickGui, n: usize) -> [f32; 2] {
        let win = ClickGui::window(SCREEN);
        let r = ClickGui::row_rect(ClickGui::list_area(win), g.scroll, n);
        [r.x + 20.0, r.y + 5.0]
    }

    #[test]
    fn click_tab_changes_category() {
        let mut g = ClickGui::new();
        assert!(g.click(tab_center(1), SCREEN, 0));
        assert_eq!(g.category, Category::Movement);
    }

    #[test]
    fn click_row_toggles_module() {
        let mut g = ClickGui::new();
        g.category = Category::Hud;
        assert!(!g.enabled("FPS"));
        g.click(row_center(&g, 0), SCREEN, 0);
        assert!(g.enabled("FPS"));
        g.click(row_center(&g, 0), SCREEN, 0);
        assert!(!g.enabled("FPS"));
    }

    #[test]
    fn click_outside_is_not_used() {
        let mut g = ClickGui::new();
        assert!(!g.click([1.0, 1.0], SCREEN, 0));
    }

    #[test]
    fn right_click_does_not_toggle() {
        let mut g = ClickGui::new();
        g.category = Category::Hud;
        g.click(row_center(&g, 0), SCREEN, 1);
        assert!(!g.enabled("FPS"));
    }

    #[test]
    fn scroll_is_clamped() {
        let mut g = ClickGui::new();
        let win = ClickGui::window(SCREEN);
        let inside = [win.x + 100.0, win.y + 100.0];
        g.scroll(inside, SCREEN, 50.0);
        assert_eq!(g.scroll, 0.0);
        g.scroll(inside, SCREEN, -50.0);
        assert!(g.scroll <= g.max_scroll(ClickGui::list_area(win)));
    }

    #[test]
    fn small_screen_window_fits() {
        let w = ClickGui::window([240.0, 140.0]);
        assert!(w.x >= 0.0 && w.y >= 0.0 && w.x + w.w <= 240.0 && w.y + w.h <= 140.0);
    }

    #[test]
    fn draw_runs_for_every_category() {
        let mut g = ClickGui::new();
        for c in Category::ALL {
            g.category = c;
            g.draw(&mut Null, [0.0, 0.0], SCREEN);
        }
    }
}
