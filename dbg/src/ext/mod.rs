// Yoinked and to be much twisted from 

use ash::{
    ext::debug_utils,
    khr::{surface, swapchain},
    vk, Device, Entry, Instance,
};
use imgui::*;
use imgui_rs_vulkan_renderer::*;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::{
    error::Error,
    ffi::{CStr, CString},
    marker::PhantomData,
    os::raw::c_void,
    time::Instant,
};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::{Window, WindowBuilder},
};

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 768;

pub trait App {
    fn destroy(&mut self, context: &VulkanContext);
}

impl App for () {
    fn destroy(&mut self, _: &VulkanContext) {}
}

pub struct System<A: App + 'static> {
    phantom_data: PhantomData<A>,
    window: Window,
    event_loop: EventLoop<()>,
    pub vulkan_context: VulkanContext,
    command_buffer: vk::CommandBuffer,
    swapchain: Swapchain,
    image_available_semaphore: vk::Semaphore,
    render_finished_semaphore: vk::Semaphore,
    fence: vk::Fence,

    pub imgui: Context,
    #[allow(dead_code)]
    pub font_size: f32,
    platform: WinitPlatform,
    pub renderer: Renderer,
}

impl<A: App> System<A> {
    pub fn new(title: &str) -> Result<Self, Box<dyn Error>> {
        // Setup window
        let (window, event_loop) = create_window(title)?;

        let vulkan_context = VulkanContext::new(&window, title)?;

        let command_buffer = {
            let allocate_info = vk::CommandBufferAllocateInfo::default()
                .command_pool(vulkan_context.command_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1);

            unsafe {
                vulkan_context
                    .device
                    .allocate_command_buffers(&allocate_info)?[0]
            }
        };

        let swapchain = Swapchain::new(&vulkan_context)?;

        // Semaphore use for presentation
        let image_available_semaphore = {
            let semaphore_info = vk::SemaphoreCreateInfo::default();
            unsafe {
                vulkan_context
                    .device
                    .create_semaphore(&semaphore_info, None)?
            }
        };
        let render_finished_semaphore = {
            let semaphore_info = vk::SemaphoreCreateInfo::default();
            unsafe {
                vulkan_context
                    .device
                    .create_semaphore(&semaphore_info, None)?
            }
        };
        let fence = {
            let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
            unsafe { vulkan_context.device.create_fence(&fence_info, None)? }
        };

        let mut imgui = Context::create();
        imgui.set_ini_filename(None);

        let mut platform = WinitPlatform::init(&mut imgui);

        let hidpi_factor = platform.hidpi_factor();
	let font_size = (13.0 * hidpi_factor) as f32;
	
	let renderer = Renderer::with_default_allocator(
	    &vulkan_context.instance,
	    vulkan_context.physical_device,
	    vulkan_context.device.clone(),
	    vulkan_context.graphics_queue,
	    vulkan_context.command_pool,
	    swapchain.render_pass,
	    &mut imgui,
	    Some(Options {
		in_flight_frames: 1,
		..Default::default()
	    }),
	).unwrap();
	
        Ok(Self {
            phantom_data: PhantomData,
            window,
            event_loop,
            vulkan_context,
            command_buffer,
            swapchain,
            image_available_semaphore,
            render_finished_semaphore,
            fence,
            imgui,
            font_size,
            platform,
            renderer,
        })
    }

    #[allow(dead_code)]
    pub fn update_fonts_texture(&mut self) -> Result<(), Box<dyn Error>> {
        self.renderer.update_fonts_texture(
            self.vulkan_context.graphics_queue,
            self.vulkan_context.command_pool,
            &mut self.imgui,
        )?;

        Ok(())
    }

