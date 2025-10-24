extern crate khronos_egl as egl;

use gl::types::{GLsizei, GLsizeiptr};
use smithay_client_toolkit::reexports::calloop::EventLoop;
use smithay_client_toolkit::reexports::calloop_wayland_source::WaylandSource;
use smithay_client_toolkit::registry::ProvidesRegistryState;
use smithay_client_toolkit::shell::wlr_layer::{
    Anchor, Layer, LayerShell, LayerShellHandler, LayerSurface, LayerSurfaceConfigure,
};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    output::{OutputHandler, OutputState},
    registry::RegistryState,
};
use smithay_client_toolkit::{
    delegate_compositor, delegate_layer, delegate_output, delegate_registry, registry_handlers,
};
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::Proxy;
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_output, wl_surface},
    Connection, QueueHandle,
};
use wayland_egl::WlEglSurface;

use core::{ffi, panic};
use egl::API as egl;
use std::ffi::CString;
use std::io::Write;
use std::process::{exit, ChildStdout};
use std::{env, fs, ptr};
use std::{
    io::{BufReader, Read},
    process::{Command, Stdio},
    time::Duration,
};

pub mod app_config;
use app_config::*;
pub mod cli_help;
use cli_help::*;
use std::collections::HashMap;

const VERTEX_SHADER_SRC: &str = include_str!("shaders/vertex_shader.glsl");

const FRAGMENT_SHADER_SRC: &str = include_str!("shaders/fragment_shader.glsl");

