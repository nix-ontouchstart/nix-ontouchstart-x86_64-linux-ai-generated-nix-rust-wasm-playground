use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, MouseEvent};
use std::cell::RefCell;
use std::rc::Rc;

// A simple pseudo-random number generator to avoid extra crate dependencies
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    // Returns a f64 between 0.0 and 1.0
    fn next_f64(&mut self) -> f64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.state >> 11) as f64 / (1u64 << 53) as f64
    }
}

struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    radius: f64,
    color: String,
}

struct Engine {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    particles: Vec<Particle>,
    mouse_x: f64,
    mouse_y: f64,
    rng: SimpleRng,
}

impl Engine {
    fn new(canvas: HtmlCanvasElement) -> Self {
        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        let rng = SimpleRng::new(42);
        let mut particles = Vec::new();
        
        // Seed 150 unique color particles
        for i in 0..150 {
            let hue = (i * 360 / 150) as u32;
            particles.push(Particle {
                x: 400.0,
                y: 300.0,
                vx: (i as f64 * 0.1).cos() * 2.0,
                vy: (i as f64 * 0.1).sin() * 2.0,
                radius: 2.0 + (i % 4) as f64,
                color: format!("hsla({}, 85%, 65%, 0.8)", hue),
            });
        }

        Self {
            canvas,
            context,
            particles,
            mouse_x: -1000.0,
            mouse_y: -1000.0,
            rng,
        }
    }

    fn update_and_render(&mut self) {
        let width = self.canvas.width() as f64;
        let height = self.canvas.height() as f64;

        // Semi-transparent background creates a neon trailing tail effect
        #[allow(deprecated)]
        self.context.set_fill_style(&JsValue::from_str("rgba(11, 15, 25, 0.2)"));
        self.context.fill_rect(0.0, 0.0, width, height);

        for p in self.particles.iter_mut() {
            // Simple physics step
            p.x += p.vx;
            p.y += p.vy;

            // Gravitational pull toward mouse if close enough
            let dx = self.mouse_x - p.x;
            let dy = self.mouse_y - p.y;
            let distance = (dx * dx + dy * dy).sqrt();
            if distance < 300.0 && distance > 5.0 {
                p.vx += (dx / distance) * 0.15;
                p.vy += (dy / distance) * 0.15;
            }

            // Apply friction drag cap
            p.vx *= 0.98;
            p.vy *= 0.98;

            // Bounce mechanics off screen walls
            if p.x < 0.0 || p.x > width { p.vx *= -1.0; }
            if p.y < 0.0 || p.y > height { p.vy *= -1.0; }

            // Render individual particle circles
            self.context.begin_path();
            self.context.arc(p.x, p.y, p.radius, 0.0, std::f64::consts::TAU).unwrap();
            #[allow(deprecated)]
            self.context.set_fill_style(&JsValue::from_str(&p.color));
            self.context.fill();
        }
    }

    fn explode(&mut self, x: f64, y: f64) {
        for p in self.particles.iter_mut() {
            let angle = self.rng.next_f64() * std::f64::consts::TAU;
            let force = 4.0 + self.rng.next_f64() * 8.0;
            p.x = x;
            p.y = y;
            p.vx = angle.cos() * force;
            p.vy = angle.sin() * force;
        }
    }
}

// Global thread-safe state container for execution frames
thread_local! {
    static ENGINE: RefCell<Option<Rc<RefCell<Engine>>>> = const { RefCell::new(None) };
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap().dyn_into::<HtmlCanvasElement>().unwrap();

    // Auto-scale target resolution to fit device display pixel sizes
    let width = window.inner_width().unwrap().as_f64().unwrap() as u32;
    let height = window.inner_height().unwrap().as_f64().unwrap() as u32;
    canvas.set_width(width);
    canvas.set_height(height);

    let engine = Rc::new(RefCell::new(Engine::new(canvas.clone())));
    ENGINE.with(|e| *e.borrow_mut() = Some(engine.clone()));

    // Capture mouse movement variables
    {
        let engine_clone = engine.clone();
        let closure = Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
            let mut engine = engine_clone.borrow_mut();
            engine.mouse_x = event.client_x() as f64;
            engine.mouse_y = event.client_y() as f64;
        });
        canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // Capture user clicks to trigger shockwaves
    {
        let engine_clone = engine.clone();
        let closure = Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
            let mut engine = engine_clone.borrow_mut();
            engine.explode(event.client_x() as f64, event.client_y() as f64);
        });
        canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // Explicitly annotate type of the closure cell to fix E0282
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::<dyn FnMut()>::new(move || {
        ENGINE.with(|e| {
            if let Some(ref engine) = *e.borrow() {
                engine.borrow_mut().update_and_render();
            }
        });

        // Request next frame recursively
        web_sys::window()
            .unwrap()
            .request_animation_frame(
                (f.borrow())
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .unchecked_ref(),
            )
            .unwrap();
    }));

    window.request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())?;
    Ok(())
}