    pub fn run<B>(self, mut app: A, mut ui_builder: B) -> Result<(), Box<dyn Error>>
    where
        B: FnMut(&mut bool, &mut Ui, &mut A) + 'static,
    {
        let Self {
            window,
            event_loop,
            vulkan_context,
            command_buffer,
            mut swapchain,
            image_available_semaphore,
            render_finished_semaphore,
            fence,
            mut imgui,
            mut platform,
            mut renderer,
            ..
        } = self;

        let mut last_frame = Instant::now();
        let mut run = true;
        let mut dirty_swapchain = false;

        // Main loop
        event_loop.run(move |event, elwt| {
            // Make sure Renderer is moved before VulkanContext and therefore dropped before
            let renderer = &mut renderer;

	    //this fixes my issue with double input in input_texts lmfao
	    //there's probably a proper solution to that but this works for now
	    if let Event::WindowEvent { event: WindowEvent::KeyboardInput {event: KeyEvent { state: ElementState::Released, .. }, ..}, .. } = event {} else {
		platform.handle_event(imgui.io_mut(), &window, &event);
	    }

            match event {
                // New frame
                Event::NewEvents(_) => {
                    let now = Instant::now();
                    imgui.io_mut().update_delta_time(now - last_frame);
                    last_frame = now;
                }
                // End of event processing
                Event::AboutToWait => {
                    // If swapchain must be recreated wait for windows to not be minimized anymore
                    if dirty_swapchain {
                        let PhysicalSize { width, height } = window.inner_size();
                        if width > 0 && height > 0 {
                            swapchain
                                .recreate(&vulkan_context)
                                .expect("Failed to recreate swapchain");
                            renderer
                                .set_render_pass(swapchain.render_pass)
                                .expect("Failed to rebuild renderer pipeline");
                            dirty_swapchain = false;
                        } else {
                            return;
                        }
                    }

                    // Generate UI
                    platform
                        .prepare_frame(imgui.io_mut(), &window)
                        .expect("Failed to prepare frame");
                    let ui = imgui.frame();
                    ui_builder(&mut run, ui, &mut app);
                    platform.prepare_render(ui, &window);
                    let draw_data = imgui.render();

                    if !run {
                        elwt.exit();
                        return;
                    }

                    unsafe {
                        vulkan_context
                            .device
                            .wait_for_fences(&[fence], true, u64::MAX)
                            .expect("Failed to wait ")
                    };

                    // Drawing the frame
                    let next_image_result = unsafe {
                        swapchain.loader.acquire_next_image(
                            swapchain.khr,
                            u64::MAX,
                            image_available_semaphore,
                            vk::Fence::null(),
                        )
                    };
                    let image_index = match next_image_result {
                        Ok((image_index, _)) => image_index,
                        Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                            dirty_swapchain = true;
                            return;
                        }
                        Err(error) => panic!("Error while acquiring next image. Cause: {}", error),
                    };

                    unsafe {
                        vulkan_context
                            .device
                            .reset_fences(&[fence])
                            .expect("Failed to reset fences")
                    };

                    let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
                    let wait_semaphores = [image_available_semaphore];
                    let signal_semaphores = [render_finished_semaphore];

                    // Re-record commands to draw geometry
                    record_command_buffers(
                        &vulkan_context.device,
                        vulkan_context.command_pool,
                        command_buffer,
                        swapchain.framebuffers[image_index as usize],
                        swapchain.render_pass,
                        swapchain.extent,
                        renderer,
                        draw_data,
                    )
                    .expect("Failed to record command buffer");

                    let command_buffers = [command_buffer];
                    let submit_info = [vk::SubmitInfo::default()
                        .wait_semaphores(&wait_semaphores)
                        .wait_dst_stage_mask(&wait_stages)
                        .command_buffers(&command_buffers)
                        .signal_semaphores(&signal_semaphores)];
                    unsafe {
                        vulkan_context
                            .device
                            .queue_submit(vulkan_context.graphics_queue, &submit_info, fence)
                            .expect("Failed to submit work to gpu.")
                    };

                    let swapchains = [swapchain.khr];
                    let images_indices = [image_index];
                    let present_info = vk::PresentInfoKHR::default()
                        .wait_semaphores(&signal_semaphores)
                        .swapchains(&swapchains)
                        .image_indices(&images_indices);

                    let present_result = unsafe {
                        swapchain
                            .loader
                            .queue_present(vulkan_context.present_queue, &present_info)
                    };
                    match present_result {
                        Ok(is_suboptimal) if is_suboptimal => {
                            dirty_swapchain = true;
                        }
                        Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                            dirty_swapchain = true;
                        }
                        Err(error) => panic!("Failed to present queue. Cause: {}", error),
                        _ => {}
                    }
                }
                // Resizing
                Event::WindowEvent {
                    event: WindowEvent::Resized(new_size),
                    ..
                } => {
                    dirty_swapchain = true;
                }
                // Exit
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    elwt.exit();
                }
                // Cleanup
                Event::LoopExiting => {
                    unsafe {
                        vulkan_context
                            .device
                            .device_wait_idle()
                            .expect("Failed to wait for graphics device to idle.");

                        app.destroy(&vulkan_context);

                        vulkan_context.device.destroy_fence(fence, None);
                        vulkan_context
                            .device
                            .destroy_semaphore(image_available_semaphore, None);
                        vulkan_context
                            .device
                            .destroy_semaphore(render_finished_semaphore, None);

                        swapchain.destroy(&vulkan_context);

                        vulkan_context
                            .device
                            .free_command_buffers(vulkan_context.command_pool, &[command_buffer]);
                    }
                }
                _ => (),
            }
        })?;

        Ok(())
    }
}

