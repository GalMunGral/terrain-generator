mod terrain;
mod utils;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

use nalgebra::{Matrix4, Rotation3, Unit, Vector3, Point3};
use std::{cell::RefCell, convert::TryInto, f32::consts::PI, rc::Rc};
use terrain::generate_terrain;
use utils::{
    set_panic_hook,
    web::{document, request_animation_frame, window},
    webgl::{load_shaders, setup_geometry, Geometry},
};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{
    Event, HtmlCanvasElement, HtmlImageElement, KeyboardEvent, WebGl2RenderingContext,
    WebGlProgram, WebGlVertexArrayObject,
};

#[derive(Debug)]
struct PressedKeys {
    w: bool,
    s: bool,
    a: bool,
    d: bool,
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

impl PressedKeys {
    fn new() -> PressedKeys {
        PressedKeys {
            w: false,
            s: false,
            a: false,
            d: false,
            up: false,
            down: false,
            left: false,
            right: false,
        }
    }
}

#[derive(Debug)]
struct Camera {
    eye: Point3<f32>,
    forward: Vector3<f32>,
    right: Vector3<f32>,
    up: Vector3<f32>,
}

impl Camera {
    fn new(x: f32, y: f32, z: f32) -> Camera {
        let mut camera = Camera {
            eye: Point3::origin(),
            forward: Vector3::zeros(),
            right: Vector3::zeros(),
            up: Vector3::zeros(),
        };
        camera.eye = Point3::new(x, y, z);
        camera.forward = (Point3::origin() - camera.eye).normalize();
        let up = Vector3::new(0.0, 0.0, 1.0);
        camera.right = camera.forward.cross(&up).normalize();
        camera.up = camera.right.cross(&camera.forward);
        camera
    }
}

const VERTEX_SHADER_URL: &str = "terrain_vertex.glsl";
const FRAGMENT_SHADER_URL: &str = "terrain_fragment.glsl";
const BOX_SIZE: f32 = 500.0;
const NEAR_PLANE: f32 = 10.0;
const FAR_PLANE: f32 = 4.0 * BOX_SIZE;
const FOCAL_LENGTH: f32 = 200.0;
const FLIGHT_SPEED: f32 = 5.0;
const TURNING_SPEED: f32 = 0.02;

#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    set_panic_hook();

    let camera = Rc::new(RefCell::new(Camera::new(0.0, -BOX_SIZE, 0.8 * BOX_SIZE)));
    let pressed = Rc::new(RefCell::new(PressedKeys::new()));
    let proj_mat = Rc::new(RefCell::new(Matrix4::<f32>::zeros()));

    let gl = document()
        .query_selector("canvas")?
        .ok_or("element canvas not found")?
        .dyn_into::<HtmlCanvasElement>()?
        .get_context("webgl2")?
        .ok_or("can't get webgl2 context")?
        .dyn_into::<WebGl2RenderingContext>()?;

    gl.enable(WebGl2RenderingContext::DEPTH_TEST);

    let program = load_shaders(&gl, VERTEX_SHADER_URL, FRAGMENT_SHADER_URL).await?;
    let geometry = generate_terrain();
    let vao = setup_geometry(&gl, &program, &geometry)?;

    setup_event_listeners(&gl, &pressed, &proj_mat)?;
    load_texture(gl.clone(), program.clone())?;

    let raf_cb = Rc::new(RefCell::new(None as Option<Closure<dyn FnMut(f64)>>));
    let raf_cb_ = Rc::clone(&raf_cb);
    *raf_cb.borrow_mut() = Some(Closure::new(move |ms| {
        draw(
            &gl, &program, &vao, &geometry, &pressed, &camera, &proj_mat, ms,
        );
        request_animation_frame(raf_cb_.borrow().as_ref().expect("RAF callback is defined"))
            .expect("RAF failed");
    }));
    request_animation_frame(raf_cb.borrow().as_ref().expect("RAF callback is defined"))
        .expect("RAF failed");

    Ok(())
}

