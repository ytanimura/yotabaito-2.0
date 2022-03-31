use crate::*;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc, Mutex,
};
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
    texture: Option<WebGlTexture>,
    position_location: u32,
    resolution_location: Option<WebGlUniformLocation>,
    time_location: Option<WebGlUniformLocation>,
    texture_location: Option<WebGlUniformLocation>,
    texture_resolution_location: Option<WebGlUniformLocation>,
    texture_resolution: Option<Arc<[AtomicU32; 2]>>,
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
        match (&self.gl, &mut self.pipeline) {
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

fn create_pipeline(gl: &GL, shader: &'static str) -> Pipeline {
    let (program, texture, texture_resolution) = prepare_program(gl, shader);
    Pipeline {
        position_location: gl.get_attrib_location(&program, "position") as u32,
        resolution_location: gl.get_uniform_location(&program, "iResolution"),
        time_location: gl.get_uniform_location(&program, "iTime"),
        texture_location: gl.get_uniform_location(&program, "iChannel0"),
        texture_resolution_location: gl.get_uniform_location(&program, "iChannelResolution"),
        texture_resolution,
        program,
        texture,
        pixel_ratio: 1,
    }
}

fn shader_trim(
    gl: GL,
    shader: &'static str,
) -> (
    &'static str,
    Option<WebGlTexture>,
    Option<Arc<[AtomicU32; 2]>>,
) {
    use std::io::{BufRead, BufReader};
    let first_line = BufReader::new(shader.as_bytes())
        .lines()
        .next()
        .unwrap()
        .unwrap();
    if first_line.len() >= 10 && &first_line[0..10] == "#iChannel0" {
        let path = std::path::Path::new(&first_line[12..first_line.len() - 1]);
        let path = String::from("./") + path.file_name().unwrap().to_str().unwrap();
        let texture = gl.create_texture();
        let res_texture = texture.clone();
        let image = HtmlImageElement::new().expect_throw("failed to create Image element");
        let cloned_image = image.clone();
        let image_resolution = Arc::new([AtomicU32::new(0), AtomicU32::new(0)]);
        let cloned_image_resolution = Arc::clone(&image_resolution);
        gloo::events::EventListener::once(&image, "load", move |_| {
            gl.bind_texture(GL::TEXTURE_2D, texture.as_ref());
            gl.pixel_storei(GL::UNPACK_FLIP_Y_WEBGL, 1);
            gl.tex_image_2d_with_u32_and_u32_and_html_image_element(
                GL::TEXTURE_2D,
                0,
                GL::RGBA as i32,
                GL::RGBA,
                GL::UNSIGNED_BYTE,
                &cloned_image,
            )
            .unwrap_or_else(|e| panic!("{e:?}"));
            gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32);
            gl.tex_parameteri(
                GL::TEXTURE_2D,
                GL::TEXTURE_MIN_FILTER,
                GL::LINEAR_MIPMAP_NEAREST as i32,
            );
            gl.generate_mipmap(GL::TEXTURE_2D);
            gl.bind_texture(GL::TEXTURE_2D, None);
            cloned_image_resolution[0].store(cloned_image.natural_width(), Ordering::SeqCst);
            cloned_image_resolution[1].store(cloned_image.natural_height(), Ordering::SeqCst);
        })
        .forget();
        image.set_src(&path);
        (
            &shader[first_line.len() + 1..],
            res_texture,
            Some(image_resolution),
        )
    } else {
        (shader, None, None)
    }
}

fn prepare_program(
    gl: &GL,
    shader: &'static str,
) -> (
    WebGlProgram,
    Option<WebGlTexture>,
    Option<Arc<[AtomicU32; 2]>>,
) {
    const VERTEX_SHADER: &str = "#version 300 es
in vec3 position;void main(){gl_Position=vec4(position,1);}";
    const FRAMENT_SHADER_PREFIX: &str = "#version 300 es
precision highp float;uniform vec3 iResolution;uniform float iTime;\
uniform sampler2D iChannel0;uniform vec3 iChannelResolution[1];\
out vec4 outColor;void mainImage(out vec4,in vec2);void main(){mainImage(outColor,gl_FragCoord.xy);}";

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
    let (shader, texture, texture_resolution) = shader_trim(gl.clone(), shader);
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

    (program, texture, texture_resolution)
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

    let vertex_buffer = gl.create_buffer();
    let vertex_buffer_js = js_sys::Float32Array::from(POSITIONS);
    gl.bind_buffer(GL::ARRAY_BUFFER, vertex_buffer.as_ref());
    gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &vertex_buffer_js, GL::STATIC_DRAW);

    let index_buffer = gl.create_buffer();
    let index_buffer_js = js_sys::Uint32Array::from(INDEX);
    gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, index_buffer.as_ref());
    gl.buffer_data_with_array_buffer_view(
        GL::ELEMENT_ARRAY_BUFFER,
        &index_buffer_js,
        GL::STATIC_DRAW,
    );
}

fn gl_rendering(gl: &GL, pipeline: &mut Pipeline, resolution: [f32; 2], time: f32) {
    let Pipeline {
        program,
        position_location,
        resolution_location,
        time_location,
        texture_location,
        texture,
        texture_resolution_location,
        texture_resolution,
        ..
    } = pipeline;
    gl.use_program(Some(program));

    gl.enable_vertex_attrib_array(*position_location);
    gl.vertex_attrib_pointer_with_i32(*position_location, 3, GL::FLOAT, false, 0, 0);

    gl.uniform3f(
        resolution_location.as_ref(),
        resolution[0],
        resolution[1],
        1.0,
    );
    gl.uniform1f(time_location.as_ref(), time * 0.001);
    gl.uniform1i(texture_location.as_ref(), 0);

    gl.bind_texture(GL::TEXTURE_2D, texture.as_ref());
    if let Some(texture_resolution_ref) = &texture_resolution {
        gl.uniform3f(
            texture_resolution_location.as_ref(),
            texture_resolution_ref[0].load(Ordering::SeqCst) as f32,
            texture_resolution_ref[1].load(Ordering::SeqCst) as f32,
            1.0,
        );
    }

    gl.clear(GL::COLOR_BUFFER_BIT);
    gl.draw_elements_with_i32(GL::TRIANGLES, 6, GL::UNSIGNED_INT, 0);
    gl.flush();
}