pub struct VulkanContext {
    _entry: Entry,
    pub instance: Instance,
    debug_utils: debug_utils::Instance,
    debug_utils_messenger: vk::DebugUtilsMessengerEXT,
    surface: surface::Instance,
    surface_khr: vk::SurfaceKHR,
    pub physical_device: vk::PhysicalDevice,
    graphics_q_index: u32,
    present_q_index: u32,
    pub device: Device,
    pub graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    pub command_pool: vk::CommandPool,
}

impl VulkanContext {
    pub fn new(window: &Window, name: &str) -> Result<Self, Box<dyn Error>> {
        // Vulkan instance
        let entry = unsafe {Entry::load().unwrap()};
        let (instance, debug_utils, debug_utils_messenger) =
            create_vulkan_instance(&entry, window, name)?;

        // Vulkan surface
        let surface = surface::Instance::new(&entry, &instance);
        let surface_khr = unsafe {
            ash_window::create_surface(
                &entry,
                &instance,
                window.display_handle().unwrap().into(),
                window.window_handle().unwrap().into(),
                None,
            )?
        };

        // Vulkan physical device and queue families indices (graphics and present)
        let (physical_device, graphics_q_index, present_q_index) =
            create_vulkan_physical_device_and_get_graphics_and_present_qs_indices(
                &instance,
                &surface,
                surface_khr,
            )?;

        // Vulkan logical device and queues
        let (device, graphics_queue, present_queue) =
            create_vulkan_device_and_graphics_and_present_qs(
                &instance,
                physical_device,
                graphics_q_index,
                present_q_index,
            )?;

        // Command pool & buffer
        let command_pool = {
            let command_pool_info = vk::CommandPoolCreateInfo::default()
                .queue_family_index(graphics_q_index)
                .flags(vk::CommandPoolCreateFlags::empty());
            unsafe { device.create_command_pool(&command_pool_info, None)? }
        };

        Ok(Self {
            _entry: entry,
            instance,
            debug_utils,
            debug_utils_messenger,
            surface,
            surface_khr,
            physical_device,
            graphics_q_index,
            present_q_index,
            device,
            graphics_queue,
            present_queue,
            command_pool,
        })
    }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_command_pool(self.command_pool, None);
            self.device.destroy_device(None);
            self.surface.destroy_surface(self.surface_khr, None);
            self.debug_utils
                .destroy_debug_utils_messenger(self.debug_utils_messenger, None);
            self.instance.destroy_instance(None);
        }
    }
}

struct Swapchain {
    loader: swapchain::Device,
    extent: vk::Extent2D,
    khr: vk::SwapchainKHR,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
    render_pass: vk::RenderPass,
    framebuffers: Vec<vk::Framebuffer>,
}

impl Swapchain {
    fn new(vulkan_context: &VulkanContext) -> Result<Self, Box<dyn Error>> {
        // Swapchain
        let (loader, khr, extent, format, images, image_views) =
            create_vulkan_swapchain(vulkan_context)?;

        // Renderpass
        let render_pass = create_vulkan_render_pass(&vulkan_context.device, format)?;

        // Framebuffers
        let framebuffers =
            create_vulkan_framebuffers(&vulkan_context.device, render_pass, extent, &image_views)?;

        Ok(Self {
            loader,
            extent,
            khr,
            images,
            image_views,
            render_pass,
            framebuffers,
        })
    }

    fn recreate(&mut self, vulkan_context: &VulkanContext) -> Result<(), Box<dyn Error>> {
        unsafe { vulkan_context.device.device_wait_idle()? };

        self.destroy(vulkan_context);

        // Swapchain
        let (loader, khr, extent, format, images, image_views) =
            create_vulkan_swapchain(vulkan_context)?;

        // Renderpass
        let render_pass = create_vulkan_render_pass(&vulkan_context.device, format)?;

        // Framebuffers
        let framebuffers =
            create_vulkan_framebuffers(&vulkan_context.device, render_pass, extent, &image_views)?;

        self.loader = loader;
        self.extent = extent;
        self.khr = khr;
        self.images = images;
        self.image_views = image_views;
        self.render_pass = render_pass;
        self.framebuffers = framebuffers;

        Ok(())
    }