fn load_texture(gl: WebGl2RenderingContext, program: WebGlProgram) -> Result<(), JsValue> {
    let slot = 0;
    let texture = gl.create_texture().ok_or("failed to create texture")?;
    gl.active_texture(WebGl2RenderingContext::TEXTURE0 + slot);
    gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

    gl.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_WRAP_S,
        WebGl2RenderingContext::CLAMP_TO_EDGE.try_into().unwrap(),
    );
    gl.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_WRAP_T,
        WebGl2RenderingContext::CLAMP_TO_EDGE.try_into().unwrap(),
    );
    gl.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_MIN_FILTER,
        WebGl2RenderingContext::NEAREST.try_into().unwrap(),
    );
    gl.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_MAG_FILTER,
        WebGl2RenderingContext::LINEAR.try_into().unwrap(),
    );

    let image_el = document()
        .create_element("img")
        .expect("unable to create image element")
        .dyn_into::<HtmlImageElement>()?;

    image_el.set_cross_origin(Some("anonymous"));
    image_el.set_src("texture.jpeg");

    let onload = Closure::<dyn FnMut(Event)>::new(move |e: Event| {
        let image_el = &e.target().unwrap().dyn_into::<HtmlImageElement>().unwrap();
        gl.tex_image_2d_with_u32_and_u32_and_html_image_element(
            WebGl2RenderingContext::TEXTURE_2D,
            0,
            WebGl2RenderingContext::RGBA.try_into().unwrap(),
            WebGl2RenderingContext::RGBA,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            image_el,
        )
        .expect("failed to load texture");

        let loc = gl
            .get_uniform_location(&program, "terrainTexture")
            .expect("uniform not found");
        gl.use_program(Some(&program));
        gl.uniform1i(Some(&loc), slot.try_into().unwrap());
    });
    image_el.add_event_listener_with_callback("load", onload.as_ref().unchecked_ref())?;
    onload.forget();
    Ok(())
}

fn reset_aspect_ratio(
    gl: &WebGl2RenderingContext,
    proj_mat: &Rc<RefCell<Matrix4<f32>>>,
) -> Result<(), JsValue> {
    let canvas = document()
        .query_selector("canvas")?
        .ok_or("element canvas not found")?
        .dyn_into::<HtmlCanvasElement>()?;

    canvas.set_width(
        canvas
            .client_width()
            .try_into()
            .expect("canvas width shouldn't be negative"),
    );
    canvas.set_height(
        canvas
            .client_height()
            .try_into()
            .expect("canvas height shouldn't be negative"),
    );

    *proj_mat.borrow_mut() = Matrix4::new_perspective(
        canvas.width() as f32 / canvas.height() as f32,
        PI / 2.0,
        NEAR_PLANE,
        FAR_PLANE,
    );

    gl.viewport(
        0,
        0,
        canvas
            .width()
            .try_into()
            .expect("canvas width is non-negative"),
        canvas
            .height()
            .try_into()
            .expect("canvas height is non-negative"),
    );
    Ok(())
}

