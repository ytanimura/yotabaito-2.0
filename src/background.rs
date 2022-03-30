use crate::*;
use std::sync::{Arc, Mutex};
use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext as GL, *};

mod shaders {
    include!(concat!(env!("OUT_DIR"), "/shaders.rs"));
}

pub enum Msg {
    Render(f64),
}

#[derive(Clone, Debug)]
struct Pipeline {
    program: WebGlProgram,
    position_location: u32,
    resolution_location: Option<WebGlUniformLocation>,
    time_location: Option<WebGlUniformLocation>,
    pixel_ratio: u32,
}

#[derive(Clone, Debug)]
struct PipelineBuilder(Arc<Mutex<Option<Pipeline>>>);

impl PipelineBuilder {
    fn new(gl: GL, shader: &'static str) -> Self {
        let arc = Arc::new(Mutex::new(None));
        let cloned_arc = Arc::clone(&arc);
        wasm_bindgen_futures::spawn_local(async move {
            let pipeline = create_pipeline(&gl, shader);
            *cloned_arc.lock().unwrap() = Some(pipeline);
        });
        Self(arc)
    }
    fn take(&self) -> Option<Pipeline> {
        self.0.lock().unwrap().take()
    }
}

#[derive(Debug)]
pub struct BackGround {
    gl: Option<GL>,
    canvas: NodeRef,
    pipeline: Option<Pipeline>,
    pipeline_builder: Option<PipelineBuilder>,
    render_loop: Option<gloo::render::AnimationFrame>,
    frame_count: u32,
}

fn get_shader() -> Option<&'static str> {
    let location = gloo::utils::window().location();
    let raw_query = location.search().expect_throw("failed to get query");
    let query = qstring::QString::from(raw_query.as_str());
    let hash = query.get("doc").unwrap_or("default");
    let shaders = shaders::get_texts();
    shaders
        .get(&hash)
        .copied()
        .or_else(|| shaders.get(&"default").copied())
}

impl Component for BackGround {
    type Message = Msg;
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Self {
            gl: None,
            canvas: Default::default(),
            pipeline: None,
            pipeline_builder: None,
            render_loop: None,
            frame_count: 0,
        }
    }

    fn view(&self, _: &Context<Self>) -> Html {
        html! { <canvas ref={ self.canvas.clone() } class="background"></canvas> }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        let canvas = self.canvas.cast::<HtmlCanvasElement>().unwrap();
        self.gl = { || canvas.get_context("webgl2").ok()??.dyn_into().ok() }();
        if let Some(gl) = &self.gl {
            init_gl(gl);
            if let Some(shader) = get_shader() {
                self.pipeline_builder = Some(PipelineBuilder::new(gl.clone(), shader));
            } else {
                gloo::utils::window()
                    .alert_with_message("failed to load shader")
                    .expect_throw("failed to show alert");
            }
        } else {
            let ctx: CanvasRenderingContext2d =
                { || canvas.get_context("2d").ok()??.dyn_into().ok() }()
                    .expect_throw("failed to init rendering context 2d");
            ctx.set_font("30px serif");
            let y = canvas.height() - 30;
            ctx.fill_text("Failed to init WebGL2...", 30.0, y as f64)
                .unwrap_or_else(|e| gloo::console::log!(format!("{e:?}")));
        }
        if first_render {
            self.set_render_loop(ctx);
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let Msg::Render(timestamp) = msg;
        let canvas = self.canvas.cast::<HtmlCanvasElement>().unwrap();
        match (&self.gl, &self.pipeline) {
            (Some(gl), Some(pipeline)) => {
                if correct_canvas_size(&canvas, pipeline.pixel_ratio) {
                    if let Some(gl) = &self.gl {
                        gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
                    }
                }
                // 30 FPS
                if self.frame_count % 2 == 0 {
                    let resolution = [canvas.width() as f32, canvas.height() as f32];
                    gl_rendering(gl, pipeline, resolution, timestamp as f32);
                }
                self.frame_count += 1;
            }
            (Some(_), None) => {
                self.pipeline = self
                    .pipeline_builder
                    .as_ref()
                    .and_then(PipelineBuilder::take);
            }
            _ => {}
        }
        self.set_render_loop(ctx);
        false
    }
}

impl BackGround {
    fn set_render_loop(&mut self, ctx: &Context<Self>) {
        let link = ctx.link().clone();
        let handle =
            gloo::render::request_animation_frame(move |time| link.send_message(Msg::Render(time)));
        self.render_loop = Some(handle);
    }
}