fn main() {
    let mut config_filename = "config.toml";
    let args: Vec<String> = env::args().collect();
    if args.len() == 3 {
        if args[1] != "--config" {
            print_help();
            exit(0);
        }
        config_filename = &args[2];
    } else if args.len() != 1 {
        print_help();
        exit(0);
    }
    let cava_output_config: HashMap<String, String> = HashMap::from([
        ("method".into(), "raw".into()),
        ("raw_target".into(), "/dev/stdout".into()),
        ("bit_format".into(), "16bit".into()),
    ]);
    let config_str = fs::read_to_string(config_filename).expect("Unable to read config file");
    let config: Config = match toml::from_str(&config_str) {
        Ok(config) => config,
        Err(error) => panic!("Error parsing config: {}", error.message()),
    };
    let cava_config = CavaConfig {
        general: CavaGeneralConfig {
            framerate: config.general.framerate,
            bars: config.bars.amount,
            autosens: config.general.autosens,
            sensitivity: config.general.sensitivity,
        },
        smoothing: CavaSmoothingConfig {
            monstercat: config.smoothing.monstercat,
            waves: config.smoothing.waves,
            noise_reduction: config.smoothing.noise_reduction,
        },
        output: cava_output_config,
    };
    let string_cava_config: String = toml::to_string(&cava_config).unwrap();
    let mut cmd = Command::new("cava");
    cmd.arg("-p").arg("/dev/stdin");
    let cava_process = cmd
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .expect("failed to spawn cava process");
    let mut cava_stdin = cava_process.stdin.unwrap();
    cava_stdin.write_all(string_cava_config.as_bytes()).unwrap();
    drop(cava_stdin);
    let cava_stdout = cava_process.stdout.unwrap();
    let cava_reader = BufReader::new(cava_stdout);
    let conn = Connection::connect_to_env().unwrap();
    let (globals, event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();
    let mut event_loop: EventLoop<AppState> =
        EventLoop::try_new().expect("Failed to initialize the event loop!");
    let loop_handle = event_loop.handle();
    WaylandSource::new(conn.clone(), event_queue)
        .insert(loop_handle)
        .unwrap();
    let frame_duration = Duration::from_secs(1) / config.general.framerate;
    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
    let surface = compositor.create_surface(&qh);
    let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell not available");
    let layer_surface = layer_shell.create_layer_surface(
        &qh,
        surface.clone(),
        Layer::Bottom,
        Some("wallpaper-cava"),
        None,
    );
    layer_surface.set_size(256, 256);
    layer_surface.set_anchor(Anchor::TOP);
    surface.commit();
    egl.bind_api(egl::OPENGL_API).unwrap();
    let egl_display = unsafe {
        egl.get_display(conn.display().id().as_ptr() as *mut std::ffi::c_void)
            .unwrap()
    };
    egl.initialize(egl_display).unwrap();
    const ATTRIBUTES: [i32; 9] = [
        egl::RED_SIZE,
        8,
        egl::GREEN_SIZE,
        8,
        egl::BLUE_SIZE,
        8,
        egl::ALPHA_SIZE,
        8,
        egl::NONE,
    ];

    let egl_config = egl
        .choose_first_config(egl_display, &ATTRIBUTES)
        .unwrap()
        .unwrap();
    const CONTEXT_ATTRIBUTES: [i32; 7] = [
        egl::CONTEXT_MAJOR_VERSION,
        4,
        egl::CONTEXT_MINOR_VERSION,
        6,
        egl::CONTEXT_OPENGL_PROFILE_MASK,
        egl::CONTEXT_OPENGL_CORE_PROFILE_BIT,
        egl::NONE,
    ];

    let egl_context = egl
        .create_context(egl_display, egl_config, None, &CONTEXT_ATTRIBUTES)
        .unwrap();

    let wl_egl_surface = WlEglSurface::new(surface.id(), 256, 256).unwrap();
    let egl_surface = unsafe {
        egl.create_window_surface(
            egl_display,
            egl_config,
            wl_egl_surface.ptr() as egl::NativeWindowType,
            None,
        )
        .unwrap()
    };
    egl.make_current(
        egl_display,
        Some(egl_surface),
        Some(egl_surface),
        Some(egl_context),
    )
    .unwrap();
    gl::load_with(|name| egl.get_proc_address(name).unwrap() as *const std::ffi::c_void);
    let version = unsafe {
        let data = gl::GetString(gl::VERSION) as *const i8;
        CString::from_raw(data as *mut _).into_string().unwrap()
    };

    println!("OpenGL version: {}", version);
    println!("EGL version: {}", egl.version());
    let vert_shader_source = CString::new(VERTEX_SHADER_SRC).unwrap();
    let vert_shader = unsafe { gl::CreateShader(gl::VERTEX_SHADER) };
    unsafe {
        gl::ShaderSource(
            vert_shader,
            1,
            &vert_shader_source.as_ptr(),
            std::ptr::null(),
        );
        gl::CompileShader(vert_shader);
    }
    let frag_shader_source = CString::new(FRAGMENT_SHADER_SRC).unwrap();
    let frag_shader = unsafe { gl::CreateShader(gl::FRAGMENT_SHADER) };
    unsafe {
        gl::ShaderSource(
            frag_shader,
            1,
            &frag_shader_source.as_ptr(),
            std::ptr::null(),
        );
        gl::CompileShader(frag_shader);
    }

    let shader_program = unsafe { gl::CreateProgram() };
    unsafe {
        gl::AttachShader(shader_program, vert_shader);
        gl::AttachShader(shader_program, frag_shader);
        gl::LinkProgram(shader_program);
        let mut status = gl::FALSE as gl::types::GLint;
        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut status);
        if status != 1 {
            let mut error_log_size: gl::types::GLint = 0;
            gl::GetProgramiv(shader_program, gl::INFO_LOG_LENGTH, &mut error_log_size);
            let mut error_log: Vec<u8> = Vec::with_capacity(error_log_size as usize);
            gl::GetProgramInfoLog(
                shader_program,
                error_log_size,
                &mut error_log_size,
                error_log.as_mut_ptr() as *mut _,
            );

            error_log.set_len(error_log_size as usize);
            let log = String::from_utf8(error_log).unwrap();
            panic!("{}", log);
        }
    }
    let mut vbo = 0;
    let mut vao = 0;
    let mut ebo = 0;
    let mut gradient_colors_ssbo = 0;
    let gradient_colors_rgba: Vec<[f32; 4]> = config
        .colors
        .iter()
        .map(|color| array_from_config_color((color.1).clone()))
        .collect();

    let gradient_colors_size = gradient_colors_rgba.len() as i32;
    let mut buffer_data: Vec<u8> = (gradient_colors_size).to_le_bytes().to_vec();
    buffer_data.extend([0, 0, 0, 0].repeat(3)); // Fix for vec4 alignment
    for color in gradient_colors_rgba.iter() {
        for color_value in color {
            buffer_data.extend_from_slice(&color_value.to_le_bytes());
        }
    }

    let mut indices: Vec<u16> = vec![0; config.bars.amount as usize * 6];
    for i in 0..config.bars.amount as usize {
        indices[i * 6] = i as u16 * 4;
        indices[i * 6 + 1] = i as u16 * 4 + 1;
        indices[i * 6 + 2] = i as u16 * 4 + 2;
        indices[i * 6 + 3] = i as u16 * 4 + 1;
        indices[i * 6 + 4] = i as u16 * 4 + 2;
        indices[i * 6 + 5] = i as u16 * 4 + 3;
    }

    let window_size_string = CString::new("WindowSize").unwrap();
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut ebo);
        gl::GenBuffers(1, &mut gradient_colors_ssbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (indices.len() * std::mem::size_of::<u16>()) as gl::types::GLsizeiptr,
            indices.as_ptr() as *const ffi::c_void,
            gl::STATIC_DRAW,
        );
        gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, gradient_colors_ssbo);
        gl::BufferData(
            gl::SHADER_STORAGE_BUFFER,
            buffer_data.len() as GLsizeiptr,
            buffer_data.as_ptr() as *const ffi::c_void,
            gl::STATIC_DRAW,
        );
        gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, gradient_colors_ssbo);
        gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
        gl::VertexAttribPointer(
            0,
            2,
            gl::FLOAT,
            gl::FALSE,
            (2 * std::mem::size_of::<f32>()) as gl::types::GLsizei,
            std::ptr::null(),
        );
        gl::EnableVertexAttribArray(0);
        gl::BindVertexArray(0);
    }

    let windows_size_location =
        unsafe { gl::GetUniformLocation(shader_program, window_size_string.as_ptr()) };

    let mut simple_window = AppState {
        registry_state: RegistryState::new(&globals),
        output_state: OutputState::new(&globals, &qh),
        width: 256,
        height: 256,
        layer_shell,
        layer_surface,
        surface,
        cava_reader,
        wl_egl_surface,
        egl_surface,
        egl_config,
        egl_context,
        egl_display,
        shader_program,
        vao,
        vbo,
        windows_size_location,
        bar_count: config.bars.amount,
        bar_gap: config.bars.gap,
        background_color: array_from_config_color(config.general.background_color),
        preferred_output_name: config.general.preferred_output,
        compositor,
    };
    event_loop
        .run(frame_duration, &mut simple_window, |_| {})
        .unwrap();
}

