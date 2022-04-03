use crate::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[derive(Debug, Default)]
pub struct NavBar {
    div: NodeRef,
    cursored: Arc<AtomicBool>,
    render_loop: Option<gloo::render::AnimationFrame>,
    previous: f64,
}

#[derive(Clone, Debug)]
pub enum Msg {
    Render(f64),
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {}

impl Component for NavBar {
    type Message = Msg;
    type Properties = Props;

    fn create(_: &Context<Self>) -> Self {
        Default::default()
    }

    fn view(&self, _: &Context<Self>) -> Html {
        let over_cursord = Arc::clone(&self.cursored);
        let out_cursord = Arc::clone(&self.cursored);
        html! {
            <div class="navbar" ref={ self.div.clone() }
                onmouseover={ move |_| over_cursord.store(true, Ordering::SeqCst) }
                onmouseout={ move |_| out_cursord.store(false, Ordering::SeqCst) }
            >
            <a href="./"><img src="./logo.png" class="logo"/></a>
            <a href="./index.html?doc=profile"><div class="text-icon">{ "Profile" }</div></a>
            <a href="./index.html?doc=mathematics"><div class="text-icon">{ "Math" }</div></a>
            <a href="./index.html?doc=development"><div class="text-icon">{ "Dev" }</div></a>
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            self.send_render_message(ctx);
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Msg) -> bool {
        let Msg::Render(timestamp) = msg;
        if timestamp < 1000.0 {
            self.previous = timestamp;
            self.send_render_message(ctx);
            return false;
        }
        let div = self.div.clone().cast::<HtmlDivElement>().unwrap();
        let style = div.style();
        let mut opacity = style
            .get_property_value("opacity")
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(1.0);
        let deltime = (timestamp - self.previous) * 0.001;
        opacity = match self.cursored.load(Ordering::SeqCst) {
            true => opacity + deltime * 2.0,
            false => {
                opacity + (smoothstep(0.0, 1.0, opacity - deltime) - smoothstep(0.0, 1.0, opacity))
            }
        };
        opacity = f64::clamp((opacity - 0.001) / 0.999, 0.0, 1.0);
        let _ = style.set_property("opacity", &format!("{opacity}"));

        self.previous = timestamp;
        self.send_render_message(ctx);
        true
    }
}

impl NavBar {
    fn send_render_message(&mut self, ctx: &Context<Self>) {
        let link = ctx.link().clone();
        self.render_loop = Some(gloo::render::request_animation_frame(move |time| {
            link.send_message(Msg::Render(time))
        }));
    }
}

#[inline]
fn smoothstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    let t = f64::clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