fn draw(
    gl: &WebGl2RenderingContext,
    program: &WebGlProgram,
    vao: &WebGlVertexArrayObject,
    geometry: &Geometry,
    pressed: &Rc<RefCell<PressedKeys>>,
    camera: &Rc<RefCell<Camera>>,
    proj_mat: &Rc<RefCell<Matrix4<f32>>>,
    _ms: f64,
) {
    gl.clear_color(0.5, 0.5, 0.5, 0.5);
    gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT);
    gl.use_program(Some(program));
    gl.bind_vertex_array(Some(vao));

    let pressed = &mut *pressed.borrow_mut();
    let camera = &mut *camera.borrow_mut();

    if pressed.w {
        camera.eye += FLIGHT_SPEED * camera.forward;
    } else if pressed.s {
        camera.eye -= FLIGHT_SPEED * camera.forward;
    } else if pressed.d {
        camera.eye += FLIGHT_SPEED * camera.right;
    } else if pressed.a {
        camera.eye -= FLIGHT_SPEED * camera.right;
    }

    if pressed.up {
        let axis = Unit::new_normalize(camera.right.clone());
        let rot = Rotation3::from_axis_angle(&axis, TURNING_SPEED);
        camera.up = rot * camera.up;
        camera.forward = rot * camera.forward;
    } else if pressed.down {
        let axis = Unit::new_normalize(camera.right.clone());
        let rot = Rotation3::from_axis_angle(&axis, -TURNING_SPEED);
        camera.up = rot * camera.up;
        camera.forward = rot * camera.forward;
    } else if pressed.left {
        let axis = Unit::new_normalize(camera.up.clone());
        let rot = Rotation3::from_axis_angle(&axis, TURNING_SPEED);
        camera.right = rot * camera.right;
        camera.forward = rot * camera.forward;
    } else if pressed.right {
        let axis = Unit::new_normalize(camera.up.clone());
        let rot = Rotation3::from_axis_angle(&axis, -TURNING_SPEED);
        camera.right = rot * camera.right;
        camera.forward = rot * camera.forward;
    }

    gl.uniform3fv_with_f32_array(
        gl.get_uniform_location(&program, "eye").as_ref(),
        (camera.eye - Point3::origin()).as_ref(),
    );

    let model_mat = Matrix4::<f32>::identity();
    gl.uniform_matrix4fv_with_f32_array(
        gl.get_uniform_location(&program, "m").as_ref(),
        false,
        model_mat.as_slice(),
    );

    let target = camera.eye + FOCAL_LENGTH * camera.forward;
    let view_mat = Matrix4::look_at_rh(&camera.eye, &target, &camera.up);
    gl.uniform_matrix4fv_with_f32_array(
        gl.get_uniform_location(&program, "v").as_ref(),
        false,
        view_mat.as_slice(),
    );

    gl.uniform_matrix4fv_with_f32_array(
        gl.get_uniform_location(&program, "p").as_ref(),
        false,
        proj_mat.borrow().as_slice(),
    );

    gl.uniform1i(gl.get_uniform_location(&program, "fogEnabled").as_ref(), 1);

    gl.draw_elements_with_i32(
        WebGl2RenderingContext::TRIANGLES,
        geometry
            .triangles
            .len()
            .try_into()
            .expect("number of indices should fit into i32"),
        WebGl2RenderingContext::UNSIGNED_INT,
        0,
    );
}

fn setup_event_listeners(
    gl: &WebGl2RenderingContext,
    pressed: &Rc<RefCell<PressedKeys>>,
    proj_mat: &Rc<RefCell<Matrix4<f32>>>,
) -> Result<(), JsValue> {
    let pressed_1 = Rc::clone(pressed);
    let onkeydown = Closure::<dyn FnMut(KeyboardEvent)>::new(move |e: KeyboardEvent| {
        let mut pressed = pressed_1.borrow_mut();
        match e.key_code() {
            87 => pressed.w = true,
            83 => pressed.s = true,
            65 => pressed.a = true,
            68 => pressed.d = true,
            38 => pressed.up = true,
            40 => pressed.down = true,
            37 => pressed.left = true,
            39 => pressed.right = true,
            _ => (),
        }
    });
    window().add_event_listener_with_callback("keydown", onkeydown.as_ref().unchecked_ref())?;
    onkeydown.forget();

    let pressed_2 = Rc::clone(pressed);
    let onkeyup = Closure::<dyn FnMut(KeyboardEvent)>::new(move |e: KeyboardEvent| {
        let mut pressed = pressed_2.borrow_mut();
        match e.key_code() {
            87 => pressed.w = false,
            83 => pressed.s = false,
            65 => pressed.a = false,
            68 => pressed.d = false,
            38 => pressed.up = false,
            40 => pressed.down = false,
            37 => pressed.left = false,
            39 => pressed.right = false,
            _ => (),
        }
    });
    window().add_event_listener_with_callback("keyup", onkeyup.as_ref().unchecked_ref())?;
    onkeyup.forget();

    let gl_1 = gl.clone();
    let proj_mat_1 = Rc::clone(proj_mat);
    reset_aspect_ratio(&gl, &proj_mat_1)?;

    let onresize = Closure::<dyn FnMut()>::new(move || {
        reset_aspect_ratio(&gl_1, &proj_mat_1).expect("failed to reset aspect ratio");
    });
    window().add_event_listener_with_callback("resize", onresize.as_ref().unchecked_ref())?;
    onresize.forget();

    Ok(())
}
