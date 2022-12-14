use crate::{
    camera::*, hittable_list::HittableList, imgui_dock, input::*, mesh::Mesh, renderer::Renderer,
    Color,
};
use bytemuck::{Pod, Zeroable};
use glam::{vec3a, Quat, Vec3A};
use imgui::{FontConfig, FontGlyphRanges, FontSource};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use rand::{Rng, SeedableRng};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{
        allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo},
        AutoCommandBufferBuilder, ClearColorImageInfo, CommandBufferUsage,
        PrimaryCommandBufferAbstract,
    },
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, Queue,
        QueueCreateInfo,
    },
    format::Format,
    image::{
        view::ImageView, AttachmentImage, ImageDimensions, ImageUsage, ImmutableImage,
        MipmapsCount, SwapchainImage,
    },
    impl_vertex,
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::StandardMemoryAllocator,
    pipeline::{
        graphics::{
            input_assembly::InputAssemblyState, vertex_input::BuffersDefinition,
            viewport::ViewportState,
        },
        GraphicsPipeline,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, Subpass},
    sampler::{Sampler, SamplerCreateInfo},
    swapchain::{
        self, AcquireError, ColorSpace, PresentMode, Surface, Swapchain, SwapchainCreateInfo,
        SwapchainCreationError,
    },
    sync::{self, FlushError, GpuFuture},
    VulkanLibrary,
};
use vulkano_win::VkSurfaceBuild;
use winit::{
    dpi::PhysicalSize,
    event::{Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{CursorGrabMode, Window, WindowBuilder},
};

// Initialize texture
// TODO: Dynamically size texture based on the viewport window size.
const ASPECT_RATIO: f32 = 4.0 / 3.0;
const TEX_WIDTH: usize = 800;
const TEX_HEIGHT: usize = (TEX_WIDTH as f32 / ASPECT_RATIO) as usize;

/// Returns a scene of 'n' random triangles.
fn random_triangles(n: i32) -> Vec<crate::Triangle> {
    // let mut world = HittableList::new();
    let mut list = Vec::new();
    let mut rng = rand_xoshiro::Xoroshiro128PlusPlus::from_entropy();
    let max_scale = 100.0;
    let max_range = 500.0;

    for _ in 0..n {
        let r0 = vec3a(
            rng.gen_range(0.0..1.0),
            rng.gen_range(0.0..1.0),
            rng.gen_range(0.0..1.0),
        );
        let r1 = vec3a(
            rng.gen_range(0.0..max_scale),
            rng.gen_range(0.0..max_scale),
            rng.gen_range(0.0..max_scale),
        );
        let r2 = vec3a(
            rng.gen_range(0.0..max_scale),
            rng.gen_range(0.0..max_scale),
            rng.gen_range(0.0..max_scale),
        );
        let v0 = r0 * max_range - vec3a(max_range * 0.5, max_range * 0.5, max_range * 0.5);
        let v1 = v0 + r1;
        let v2 = v0 + r2;
        let albedo = Color::new(
            rng.gen_range(0.0..1.0),
            rng.gen_range(0.0..1.0),
            rng.gen_range(0.0..1.0),
        );
        list.push(crate::Triangle::new(v0, v1, v2, albedo, Color::ZERO));
    }

    list
}

pub struct Application {
    pub event_loop: EventLoop<()>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub surface: Arc<Surface>,
    pub swapchain: Arc<Swapchain>,
    pub images: Vec<Arc<SwapchainImage>>,
    pub imgui: imgui::Context,
    pub platform: WinitPlatform,
    pub imgui_renderer: imgui_vulkano_renderer::Renderer,
    pub font_size: f32,
    final_texture_id: imgui::TextureId,
    renderer: Renderer,
    scene: HittableList,
    camera: Camera,

    pub memory_allocator: Arc<StandardMemoryAllocator>,
    pub command_buffer_allocator: StandardCommandBufferAllocator,
}

impl Application {
    /// Initializes the application.
    /// TODO: Refactor using the builder design patter.
    pub fn init(title: &str, width: u32, height: u32) -> Self {
        // Load the Vulkan library.
        let library = VulkanLibrary::new().unwrap();

        // Create the Vulkan instance w/ the required extensions.
        let required_extensions = vulkano_win::required_extensions(&library);
        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                enabled_extensions: required_extensions,
                ..Default::default()
            },
        )
        .unwrap();

        // Create window, surface, and event loop.
        let title = match title.rfind('/') {
            Some(idx) => title.split_at(idx + 1).1,
            None => title,
        };

        let event_loop = EventLoop::new();
        let surface = WindowBuilder::new()
            .with_inner_size(PhysicalSize { width, height })
            .with_title(title.to_owned())
            .build_vk_surface(&event_loop, instance.clone())
            .expect("Failed to create a window!");
        let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();

        // Select a physical device and queue family.
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };
        let (physical, queue_family) = instance
            .enumerate_physical_devices()
            .unwrap()
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                let queue_family = p
                    .queue_family_properties()
                    .iter()
                    .enumerate()
                    .find(|(i, q)| {
                        q.queue_flags.graphics
                            && p.surface_support(*i as u32, &surface).unwrap_or(false)
                    })
                    .map(|(i, _q)| i);

                queue_family.map(|i| (p, i))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                _ => 4,
            })
            .unwrap();

        // Create the logical device.
        let (device, mut queues) = Device::new(
            Arc::clone(&physical),
            DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index: queue_family as u32,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .unwrap();
        // Get the graphics queue to use.
        let queue = queues.next().unwrap();

        // Create swapchain.
        let format;
        let (swapchain, images) = {
            let caps = physical
                .surface_capabilities(&surface, Default::default())
                .unwrap();

            format = Some(
                physical
                    .surface_formats(&surface, Default::default())
                    .unwrap()[0]
                    .0,
            );

            let image_usage = ImageUsage {
                transfer_dst: true,
                color_attachment: true,
                ..ImageUsage::empty()
            };

            Swapchain::new(
                device.clone(),
                surface.clone(),
                SwapchainCreateInfo {
                    min_image_count: caps.min_image_count,
                    image_format: format,
                    image_extent: window.inner_size().into(),
                    image_usage,
                    composite_alpha: caps.supported_composite_alpha.iter().next().unwrap(),
                    image_color_space: ColorSpace::SrgbNonLinear,
                    present_mode: PresentMode::Fifo,
                    ..Default::default()
                },
            )
            .unwrap()
        };

        // Create memory allocators.
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(Arc::clone(&device)));

        // We now create a buffer that will store the shape of our triangle.
        // We use #[repr(C)] here to force rustc to not do anything funky with our data, although for this
        // particular example, it doesn't actually change the in-memory representation.
        #[repr(C)]
        #[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
        struct Vertex {
            position: [f32; 2],
        }
        impl_vertex!(Vertex, position);

        let vertices = [
            Vertex {
                position: [-0.5, -0.25],
            },
            Vertex {
                position: [0.0, 0.5],
            },
            Vertex {
                position: [0.25, -0.1],
            },
        ];
        let vertex_buffer = CpuAccessibleBuffer::from_iter(
            &memory_allocator,
            BufferUsage {
                vertex_buffer: true,
                ..Default::default()
            },
            false,
            vertices,
        )
        .unwrap();

        // The next step is to create the shaders.
        //
        // The raw shader creation API provided by the vulkano library is unsafe for various
        // reasons, so The `shader!` macro provides a way to generate a Rust module from GLSL
        // source - in the example below, the source is provided as a string input directly to
        // the shader, but a path to a source file can be provided as well. Note that the user
        // must specify the type of shader (e.g., "vertex," "fragment, etc.") using the `ty`
        // option of the macro.
        //
        // The module generated by the `shader!` macro includes a `load` function which loads
        // the shader using an input logical device. The module also includes type definitions
        // for layout structures defined in the shader source, for example, uniforms and push
        // constants.
        //
        // A more detailed overview of what the `shader!` macro generates can be found in the
        // `vulkano-shaders` crate docs. You can view them at https://docs.rs/vulkano-shaders/
        mod vs {
            vulkano_shaders::shader! {
                ty: "vertex",
                src: "
				#version 450

				layout(location = 0) in vec2 position;

				void main() {
					gl_Position = vec4(position, 0.0, 1.0);
				}
			"
            }
        }

        mod fs {
            vulkano_shaders::shader! {
                ty: "fragment",
                src: "
				#version 450

				layout(location = 0) out vec4 f_color;

				void main() {
					f_color = vec4(1.0, 0.0, 0.0, 1.0);
				}
			"
            }
        }

        let vs = vs::load(device.clone()).unwrap();
        let fs = fs::load(device.clone()).unwrap();

        // At this point, OpenGL initialization would be finished. However in Vulkan it is not. OpenGL
        // implicitly does a lot of computation whenever you draw. In Vulkan, you have to do all this
        // manually.

        // The next step is to create a *render pass*, which is an object that describes where the
        // output of the graphics pipeline will go. It describes the layout of the images
        // where the colors, depth and/or stencil information will be written.
        let render_pass = vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                // `color` is a custom name we give to the first and only attachment.
                color: {
                    // `load: Clear` means that we ask the GPU to clear the content of this
                    // attachment at the start of the drawing.
                    load: Clear,
                    // `store: Store` means that we ask the GPU to store the output of the draw
                    // in the actual image. We could also ask it to discard the result.
                    store: Store,
                    // `format: <ty>` indicates the type of the format of the image. This has to
                    // be one of the types of the `vulkano::format` module (or alternatively one
                    // of your structs that implements the `FormatDesc` trait). Here we use the
                    // same format as the swapchain.
                    format: swapchain.image_format(),
                    // `samples: 1` means that we ask the GPU to use one sample to determine the value
                    // of each pixel in the color attachment. We could use a larger value (multisampling)
                    // for antialiasing. An example of this can be found in msaa-renderpass.rs.
                    samples: 1,
                }
            },
            pass: {
                // We use the attachment named `color` as the one and only color attachment.
                color: [color],
                // No depth-stencil attachment is indicated with empty brackets.
                depth_stencil: {}
            }
        )
        .unwrap();

        // Before we draw we have to create what is called a pipeline. This is similar to an OpenGL
        // program, but much more specific.
        let pipeline = GraphicsPipeline::start()
            // We have to indicate which subpass of which render pass this pipeline is going to be used
            // in. The pipeline will only be usable from this particular subpass.
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            // We need to indicate the layout of the vertices.
            .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
            // The content of the vertex buffer describes a list of triangles.
            .input_assembly_state(InputAssemblyState::new())
            // A Vulkan shader can in theory contain multiple entry points, so we have to specify
            // which one.
            .vertex_shader(vs.entry_point("main").unwrap(), ())
            // Use a resizable viewport set to draw over the entire window
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            // See `vertex_shader`.
            .fragment_shader(fs.entry_point("main").unwrap(), ())
            // Now that our builder is filled, we call `build()` to obtain an actual pipeline.
            .build(device.clone())
            .unwrap();

        let command_buffer_allocator = StandardCommandBufferAllocator::new(
            Arc::clone(&device),
            StandardCommandBufferAllocatorCreateInfo::default(),
        );

        // Init Imgui stuff.
        let mut imgui = imgui::Context::create();
        imgui.io_mut().config_flags |= imgui::ConfigFlags::DOCKING_ENABLE;

        let mut platform = WinitPlatform::init(&mut imgui);
        platform.attach_window(imgui.io_mut(), window, HiDpiMode::Rounded);

        let hidpi_factor = platform.hidpi_factor();
        let font_size = (13.0 * hidpi_factor) as f32;
        imgui.fonts().add_font(&[
            FontSource::DefaultFontData {
                config: Some(FontConfig {
                    size_pixels: font_size,
                    ..FontConfig::default()
                }),
            },
            FontSource::TtfData {
                data: include_bytes!("../assets/mplus-1p-regular.ttf"),
                size_pixels: font_size,
                config: Some(FontConfig {
                    rasterizer_multiply: 1.75,
                    glyph_ranges: FontGlyphRanges::japanese(),
                    ..FontConfig::default()
                }),
            },
        ]);
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        let mut imgui_renderer = imgui_vulkano_renderer::Renderer::init(
            &mut imgui,
            device.clone(),
            queue.clone(),
            format.unwrap(),
            None,
            None,
        )
        .expect("Failed to initialize renderer");

        // Initialize the renderer.
        let renderer = Renderer::new(TEX_WIDTH, TEX_HEIGHT);

        // Create the initial texture.
        let mut builder = AutoCommandBufferBuilder::primary(
            &command_buffer_allocator,
            queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        // let texture = ImmutableImage::from_iter(
        //     &memory_allocator,
        //     renderer.get_final_image().iter().cloned(),
        //     ImageDimensions::Dim2d {
        //         width: TEX_WIDTH as u32,
        //         height: TEX_HEIGHT as u32,
        //         array_layers: 1,
        //     },
        //     MipmapsCount::One,
        //     // TODO: Change format to support u32 for more range.
        //     Format::R8G8B8A8_SRGB,
        //     &mut builder,
        // )
        // .expect("Failed to create texture");

        let texture = AttachmentImage::input_attachment(
            &memory_allocator,
            [TEX_WIDTH as u32, TEX_HEIGHT as u32],
            Format::R8G8B8A8_SRGB,
        ).unwrap();

        // Build and execute the command buffer.
        let command_buffer = builder.build().unwrap();
        command_buffer
            .execute(Arc::clone(&queue))
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();

        let sampler = Sampler::new(
            device.clone(),
            SamplerCreateInfo::simple_repeat_linear_no_mipmap(),
        )
        .unwrap();

        let textures = imgui_renderer.textures_mut();
        // Add the ImageView and Sampler for the image to the texture id map.
        // Texture must be of type (Arc<dyn ImageViewAbstract + Send + Sync>, Arc<Sampler>)
        let texture_image_view = ImageView::new_default(texture).unwrap();

        // Create framebuffer for the texture we want to render to.
        let framebuffer = Framebuffer::new(
            render_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![texture_image_view.clone()],
                ..Default::default()
            },
        );

        let final_texture_id = textures.insert((texture_image_view, sampler));

        // Init the scene
        let mut scene = HittableList::new();
        // let mut random_triangles = random_triangles(100);
        // for tri in random_triangles {
        //     scene.add(tri);
        // }
        // scene.add(Mesh::from_triangles(random_triangles));
        // let mut plane = Mesh::from_gltf("assets/plane.glb").unwrap();
        // plane.transformation(Vec3A::ONE * 5.0, Quat::from_rotation_z(0.1), Vec3A::ZERO);
        // scene.add(plane);
        // let triangle = crate::Triangle::new(
        //     vec3a(2.5, 0.0, 0.0),
        //     vec3a(1.5, 2.1213203435596457, 0.0),
        //     vec3a(-1.5, 1.5, 0.0),
        // );
        // scene.add(triangle);
        // let mut icosphere = Mesh::from_gltf("assets/icosphere.glb").unwrap();
        // icosphere.translation(vec3a(0.0, 1.0, 0.0));
        // scene.add(icosphere);
        // let mut cube = Mesh::from_gltf("assets/cube.glb").unwrap();
        // cube.transformation(
        //     Vec3A::ONE * 2.0,
        //     Quat::from_rotation_y(1.0),
        //     vec3a(-1.0, 1.0, -1.0),
        // );
        // scene.add(cube);
        let cornell = Mesh::from_gltf("assets/cornell_light.glb").unwrap();
        println!("cube tri count: {}", cornell.num_triangles());
        scene.add(cornell);
        // let mut bunny = Mesh::from_gltf("assets/bunny.glb").unwrap();
        // bunny.transformation(
        //     vec3a(0.25, 0.25, 0.25),
        //     Quat::IDENTITY,
        //     vec3a(0.3, 0.8, 0.5),
        // );
        // scene.add(bunny);
        let camera = Camera::new(45.0, 0.1, 100.0, TEX_WIDTH as u32, TEX_HEIGHT as u32, false);

        Application {
            event_loop,
            device,
            queue,
            surface,
            swapchain,
            images,
            imgui,
            platform,
            imgui_renderer,
            font_size,
            final_texture_id,
            renderer,
            scene,
            camera,

            memory_allocator,
            command_buffer_allocator,
        }
    }

    fn render_ui(
        renderer: &mut Renderer,
        ui: &imgui::Ui,
        texture_id: Option<imgui::TextureId>,
        since_last_redraw: Duration,
        camera: &mut Camera,
    ) {
        let flags =
        // No borders etc for top-level window
        imgui::WindowFlags::NO_DECORATION | imgui::WindowFlags::NO_MOVE
        // Show menu bar
        | imgui::WindowFlags::MENU_BAR
        // Don't raise window on focus (as it'll clobber floating windows)
        | imgui::WindowFlags::NO_BRING_TO_FRONT_ON_FOCUS | imgui::WindowFlags::NO_NAV_FOCUS
        // Don't want the dock area's parent to be dockable!
        | imgui::WindowFlags::NO_DOCKING
        ;

        // Remove padding/rounding on main container window
        let mw_style_tweaks = {
            let padding = ui.push_style_var(imgui::StyleVar::WindowPadding([0.0, 0.0]));
            let rounding = ui.push_style_var(imgui::StyleVar::WindowRounding(0.0));
            (padding, rounding)
        };

        // Create top-level window which occuplies full screen
        ui.window("Main Window")
            .flags(flags)
            .position([0.0, 0.0], imgui::Condition::Always)
            .size(ui.io().display_size, imgui::Condition::Always)
            .build(|| {
                // Pop main window style.
                mw_style_tweaks.0.end();
                mw_style_tweaks.1.end();

                // Create top-level docking area, needs to be made early (before docked windows)
                let ui_d = imgui_dock::UiDocking {};
                let space = ui_d.dockspace("MainDockArea");

                // Set up splits, docking windows. This can be done conditionally,
                // or calling it every time is also mostly fine
                space.split(
                    imgui::Direction::Left,
                    0.7,
                    |left| {
                        left.dock_window("Viewport");
                    },
                    |right| {
                        right.split(
                            imgui::Direction::Up,
                            0.5,
                            |up| up.dock_window("Scene"),
                            |down| down.dock_window("Settings"),
                        )
                    },
                );

                // Create application windows as normal
                ui.window("Viewport")
                    .size([300.0, 110.0], imgui::Condition::FirstUseEver)
                    .build(|| {
                        if let Some(my_texture_id) = texture_id {
                            imgui::Image::new(my_texture_id, [TEX_WIDTH as f32, TEX_HEIGHT as f32])
                                // Flip the final image vertically.
                                .uv0([0.0, 1.0])
                                .uv1([1.0, 0.0])
                                .build(ui);
                        }
                    });
                ui.window("Scene")
                    .size([300.0, 110.0], imgui::Condition::FirstUseEver)
                    .build(|| {
                        ui.text("Camera Transform");
                        let mut cam_pos = camera.get_position().to_array();
                        if imgui::Drag::new("Position")
                            .speed(0.2)
                            .build_array(ui, &mut cam_pos)
                        {
                            camera.set_position(glam::Vec3A::from_array(cam_pos));

                            // Since camera was moved we need reset the accumulation data.
                            renderer.reset_accumulation_data();
                        }
                    });
                ui.window("Settings")
                    .size([300.0, 110.0], imgui::Condition::FirstUseEver)
                    .build(|| {
                        ui.text(format!("Last render: {}ms", since_last_redraw.as_millis()));
                        ui.text(format!("Frame index: {}", renderer.get_frame_index()));
                    });
            });
    }

    pub fn main_loop(self) {
        let Self {
            event_loop,
            device,
            queue,
            surface,
            memory_allocator,
            scene,
            mut camera,
            mut renderer,
            mut swapchain,
            mut images,
            mut imgui,
            mut platform,
            mut imgui_renderer,
            final_texture_id,
            ..
        } = self;

        // Flag for whether the swapchain should be recreated.
        let mut recreate_swapchain = false;

        // In the loop below we are going to submit commands to the GPU. Submitting a command produces
        // an object that implements the `GpuFuture` trait, which holds the resources for as long as
        // they are in use by the GPU.
        //
        // Destroying the `GpuFuture` blocks until the GPU is finished executing it. In order to avoid
        // that, we store the submission of the previous frame here.
        let mut previous_frame_submission = Some(sync::now(device.clone()).boxed());

        let mut last_redraw = Instant::now();

        let mut input_state = InputState::new();

        // Master application level RNG for seeding per-thread RNGs.
        let mut app_rng = rand_xoshiro::Xoshiro256PlusPlus::from_entropy();

        event_loop.run(move |event, _, control_flow| {
            let mut window = surface.object().unwrap().downcast_ref::<Window>().unwrap();

            platform.handle_event(imgui.io_mut(), &window, &event);
            match event {
                Event::NewEvents(_) => {
                    // imgui.io_mut().update_delta_time(Instant::now());
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(_) => recreate_swapchain = true,
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => input_state.update(event),
                },
                Event::MainEventsCleared => {
                    platform
                        .prepare_frame(imgui.io_mut(), &window)
                        .expect("Failed to prepare frame");
                    window.request_redraw();
                }
                Event::RedrawEventsCleared => {
                    // Do not draw frame when screen dimensions are zero.
                    // On Windows, this can occur from minimizing the application.
                    let dimensions = window.inner_size();
                    if dimensions.width == 0 || dimensions.height == 0 {
                        return;
                    }

                    // Calculate time since last redraw.
                    let t = Instant::now();
                    let since_last_redraw = t.duration_since(last_redraw);
                    last_redraw = t;

                    // It is important to call this function from time to time, otherwise resources will keep
                    // accumulating and you will eventually reach an out of memory error.
                    // Calling this function polls various fences in order to determine what the GPU has
                    // already processed, and frees the resources that are no longer needed.
                    previous_frame_submission
                        .as_mut()
                        .unwrap()
                        .cleanup_finished();

                    // Whenever the window resizes we need to recreate everything dependent on the window size.
                    if recreate_swapchain {
                        // Use the new dimensions of the window.
                        let (new_swapchain, new_images) =
                            match swapchain.recreate(SwapchainCreateInfo {
                                image_extent: dimensions.into(),
                                ..swapchain.create_info()
                            }) {
                                Ok(r) => r,
                                // This error tends to happen when the user is manually resizing the window.
                                // Simply restarting the loop is the easiest way to fix this issue.
                                Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => {
                                    return
                                }
                                Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                            };

                        images = new_images;
                        swapchain = new_swapchain;
                        recreate_swapchain = false;
                    }

                    // Update.
                    if input_state.is_mouse_button_down(MouseButton::Right) {
                        // Hide and lock cursor if right mouse button is held down.
                        window.set_cursor_visible(false);
                        window
                            .set_cursor_grab(CursorGrabMode::Locked)
                            // Attempt to lock. If fails, attempt to confine.
                            .or_else(|_e| {
                                window
                                    .set_cursor_grab(CursorGrabMode::Confined)
                                    .or_else(|_| window.set_cursor_grab(CursorGrabMode::None))
                            })
                            .unwrap();
                    } else {
                        window.set_cursor_visible(true);
                        window.set_cursor_grab(CursorGrabMode::None).unwrap();
                    }

                    if camera.update(&input_state, since_last_redraw.as_secs_f32()) {
                        // Camera moved, so we need to reset accumulation data.
                        renderer.reset_accumulation_data();
                    }

                    // Render image.
                    // renderer.render(&scene, &camera, &mut app_rng);

                    // Begin imgui frame
                    let ui = imgui.frame();
                    Application::render_ui(
                        &mut renderer,
                        &ui,
                        Some(final_texture_id),
                        since_last_redraw,
                        &mut camera,
                    );

                    // Before we can draw on the output, we have to *acquire* an image from the swapchain. If
                    // no image is available (which happens if you submit draw commands too quickly), then the
                    // function will block.
                    // This operation returns the index of the image that we are allowed to draw upon.
                    //
                    // This function can block if no image is available. The parameter is an optional timeout
                    // after which the function call will return an error.
                    let (image_num, suboptimal, acquire_future) =
                        match vulkano::swapchain::acquire_next_image(swapchain.clone(), None) {
                            Ok(r) => r,
                            Err(AcquireError::OutOfDate) => {
                                recreate_swapchain = true;
                                return;
                            }
                            Err(e) => panic!("Failed to acquire next image: {:?}", e),
                        };

                    // acquire_next_image can be successful, but suboptimal. This means that the swapchain image
                    // will still work, but it may not display correctly. With some drivers this can be when
                    // the window resizes, but it may not cause the swapchain to become out of date.
                    if suboptimal {
                        recreate_swapchain = true;
                    }

                    // Mouse cursor is changed/hidden if requested by imgui-rs.
                    // Must be called before imgui.render().
                    platform.prepare_render(&ui, window);

                    // Imgui draw data.
                    let draw_data = imgui.render();

                    // In order to draw, we have to build a *command buffer*. The command buffer object holds
                    // the list of commands that are going to be executed.
                    //
                    // Building a command buffer is an expensive operation (usually a few hundred
                    // microseconds), but it is known to be a hot path in the driver and is expected to be
                    // optimized.
                    //
                    // Note that we have to pass a queue family when we create the command buffer. The command
                    // buffer will only be executable on that given queue family.
                    let mut cmd_buf_builder = AutoCommandBufferBuilder::primary(
                        &self.command_buffer_allocator,
                        queue.queue_family_index(),
                        vulkano::command_buffer::CommandBufferUsage::OneTimeSubmit,
                    )
                    .expect("Failed to create command buffer");

                    cmd_buf_builder
                        .clear_color_image(ClearColorImageInfo::image(
                            images[image_num as usize].clone(),
                        ))
                        .expect("Failed to create image clear command");

                    // Fetch final image data from renderer.
                    let image_data = renderer.get_final_image();
                    // Get a mutable ref to the texture from the texture id, and change the image view to be built from a new image with the new data.
                    let textures = imgui_renderer.textures_mut();
                    if let Some(new_texture) = textures.get_mut(final_texture_id) {
                        new_texture.0 = ImageView::new_default(
                            ImmutableImage::from_iter(
                                &memory_allocator,
                                image_data.iter().cloned(),
                                ImageDimensions::Dim2d {
                                    width: TEX_WIDTH as u32,
                                    height: TEX_HEIGHT as u32,
                                    array_layers: 1,
                                },
                                MipmapsCount::One,
                                Format::R8G8B8A8_SRGB,
                                &mut cmd_buf_builder,
                            )
                            .expect("Failed to create texture"),
                        )
                        .unwrap();
                    }

                    // Append draw commands to the command buffer to draw the UI.
                    imgui_renderer
                        .draw_commands(
                            &mut cmd_buf_builder,
                            ImageView::new_default(images[image_num as usize].clone()).unwrap(),
                            draw_data,
                        )
                        .expect("Rendering failed");

                    // Finish building the command buffer.
                    let cmd_buf = cmd_buf_builder
                        .build()
                        .expect("Failed to build command buffer");

                    let future = previous_frame_submission
                        .take()
                        .unwrap()
                        .join(acquire_future)
                        .then_execute(queue.clone(), cmd_buf)
                        .unwrap()
                        .then_signal_fence()
                        // The color output is now expected to contain the final application image. But in order to show it on
                        // the screen, we have to *present* the image by calling `present`.
                        //
                        // This function does not actually present the image immediately. Instead it submits a
                        // present command at the end of the queue. This means that it will only be presented once
                        // the GPU has finished executing the command buffer.
                        .then_swapchain_present(
                            queue.clone(),
                            swapchain::SwapchainPresentInfo::swapchain_image_index(
                                Arc::clone(&swapchain),
                                image_num,
                            ),
                        );

                    match future.flush() {
                        Ok(_) => {
                            previous_frame_submission = Some(future.boxed());
                        }
                        Err(FlushError::OutOfDate) => {
                            recreate_swapchain = true;
                            previous_frame_submission = Some(sync::now(device.clone()).boxed());
                        }
                        Err(e) => {
                            println!("Failed to flush future: {:?}", e);
                            previous_frame_submission = Some(sync::now(device.clone()).boxed());
                        }
                    }
                }
                event => {
                    platform.handle_event(imgui.io_mut(), window, &event);
                }
            }
        })
    }
}
