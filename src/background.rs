use crate::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext as GL, *};

const PIXEL_RATIO: u32 = 2;

pub enum Msg {
    Render(f64),
}

pub struct BackGround {
    gl: Option<GL>,
    canvas: NodeRef,
    render_loop: Option<gloo::render::AnimationFrame>,
}

impl Component for BackGround {
    type Message = Msg;
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Self {
            gl: None,
            canvas: Default::default(),
            render_loop: None,
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
                let resolution = [canvas.width() as f32, canvas.height() as f32];
                if let Some(gl) = &self.gl {
                    gl_rendering(
                        gl,
                        include_str!("../shaders/default.frag"),
                        resolution,
                        timestamp as f32,
                    );
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

fn gl_rendering(gl: &GL, shader: &str, resolution: [f32; 2], time: f32) {
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

    const VERTEX_SHADER: &str = "#version 300 es
in vec3 position;

void main(void){
    gl_Position = vec4(position, 1.0);
}
";

    // vertex shader
    let vert_shader = gl.create_shader(GL::VERTEX_SHADER).unwrap();
    gl.shader_source(&vert_shader, VERTEX_SHADER);
    gl.compile_shader(&vert_shader);

    // fragment shader
    let frag_shader = gl.create_shader(GL::FRAGMENT_SHADER).unwrap();
    gl.shader_source(&frag_shader, shader);
    gl.compile_shader(&frag_shader);

    // create program
    let program = gl.create_program().unwrap();
    gl.attach_shader(&program, &vert_shader);
    gl.attach_shader(&program, &frag_shader);
    gl.link_program(&program);

    gl.use_program(Some(&program));

    // create vbo
    let vertex_buffer = gl.create_buffer();
    let vertex_buffer_js = js_sys::Float32Array::from(POSITIONS);
    let position_location = gl.get_attrib_location(&program, "position") as u32;
    gl.bind_buffer(GL::ARRAY_BUFFER, vertex_buffer.as_ref());
    gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &vertex_buffer_js, GL::STATIC_DRAW);
    gl.enable_vertex_attrib_array(position_location);
    gl.vertex_attrib_pointer_with_i32(position_location, 3, GL::FLOAT, false, 0, 0);

    // create ibo
    let index_buffer = gl.create_buffer();
    let index_buffer_js = js_sys::Uint32Array::from(INDEX);
    gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, index_buffer.as_ref());
    gl.buffer_data_with_array_buffer_view(
        GL::ELEMENT_ARRAY_BUFFER,
        &index_buffer_js,
        GL::STATIC_DRAW,
    );

    let resolution_location = gl.get_uniform_location(&program, "iResolution");
    let time_location = gl.get_uniform_location(&program, "iTime");

    gl.clear_color(0.0, 0.0, 0.0, 1.0);

    gl.clear(GL::COLOR_BUFFER_BIT);

    gl.uniform3f(
        resolution_location.as_ref(),
        resolution[0],
        resolution[1],
        0.0,
    );
    gl.uniform1f(time_location.as_ref(), time * 0.001);

    gl.draw_elements_with_i32(GL::TRIANGLES, 6, GL::UNSIGNED_INT, 0);
    gl.flush();
}