fn correct_canvas_size(canvas: &HtmlCanvasElement, pixel_ratio: u32) -> bool {
    let doc = gloo::utils::document_element();
    let resized = canvas.width() != doc.client_width() as u32 / pixel_ratio
        || canvas.height() != doc.client_height() as u32 / pixel_ratio;
    if resized {
        canvas.set_width(doc.client_width() as u32 / pixel_ratio);
        canvas.set_height(doc.client_height() as u32 / pixel_ratio);
    }
    resized
}

fn create_pipeline(gl: &GL, shader: &str) -> Pipeline {
    let program = prepare_program(gl, shader);
    Pipeline {
        position_location: gl.get_attrib_location(&program, "position") as u32,
        resolution_location: gl.get_uniform_location(&program, "iResolution"),
        time_location: gl.get_uniform_location(&program, "iTime"),
        program,
        pixel_ratio: 1,
    }
}

fn prepare_program(gl: &GL, shader: &str) -> WebGlProgram {
    const VERTEX_SHADER: &str = "#version 300 es
in vec3 position;void main(){gl_Position=vec4(position,1);}";
    const FRAMENT_SHADER_PREFIX: &str = "#version 300 es
precision highp float;uniform vec3 iResolution;uniform float iTime;out vec4 outColor;\
void mainImage(out vec4,in vec2);void main(){mainImage(outColor,gl_FragCoord.xy);}";

    // vertex shader
    let vert_shader = gl
        .create_shader(GL::VERTEX_SHADER)
        .expect_throw("failed to create shader pointer");
    gl.shader_source(&vert_shader, VERTEX_SHADER);
    gl.compile_shader(&vert_shader);

    // fragment shader
    let frag_shader = gl
        .create_shader(GL::FRAGMENT_SHADER)
        .expect_throw("failed to create shader pointer");
    let shader = String::from(FRAMENT_SHADER_PREFIX) + shader;
    gl.shader_source(&frag_shader, &shader);
    gl.compile_shader(&frag_shader);

    // create program
    let program = gl
        .create_program()
        .expect_throw("failed to create program pointer");
    gl.attach_shader(&program, &vert_shader);
    gl.attach_shader(&program, &frag_shader);
    gl.link_program(&program);

    program
}

fn init_gl(gl: &GL) {
    gl.clear_color(0.0, 0.0, 0.0, 0.0);

    #[rustfmt::skip]
    const POSITIONS: &[f32] = &[
        -1.0,  1.0, 0.0,
         1.0,  1.0, 0.0,
        -1.0, -1.0, 0.0,
         1.0, -1.0, 0.0,
    ];
    #[rustfmt::skip]
    const INDEX: &[u32] = &[
        0, 2, 1,
        1, 2, 3,
    ];

    // create vbo
    let vertex_buffer = gl.create_buffer();
    let vertex_buffer_js = js_sys::Float32Array::from(POSITIONS);
    gl.bind_buffer(GL::ARRAY_BUFFER, vertex_buffer.as_ref());
    gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &vertex_buffer_js, GL::STATIC_DRAW);

    // create ibo
    let index_buffer = gl.create_buffer();
    let index_buffer_js = js_sys::Uint32Array::from(INDEX);
    gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, index_buffer.as_ref());
    gl.buffer_data_with_array_buffer_view(
        GL::ELEMENT_ARRAY_BUFFER,
        &index_buffer_js,
        GL::STATIC_DRAW,
    );
}

fn gl_rendering(gl: &GL, pipeline: &Pipeline, resolution: [f32; 2], time: f32) {
    let Pipeline {
        program,
        position_location,
        resolution_location,
        time_location,
        ..
    } = pipeline;
    gl.use_program(Some(program));

    gl.enable_vertex_attrib_array(*position_location);
    gl.vertex_attrib_pointer_with_i32(*position_location, 3, GL::FLOAT, false, 0, 0);

    gl.uniform3f(
        resolution_location.as_ref(),
        resolution[0],
        resolution[1],
        0.0,
    );
    gl.uniform1f(time_location.as_ref(), time * 0.001);

    gl.clear(GL::COLOR_BUFFER_BIT);
    gl.draw_elements_with_i32(GL::TRIANGLES, 6, GL::UNSIGNED_INT, 0);
    gl.flush();
}