    fn destroy(&mut self, vulkan_context: &VulkanContext) {
        unsafe {
            self.framebuffers
                .iter()
                .for_each(|fb| vulkan_context.device.destroy_framebuffer(*fb, None));
            self.framebuffers.clear();
            vulkan_context
                .device
                .destroy_render_pass(self.render_pass, None);
            self.image_views
                .iter()
                .for_each(|v| vulkan_context.device.destroy_image_view(*v, None));
            self.image_views.clear();
            self.loader.destroy_swapchain(self.khr, None);
        }
    }
}

fn create_window(title: &str) -> Result<(Window, EventLoop<()>), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let window = WindowBuilder::new()
        .with_title(title)
        .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT))
        .with_resizable(false)
        .build(&event_loop)?;

    Ok((window, event_loop))
}

fn create_vulkan_instance(
    entry: &Entry,
    window: &Window,
    title: &str,
) -> Result<(Instance, debug_utils::Instance, vk::DebugUtilsMessengerEXT), Box<dyn Error>> {
    // Vulkan instance
    let app_name = CString::new(title)?;
    let engine_name = CString::new("No Engine")?;
    let app_info = vk::ApplicationInfo::default()
        .application_name(app_name.as_c_str())
        .application_version(vk::make_api_version(0, 0, 1, 0))
        .engine_name(engine_name.as_c_str())
        .engine_version(vk::make_api_version(0, 0, 1, 0))
        .api_version(vk::make_api_version(0, 1, 0, 0));

    let mut extension_names =
        ash_window::enumerate_required_extensions(window.display_handle().unwrap().into())?
            .to_vec();
    extension_names.push(debug_utils::NAME.as_ptr());

    let instance_create_info = vk::InstanceCreateInfo::default()
        .application_info(&app_info)
        .enabled_extension_names(&extension_names);

    let instance = unsafe { entry.create_instance(&instance_create_info, None)? };

    // Vulkan debug report
    let create_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
        .flags(vk::DebugUtilsMessengerCreateFlagsEXT::empty())
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        )
        .pfn_user_callback(Some(vulkan_debug_callback));
    let debug_utils = debug_utils::Instance::new(entry, &instance);
    let debug_utils_messenger =
        unsafe { debug_utils.create_debug_utils_messenger(&create_info, None)? };

    Ok((instance, debug_utils, debug_utils_messenger))
}

unsafe extern "system" fn vulkan_debug_callback(
    flag: vk::DebugUtilsMessageSeverityFlagsEXT,
    typ: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void,
) -> vk::Bool32 {
    use vk::DebugUtilsMessageSeverityFlagsEXT as Flag;

    let message = CStr::from_ptr((*p_callback_data).p_message);
    match flag {
        Flag::VERBOSE => log::debug!("{typ:?} - {message:?}"),
        Flag::INFO => log::info!("{typ:?} - {message:?}"),
        Flag::WARNING => log::warn!("{typ:?} - {message:?}"),
        _ => log::error!("{typ:?} - {message:?}"),
    }
    vk::FALSE
}