struct AppState {
    registry_state: RegistryState,
    output_state: OutputState,
    width: u32,
    height: u32,
    layer_shell: LayerShell,
    layer_surface: LayerSurface,
    surface: WlSurface,
    cava_reader: BufReader<ChildStdout>,
    wl_egl_surface: WlEglSurface,
    egl_surface: egl::Surface,
    egl_config: egl::Config,
    egl_context: egl::Context,
    egl_display: egl::Display,
    shader_program: u32,
    vao: u32,
    vbo: u32,
    windows_size_location: i32,
    bar_count: u32,
    bar_gap: f32,
    background_color: [f32; 4],
    preferred_output_name: Option<String>,
    compositor: CompositorState,
}

impl AppState {
    pub fn draw(&mut self, _conn: &Connection, qh: &QueueHandle<Self>) {
        let mut cava_buffer: Vec<u8> = vec![0; self.bar_count as usize * 2];
        let mut unpacked_data: Vec<f32> = vec![0.0; self.bar_count as usize];
        self.cava_reader.read_exact(&mut cava_buffer).unwrap();
        for (unpacked_data_index, i) in (0..cava_buffer.len()).step_by(2).enumerate() {
            let num = u16::from_le_bytes([cava_buffer[i], cava_buffer[i + 1]]);
            unpacked_data[unpacked_data_index] = (num as f32) / 65530.0;
        }
        let bar_width: f32 =
            2.0 / (self.bar_count as f32 + (self.bar_count as f32 - 1.0) * self.bar_gap);
        let bar_gap_width: f32 = bar_width * self.bar_gap;
        let mut vertices: Vec<f32> = vec![0.0; self.bar_count as usize * 8];
        let fwidth: f32 = self.width as f32;
        let fheight: f32 = self.height as f32;
        for i in 0..self.bar_count as usize {
            let bar_height: f32 = 2.0 * unpacked_data[i] - 1.0;
            vertices[i * 8] = bar_gap_width * i as f32 + bar_width * i as f32 - 1.0;
            vertices[i * 8 + 1] = bar_height;
            vertices[i * 8 + 2] = bar_gap_width * i as f32 + bar_width * (i + 1) as f32 - 1.0;
            vertices[i * 8 + 3] = bar_height;
            vertices[i * 8 + 4] = bar_gap_width * i as f32 + bar_width * i as f32 - 1.0;
            vertices[i * 8 + 5] = -1.0;
            vertices[i * 8 + 6] = bar_gap_width * i as f32 + bar_width * (i + 1) as f32 - 1.0;
            vertices[i * 8 + 7] = -1.0;
        }
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                vertices.as_ptr() as *const _,
                gl::DYNAMIC_DRAW,
            );
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::ClearColor(
                self.background_color[0],
                self.background_color[1],
                self.background_color[2],
                self.background_color[3],
            );
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::UseProgram(self.shader_program);
            gl::Uniform2f(self.windows_size_location, fwidth, fheight);
            gl::DrawElements(
                gl::TRIANGLES,
                (self.bar_count as usize * 3 * std::mem::size_of::<u16>()) as gl::types::GLsizei,
                // I don't know why * 3 works here, I thought that it is supposed to be * 6, but it
                // works, so I'll keep it like this for now.
                gl::UNSIGNED_SHORT,
                ptr::null(),
            );
            gl::BindVertexArray(0);
        }
        egl.swap_buffers(self.egl_display, self.egl_surface)
            .unwrap();
        self.surface.frame(qh, self.surface.clone());
    }
}

