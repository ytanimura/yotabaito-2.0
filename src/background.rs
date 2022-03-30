use crate::*;
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext as GL, *};

const PIXEL_RATIO: u32 = 1;

pub enum Msg {
    Render(f64),
}

struct Pipeline {
    program: WebGlProgram,
    position_location: u32,
    resolution_location: Option<WebGlUniformLocation>,
    time_location: Option<WebGlUniformLocation>,
}

pub struct BackGround {
    gl: Option<GL>,
    canvas: NodeRef,
    pipelines: HashMap<&'static str, Pipeline>,
    render_loop: Option<gloo::render::AnimationFrame>,
    frame_count: u32,
}

impl Component for BackGround {
    type Message = Msg;
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Self {
            gl: None,
            canvas: Default::default(),
            pipelines: Default::default(),
            render_loop: None,
            frame_count: 0,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Render(timestamp) => {
                let canvas = self.canvas.cast::<HtmlCanvasElement>().unwrap();
                if correct_canvas_size(&canvas) {
                    if let Some(gl) = &self.gl {
                        gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
                    }
                }
                if let Some(gl) = &self.gl {
                    if self.frame_count % 2 == 0 {
                        gl_rendering(
                            gl,
                            self.pipelines.get(&"default").unwrap(),
                            [canvas.width() as f32, canvas.height() as f32],
                            timestamp as f32,
                        );
                    }
                    self.frame_count += 1;
                    self.set_render_loop(ctx);
                }
                false
            }
        }
    }

    fn view(&self, _: &Context<Self>) -> Html {
        html! { <canvas ref={ self.canvas.clone() } class="background"></canvas> }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        let canvas = self.canvas.cast::<HtmlCanvasElement>().unwrap();
        correct_canvas_size(&canvas);

        let gl: GL = canvas
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();

        let program = prepare_program(&gl, include_str!("../shaders/default.frag"));
        let position_location = gl.get_attrib_location(&program, "position") as u32;
        let resolution_location = gl.get_uniform_location(&program, "iResolution");
        let time_location = gl.get_uniform_location(&program, "iTime");

        self.pipelines.insert(
            "default",
            Pipeline {
                program,
                position_location,
                resolution_location,
                time_location,
            },
        );
        gl.clear_color(0.0, 0.0, 0.0, 1.0);

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
        self.gl = Some(gl);
        if first_render {
            self.set_render_loop(ctx);
        }
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

fn correct_canvas_size(canvas: &HtmlCanvasElement) -> bool {
    let doc = gloo::utils::document_element();
    let resized = canvas.width() != doc.client_width() as u32 / PIXEL_RATIO
        || canvas.height() != doc.client_height() as u32 / PIXEL_RATIO;
    if resized {
        canvas.set_width(doc.client_width() as u32 / PIXEL_RATIO);
        canvas.set_height(doc.client_height() as u32 / PIXEL_RATIO);
    }
    resized
}

fn prepare_program(gl: &GL, shader: &str) -> WebGlProgram {
    const VERTEX_SHADER: &str = "#version 300 es
in vec3 position;void main(){gl_Position=vec4(position,1);}";
    const FRAMENT_SHADER_PREFIX: &str = "#version 300 es
precision highp float;uniform vec3 iResolution;uniform float iTime;out vec4 outColor;\
void mainImage(out vec4,in vec2);void main(){mainImage(outColor,gl_FragCoord.xy);}";

    // vertex shader
    let vert_shader = gl.create_shader(GL::VERTEX_SHADER).unwrap();
    gl.shader_source(&vert_shader, VERTEX_SHADER);
    gl.compile_shader(&vert_shader);

    // fragment shader
    let frag_shader = gl.create_shader(GL::FRAGMENT_SHADER).unwrap();
    let shader = String::from(FRAMENT_SHADER_PREFIX) + shader;
    gl.shader_source(&frag_shader, &shader);
    gl.compile_shader(&frag_shader);

    // create program
    let program = gl.create_program().unwrap();
    gl.attach_shader(&program, &vert_shader);
    gl.attach_shader(&program, &frag_shader);
    gl.link_program(&program);

    program
}

fn gl_rendering(gl: &GL, pipeline: &Pipeline, resolution: [f32; 2], time: f32) {
    let Pipeline {
        program,
        position_location,
        resolution_location,
        time_location,
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