fn create_vulkan_physical_device_and_get_graphics_and_present_qs_indices(
    instance: &Instance,
    surface: &surface::Instance,
    surface_khr: vk::SurfaceKHR,
) -> Result<(vk::PhysicalDevice, u32, u32), Box<dyn Error>> {
    log::debug!("Creating vulkan physical device");
    let devices = unsafe { instance.enumerate_physical_devices()? };
    let mut graphics = None;
    let mut present = None;
    let device = devices
        .into_iter()
        .find(|device| {
            let device = *device;

            // Does device supports graphics and present queues
            let props = unsafe { instance.get_physical_device_queue_family_properties(device) };
            for (index, family) in props.iter().filter(|f| f.queue_count > 0).enumerate() {
                let index = index as u32;
                graphics = None;
                present = None;

                if family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                    && family.queue_flags.contains(vk::QueueFlags::COMPUTE)
                    && graphics.is_none()
                {
                    graphics = Some(index);
                }

                let present_support = unsafe {
                    surface
                        .get_physical_device_surface_support(device, index, surface_khr)
                        .expect("Failed to get surface support")
                };
                if present_support && present.is_none() {
                    present = Some(index);
                }

                if graphics.is_some() && present.is_some() {
                    break;
                }
            }

            // Does device support desired extensions
            let extension_props = unsafe {
                instance
                    .enumerate_device_extension_properties(device)
                    .expect("Failed to get device ext properties")
            };
            let extension_support = extension_props.iter().any(|ext| {
                let name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
                swapchain::NAME == name
            });

            // Does the device have available formats for the given surface
            let formats = unsafe {
                surface
                    .get_physical_device_surface_formats(device, surface_khr)
                    .expect("Failed to get physical device surface formats")
            };

            // Does the device have available present modes for the given surface
            let present_modes = unsafe {
                surface
                    .get_physical_device_surface_present_modes(device, surface_khr)
                    .expect("Failed to get physical device surface present modes")
            };

            graphics.is_some()
                && present.is_some()
                && extension_support
                && !formats.is_empty()
                && !present_modes.is_empty()
        })
        .expect("Could not find a suitable device");

    unsafe {
        let props = instance.get_physical_device_properties(device);
        let device_name = CStr::from_ptr(props.device_name.as_ptr());
        log::debug!("Selected physical device: {device_name:?}");
    }

    Ok((device, graphics.unwrap(), present.unwrap()))
}

fn create_vulkan_device_and_graphics_and_present_qs(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    graphics_q_index: u32,
    present_q_index: u32,
) -> Result<(Device, vk::Queue, vk::Queue), Box<dyn Error>> {
    log::debug!("Creating vulkan device and graphics and present queues");
    let queue_priorities = [1.0f32];
    let queue_create_infos = {
        let mut indices = vec![graphics_q_index, present_q_index];
        indices.dedup();

        indices
            .iter()
            .map(|index| {
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(*index)
                    .queue_priorities(&queue_priorities)
            })
            .collect::<Vec<_>>()
    };

    let device_extensions_ptrs = [swapchain::NAME.as_ptr()];

    let device_create_info = vk::DeviceCreateInfo::default()
        .queue_create_infos(&queue_create_infos)
        .enabled_extension_names(&device_extensions_ptrs);

    let device = unsafe { instance.create_device(physical_device, &device_create_info, None)? };
    let graphics_queue = unsafe { device.get_device_queue(graphics_q_index, 0) };
    let present_queue = unsafe { device.get_device_queue(present_q_index, 0) };

    Ok((device, graphics_queue, present_queue))
}

type CreateSwapchainResult = (
    swapchain::Device,
    vk::SwapchainKHR,
    vk::Extent2D,
    vk::Format,
    Vec<vk::Image>,
    Vec<vk::ImageView>,
);

fn create_vulkan_swapchain(
    vulkan_context: &VulkanContext,
) -> Result<CreateSwapchainResult, Box<dyn Error>> {
    log::debug!("Creating vulkan swapchain");
    // Swapchain format
    let format = {
        let formats = unsafe {
            vulkan_context.surface.get_physical_device_surface_formats(
                vulkan_context.physical_device,
                vulkan_context.surface_khr,
            )?
        };
        if formats.len() == 1 && formats[0].format == vk::Format::UNDEFINED {
            vk::SurfaceFormatKHR {
                format: vk::Format::B8G8R8A8_UNORM,
                color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
            }
        } else {
            *formats
                .iter()
                .find(|format| {
                    format.format == vk::Format::B8G8R8A8_UNORM
                        && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
                })
                .unwrap_or(&formats[0])
        }
    };
    log::debug!("Swapchain format: {format:?}");

    // Swapchain present mode
    let present_mode = {
        let present_modes = unsafe {
            vulkan_context
                .surface
                .get_physical_device_surface_present_modes(
                    vulkan_context.physical_device,
                    vulkan_context.surface_khr,
                )?
        };
        if present_modes.contains(&vk::PresentModeKHR::IMMEDIATE) {
            vk::PresentModeKHR::IMMEDIATE
        } else {
            vk::PresentModeKHR::FIFO
        }
    };
    log::debug!("Swapchain present mode: {present_mode:?}");

    let capabilities = unsafe {
        vulkan_context
            .surface
            .get_physical_device_surface_capabilities(
                vulkan_context.physical_device,
                vulkan_context.surface_khr,
            )?
    };

    // Swapchain extent
    let extent = {
        if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            let min = capabilities.min_image_extent;
            let max = capabilities.max_image_extent;
            let width = WIDTH.min(max.width).max(min.width);
            let height = HEIGHT.min(max.height).max(min.height);
            vk::Extent2D { width, height }
        }
    };
    log::debug!("Swapchain extent: {extent:?}");

    // Swapchain image count
    let image_count = capabilities.min_image_count;
    log::debug!("Swapchain image count: {image_count:?}");

    // Swapchain
    let families_indices = [
        vulkan_context.graphics_q_index,
        vulkan_context.present_q_index,
    ];
    let create_info = {
        let mut builder = vk::SwapchainCreateInfoKHR::default()
            .surface(vulkan_context.surface_khr)
            .min_image_count(image_count)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT);

        builder = if vulkan_context.graphics_q_index != vulkan_context.present_q_index {
            builder
                .image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(&families_indices)
        } else {
            builder.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        };

        builder
            .pre_transform(capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
    };

    let swapchain = swapchain::Device::new(&vulkan_context.instance, &vulkan_context.device);
    let swapchain_khr = unsafe { swapchain.create_swapchain(&create_info, None)? };

    // Swapchain images and image views
    let images = unsafe { swapchain.get_swapchain_images(swapchain_khr)? };
    let views = images
        .iter()
        .map(|image| {
            let create_info = vk::ImageViewCreateInfo::default()
                .image(*image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(format.format)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });

            unsafe { vulkan_context.device.create_image_view(&create_info, None) }
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok((
        swapchain,
        swapchain_khr,
        extent,
        format.format,
        images,
        views,
    ))
}