impl OutputHandler for AppState {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        let info = self.output_state.info(&output).unwrap();
        let mut need_configuration = false;
        if let Some(output_name) = info.name {
            if let Some(preffered_output_name) = self.preferred_output_name.clone() {
                if output_name == preffered_output_name {
                    need_configuration = true;
                }
            }
        }
        if self.preferred_output_name.is_none() {
            need_configuration = true;
        }
        if need_configuration {
            let old_surface = self.surface.clone();
            self.surface = self.compositor.create_surface(qh);
            self.layer_surface = self.layer_shell.create_layer_surface(
                qh,
                self.surface.clone(),
                Layer::Bottom,
                Some("wallpaper-cava"),
                Some(&output),
            );
            let logical_size = info.logical_size.unwrap();
            self.width = logical_size.0 as u32;
            self.height = logical_size.1 as u32;
            self.layer_surface.set_size(self.width, self.height);
            self.layer_surface.set_anchor(Anchor::TOP);
            self.surface.commit();
            old_surface.destroy();
        }
    }

    // For now update_output is same as new_output, because I'm not really sure what to do with it
    fn update_output(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        self.new_output(_conn, qh, output);
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

delegate_compositor!(AppState);

delegate_output!(AppState);
delegate_registry!(AppState);
delegate_layer!(AppState);

impl ProvidesRegistryState for AppState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![];
}

impl CompositorHandler for AppState {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        self.draw(conn, qh);
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }
}

impl LayerShellHandler for AppState {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {}

    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        let width = configure.new_size.0;
        let height = configure.new_size.1;
        println!(
            "LayerSurface configure event: width={}, height={}",
            width, height
        );
        self.width = width;
        self.height = height;
        egl.destroy_surface(self.egl_display, self.egl_surface)
            .unwrap();
        self.wl_egl_surface =
            WlEglSurface::new(self.surface.id(), self.width as i32, self.height as i32).unwrap();
        self.egl_surface = unsafe {
            egl.create_window_surface(
                self.egl_display,
                self.egl_config,
                self.wl_egl_surface.ptr() as egl::NativeWindowType,
                None,
            )
            .unwrap()
        };
        egl.make_current(
            self.egl_display,
            Some(self.egl_surface),
            Some(self.egl_surface),
            Some(self.egl_context),
        )
        .unwrap();
        unsafe {
            gl::Viewport(0, 0, self.width as GLsizei, self.height as GLsizei);
        }
        self.draw(_conn, qh);
        println!("configure finished");
    }
}
