use super::*;
use WebGl2RenderingContext as GL;

#[derive(Clone, Debug)]
pub struct Pipeline {
    pub program: WebGlProgram,
    pub texture: Option<TextureInfo>,
    pub position_location: u32,
    pub resolution_location: Option<WebGlUniformLocation>,
    pub time_location: Option<WebGlUniformLocation>,
    pub texture_location: Option<WebGlUniformLocation>,
    pub texture_resolution_location: Option<WebGlUniformLocation>,
    pub pixel_ratio: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct ShaderSource {
    pub source: &'static str,
    pub texture_url: Option<&'static str>,
}

#[derive(Clone, Debug)]
pub struct TextureInfo {
    texture: WebGlTexture,
    resolution: Arc<[AtomicU32; 2]>,
}

pub fn create_pipeline(gl: &GL, shader: ShaderSource) -> Pipeline {
    let (program, texture) = prepare_program(gl, shader);
    Pipeline {
        position_location: gl.get_attrib_location(&program, "position") as u32,
        resolution_location: gl.get_uniform_location(&program, "iResolution"),
        time_location: gl.get_uniform_location(&program, "iTime"),
        texture_location: gl.get_uniform_location(&program, "iChannel0"),
        texture_resolution_location: gl.get_uniform_location(&program, "iChannelResolution"),
        program,
        texture,
        pixel_ratio: 1,
    }
}

fn set_texture(gl: GL, texture_url: &'static str) -> Option<TextureInfo> {
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
    image.set_src(texture_url);
    res_texture.map(|texture| TextureInfo {
        texture,
        resolution: image_resolution,
    })
}

fn prepare_program(gl: &GL, shader: ShaderSource) -> (WebGlProgram, Option<TextureInfo>) {
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
    let texture = shader
        .texture_url
        .and_then(|texture_url| set_texture(gl.clone(), texture_url));
    let shader = String::from(FRAMENT_SHADER_PREFIX) + shader.source;
    gl.shader_source(&frag_shader, &shader);
    gl.compile_shader(&frag_shader);

    // create program
    let program = gl
        .create_program()
        .expect_throw("failed to create program pointer");
    gl.attach_shader(&program, &vert_shader);
    gl.attach_shader(&program, &frag_shader);
    gl.link_program(&program);

    (program, texture)
}

pub fn init_gl(gl: &GL) {
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

pub fn gl_rendering(gl: &GL, pipeline: &Pipeline, resolution: [f32; 2], time: f32) {
    let Pipeline {
        program,
        position_location,
        resolution_location,
        time_location,
        texture_location,
        texture,
        texture_resolution_location,
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
    gl.uniform1f(time_location.as_ref(), time);
    gl.uniform1i(texture_location.as_ref(), 0);

    if let Some(TextureInfo {
        texture,
        resolution,
    }) = texture
    {
        gl.bind_texture(GL::TEXTURE_2D, Some(texture));
        gl.uniform3f(
            texture_resolution_location.as_ref(),
            resolution[0].load(Ordering::SeqCst) as f32,
            resolution[1].load(Ordering::SeqCst) as f32,
            1.0,
        );
    }

    gl.clear(GL::COLOR_BUFFER_BIT);
    gl.draw_elements_with_i32(GL::TRIANGLES, 6, GL::UNSIGNED_INT, 0);
    gl.flush();
}
