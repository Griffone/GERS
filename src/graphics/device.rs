use std::sync::Arc;

use vulkano::instance::PhysicalDevice;
use vulkano::device::Device as LogicalDevice;
use vulkano::device::DeviceExtensions;
use vulkano::device::Queue as DeviceQueue;
use vulkano::swapchain::{Surface, Swapchain};
use vulkano::image::SwapchainImage;

use super::context::Context;

use crate::window::Window;


type ImageFormat = (vulkano::format::Format, vulkano::swapchain::ColorSpace);

// A graphical device responsible for using hardware acceleration.
pub struct Device<W> {
	device: Arc<LogicalDevice>,

	graphics_queue: Arc<DeviceQueue>,
	transfer_queue: Arc<DeviceQueue>,
	compute_queue: Arc<DeviceQueue>,

	swapchain: Arc<Swapchain<W>>,
	swapchain_images: Vec<Arc<SwapchainImage<W>>>,
}


#[derive(Debug)]
pub enum DeviceCreationError {
	NoPhysicalDevicesFound, // There were no physical devices to chose from
	NoCompatiblePhysicalDeviceFound, // Some physical devices were found but were not applicable for gaclen
	Logical(vulkano::device::DeviceCreationError), // Error during the creation of logical device
	Surface(vulkano::swapchain::SurfaceCreationError), // Error during the creation of the draw surface
	SurfaceCapabilities(vulkano::swapchain::CapabilitiesError), // Error querying draw surface capabilities
	Swapchain(vulkano::swapchain::SwapchainCreationError), // Error during the creation of the swapchain
	NoCompatibleFormatFound, // No compatible format for window draw surface was found
	UnsizedWindow, // Window has no inner size
}


impl<W> Device<W> {
	pub fn new(context: &Context, window: W) -> Result<Device<W>, DeviceCreationError>
	where W : vulkano_win::SafeBorrow<Window>
	{
		let physical = select_physical_device(context)?;

		let device_extensions = DeviceExtensions { khr_swapchain: true, .. DeviceExtensions::none() };
		let queues = select_queue_families(&physical);
		let (logical, mut queues) = LogicalDevice::new(physical, physical.supported_features(), &device_extensions, queues.iter().cloned())?;
		let graphics_queue = queues.next().unwrap();
		let transfer_queue = queues.next().unwrap();
		let compute_queue = queues.next().unwrap();

		let dimensions = match window.borrow().get_inner_size() {
			Some(size) => size,
			None => return Err(DeviceCreationError::UnsizedWindow),
		};
		let surface = vulkano_win::create_vk_surface(window, context.instance.clone())?;
		let (swapchain, swapchain_images) = create_swapchain(physical, logical.clone(), surface, dimensions.into(), &graphics_queue)?;

		let device = Device {
			device: logical,
			graphics_queue,
			transfer_queue,
			compute_queue,
			swapchain,
			swapchain_images,
		};

		Ok(device)
	}
}


impl From<vulkano::device::DeviceCreationError> for DeviceCreationError {
	fn from(error: vulkano::device::DeviceCreationError) -> DeviceCreationError { DeviceCreationError::Logical(error) }
}
impl From<vulkano::swapchain::SurfaceCreationError> for DeviceCreationError {
	fn from(error: vulkano::swapchain::SurfaceCreationError) -> DeviceCreationError { DeviceCreationError::Surface(error) }
}


fn select_physical_device(context: &Context) -> Result<PhysicalDevice, DeviceCreationError> {
	let mut devices = PhysicalDevice::enumerate(&context.instance);
	let mut device = match devices.next() {
		Some(device) => device,
		None => return Err(DeviceCreationError::NoPhysicalDevicesFound),
	};

	for other in devices { device = choose_better_device(device, other); };
	
	match physical_device_is_compatible(&device) {
		true => Ok(device),
		false => Err(DeviceCreationError::NoCompatiblePhysicalDeviceFound),
	}
}

fn create_swapchain<W>(
	physical_device: PhysicalDevice,
	logical_device: Arc<LogicalDevice>,
	surface: Arc<Surface<W>>,
	dimensions: (u32, u32),
	graphics_queue: &Arc<DeviceQueue>
) -> Result<(Arc<Swapchain<W>>, Vec<Arc<SwapchainImage<W>>>), DeviceCreationError>
{
	let capabilities = match surface.capabilities(physical_device) {
		Ok(caps) => caps,
		Err(err) => return Err(DeviceCreationError::SurfaceCapabilities(err)),
	};
	let usage = capabilities.supported_usage_flags;
	let alpha = capabilities.supported_composite_alpha.iter().next().unwrap();

	let format = select_format(capabilities.supported_formats)?;

	let swapchain = Swapchain::new(
		logical_device,
		surface,
		capabilities.min_image_count,
		format.0,
		[dimensions.0, dimensions.1],
		1,
		usage,
		graphics_queue,
		vulkano::swapchain::SurfaceTransform::Identity,
		alpha,
		vulkano::swapchain::PresentMode::Fifo,
		true,
		None);
	match swapchain {
		Ok(swapchain) => Ok(swapchain),
		Err(err) => Err(DeviceCreationError::Swapchain(err)),
	}
}


