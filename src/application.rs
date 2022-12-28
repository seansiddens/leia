use crate::{
    camera::Camera, hittable_list::HittableList, imgui_dock, mesh::Mesh, renderer::Renderer,
};
use glam::vec3;
use imgui::{FontConfig, FontGlyphRanges, FontSource};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use vulkano::{
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
        view::ImageView, ImageDimensions, ImageUsage, ImmutableImage, MipmapsCount, SwapchainImage,
    },
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::StandardMemoryAllocator,
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
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

// Initialize texture
// TODO: Dynamically size texture based on the viewport window size.
const ASPECT_RATIO: f32 = 4.0 / 3.0;
const TEX_WIDTH: usize = 800;
const TEX_HEIGHT: usize = (TEX_WIDTH as f32 / ASPECT_RATIO) as usize;

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

        let texture = ImmutableImage::from_iter(
            &memory_allocator,
            renderer.get_final_image().iter().cloned(),
            ImageDimensions::Dim2d {
                width: TEX_WIDTH as u32,
                height: TEX_HEIGHT as u32,
                array_layers: 1,
            },
            MipmapsCount::One,
            // TODO: Change format to support u32 for more range.
            Format::R8G8B8A8_SRGB,
            &mut builder,
        )
        .expect("Failed to create texture");

        // Build and execute the command buffer.
        let mut command_buffer = builder.build().unwrap();
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
        let final_texture_id = textures.insert((ImageView::new_default(texture).unwrap(), sampler));

        // Init the scene
        let mut scene = HittableList::new();
        let cornell = Mesh::from_gltf("assets/cornell.glb").unwrap();
        println!("cube tri count: {}", cornell.num_triangles());
        scene.add(cornell);
        let camera = Camera::new(
            vec3(0.0, 1.0, 3.0),
            vec3(0.0, 1.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            60.0,
            ASPECT_RATIO,
        );

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
        ui: &imgui::Ui,
        texture_id: Option<imgui::TextureId>,
        since_last_redraw: Duration,
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
                    |right| right.dock_window("Settings"),
                );

                // Create application windows as normal
                ui.window("Viewport")
                    .size([300.0, 110.0], imgui::Condition::FirstUseEver)
                    .build(|| {
                        if let Some(my_texture_id) = texture_id {
                            imgui::Image::new(my_texture_id, [TEX_WIDTH as f32, TEX_HEIGHT as f32])
                                .build(ui);
                        }
                    });
                ui.window("Settings")
                    .size([300.0, 110.0], imgui::Condition::FirstUseEver)
                    .build(|| {
                        ui.text(format!("Last render: {}ms", since_last_redraw.as_millis()));
                        ui.button("Render");
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
            camera,
            mut renderer,
            mut swapchain,
            mut images,
            mut imgui,
            mut platform,
            mut imgui_renderer,
            mut final_texture_id,
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

        event_loop.run(move |event, _, control_flow| {
            let window = surface.object().unwrap().downcast_ref::<Window>().unwrap();

            platform.handle_event(imgui.io_mut(), &window, &event);
            match event {
                Event::NewEvents(_) => {
                    // imgui.io_mut().update_delta_time(Instant::now());
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(_),
                    ..
                } => {
                    recreate_swapchain = true;
                }
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

                    // Render image.
                    renderer.render(&scene, &camera);

                    // Begin imgui frame
                    let ui = imgui.frame();
                    Application::render_ui(&ui, Some(final_texture_id), since_last_redraw);

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
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,
                event => {
                    platform.handle_event(imgui.io_mut(), window, &event);
                }
            }
        })
    }
}