fn create_vulkan_render_pass(
    device: &Device,
    format: vk::Format,
) -> Result<vk::RenderPass, Box<dyn Error>> {
    log::debug!("Creating vulkan render pass");
    let attachment_descs = [vk::AttachmentDescription::default()
        .format(format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)];

    let color_attachment_refs = [vk::AttachmentReference::default()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];

    let subpass_descs = [vk::SubpassDescription::default()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(&color_attachment_refs)];

    let subpass_deps = [vk::SubpassDependency::default()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_access_mask(
            vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        )];

    let render_pass_info = vk::RenderPassCreateInfo::default()
        .attachments(&attachment_descs)
        .subpasses(&subpass_descs)
        .dependencies(&subpass_deps);

    Ok(unsafe { device.create_render_pass(&render_pass_info, None)? })
}

fn create_vulkan_framebuffers(
    device: &Device,
    render_pass: vk::RenderPass,
    extent: vk::Extent2D,
    image_views: &[vk::ImageView],
) -> Result<Vec<vk::Framebuffer>, Box<dyn Error>> {
    log::debug!("Creating vulkan framebuffers");
    Ok(image_views
        .iter()
        .map(|view| [*view])
        .map(|attachments| {
            let framebuffer_info = vk::FramebufferCreateInfo::default()
                .render_pass(render_pass)
                .attachments(&attachments)
                .width(extent.width)
                .height(extent.height)
                .layers(1);
            unsafe { device.create_framebuffer(&framebuffer_info, None) }
        })
        .collect::<Result<Vec<_>, _>>()?)
}

#[allow(clippy::too_many_arguments)]
fn record_command_buffers(
    device: &Device,
    command_pool: vk::CommandPool,
    command_buffer: vk::CommandBuffer,
    framebuffer: vk::Framebuffer,
    render_pass: vk::RenderPass,
    extent: vk::Extent2D,
    renderer: &mut Renderer,
    draw_data: &DrawData,
) -> Result<(), Box<dyn Error>> {
    unsafe { device.reset_command_pool(command_pool, vk::CommandPoolResetFlags::empty())? };

    let command_buffer_begin_info =
        vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);
    unsafe { device.begin_command_buffer(command_buffer, &command_buffer_begin_info)? };

    let render_pass_begin_info = vk::RenderPassBeginInfo::default()
        .render_pass(render_pass)
        .framebuffer(framebuffer)
        .render_area(vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent,
        })
        .clear_values(&[vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [1.0, 1.0, 1.0, 1.0],
            },
        }]);

    unsafe {
        device.cmd_begin_render_pass(
            command_buffer,
            &render_pass_begin_info,
            vk::SubpassContents::INLINE,
        )
    };

    renderer.cmd_draw(command_buffer, draw_data)?;

    unsafe { device.cmd_end_render_pass(command_buffer) };

    unsafe { device.end_command_buffer(command_buffer)? };

    Ok(())
}