fn select_format(formats: Vec<ImageFormat>) -> Result<ImageFormat, DeviceCreationError> {
	if formats.is_empty() { return Err(DeviceCreationError::NoCompatibleFormatFound); }

	let mut format = formats[0];

	if cfg!(debug_assertions) {
		println!("Choosing format:");
	}

	for other in formats {
		if cfg!(debug_assertions) { print_image_format(&other, "  "); }
		format = choose_better_format(format, other);
	}

	if cfg!(debug_assertions) {
		println!();
		print_image_format(&format, "Chosen format: ");
	}
	Ok(format)
}

fn physical_device_is_compatible<'a>(device: &PhysicalDevice<'a>) -> bool {
	if cfg!(debug_assertions) {
		println!("Validating device:");
		print_physical_device_details(device, "  ", "    ");
	}

	if device.api_version() < super::REQUIRED_VULKAN_VERSION { return false; }

	let mut supports_graphics = false;
	let mut supports_compute = false;

	for family in device.queue_families() {
		supports_graphics = supports_graphics || (family.queues_count() > 0 && family.supports_graphics());
		supports_compute = supports_compute || (family.queues_count() > 0 && family.supports_compute());
	};

	supports_compute && supports_graphics
}

fn select_queue_families<'a>(device: &PhysicalDevice<'a>) -> [(vulkano::instance::QueueFamily<'a>, f32); 3] {
	let mut families = device.queue_families();
	let first = families.next().unwrap();

	let mut graphics = first.clone();
	let mut transfer = first.clone();
	let mut compute = first;

	for other in families {
		graphics = choose_better_graphics_family(graphics, other.clone());
		transfer = choose_better_transfer_family(transfer, other.clone());
		compute = choose_better_compute_family(compute, other);
	};

	if cfg!(debug_assertions) {
		println!("Selected queue families:");
		println!("Graphics:");
		print_queue_family_details(&graphics, "  ");
		println!("Transfer:");
		print_queue_family_details(&transfer, "  ");
		println!("Compute:");
		print_queue_family_details(&compute, "  ");
	}

	[
		(graphics, 1.0),
		(transfer, 0.5),
		(compute, 0.25),
	]
}

fn choose_better_device<'a>(first: PhysicalDevice<'a>, second: PhysicalDevice<'a>) -> PhysicalDevice<'a> {
	if !physical_device_is_compatible(&second) { return first; };

	// TODO: compare and select best device
	first
}

fn choose_better_format(first: ImageFormat, second: ImageFormat) -> ImageFormat {
	// TODO: compare and select better format
	first
}

fn choose_better_graphics_family<'a>(first: vulkano::instance::QueueFamily<'a>, second: vulkano::instance::QueueFamily<'a>) -> vulkano::instance::QueueFamily<'a> {
	if !second.supports_graphics() { return first; };

	// prefer exclusively graphics queue
	match second.supports_compute() {
		true => first,
		false => second
	}
}

fn choose_better_transfer_family<'a>(first: vulkano::instance::QueueFamily<'a>, second: vulkano::instance::QueueFamily<'a>) -> vulkano::instance::QueueFamily<'a> {
	if !second.explicitly_supports_transfers() { return first; };

	match second.supports_graphics() {
		true => first,
		false => match first.supports_graphics() {
			true => second,
			false => match second.supports_compute() {
				true => first,
				false => second,
			},
		},
	}
}

fn choose_better_compute_family<'a>(first: vulkano::instance::QueueFamily<'a>, second: vulkano::instance::QueueFamily<'a>) -> vulkano::instance::QueueFamily<'a> {
	if !second.supports_compute() { return first; };

	match second.supports_graphics() {
		true => first,
		false => second
	}
}

fn print_physical_device_details<'a>(device: &PhysicalDevice<'a>, prefix: &str, queue_family_prefix: &str) {
	println!("{}name: {}", prefix, device.name());
	println!("{}type: {:?}", prefix, device.ty());
	println!("{}api version: {}", prefix, device.api_version());
	println!("{}driver version: {}", prefix, device.driver_version());
	println!("{}memory types count: {}", prefix, device.memory_types().count());
	println!("{}queue families ({}):", prefix, device.queue_families().count());
	for family in device.queue_families() {
		print_queue_family_details(&family, queue_family_prefix);
		println!();
	}
}

fn print_queue_family_details<'a>(family: &vulkano::instance::QueueFamily<'a>, prefix: &str) {
	println!("{}id: {}", prefix, family.id());
	println!("{}count: {}", prefix, family.queues_count());
	println!("{}graphics: {}", prefix, family.supports_graphics());
	println!("{}compute: {}", prefix, family.supports_compute());
	println!("{}transfer: {}", prefix, family.explicitly_supports_transfers());
}

fn print_image_format(format: &ImageFormat, prefix: &str) {
	println!("{}format: {:?}, color space: {:?}", prefix, format.0, format.1);
}