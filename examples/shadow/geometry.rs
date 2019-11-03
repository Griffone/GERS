use gaclen::graphics::device::Device;

use gaclen::vulkano::buffer::CpuAccessibleBuffer;

use std::sync::Arc;

pub type Mat4 = [[f32; 4]; 4];

#[derive(Default, Debug, Clone)]
pub struct Vertex {
	pos: [f32; 3],
	col: [f32; 4],
}
gaclen::vulkano::impl_vertex!(Vertex, pos, col);


#[derive(Default, Debug, Clone)]
pub struct Instance {
	model_matrix: Mat4,
	viewprojection_matrix: Mat4,
	light_matrix: Mat4,
}
gaclen::vulkano::impl_vertex!(Instance, model_matrix, viewprojection_matrix, light_matrix);

/// Generate a buffer with quad geometry.
pub fn generate_quad(device: &Device) -> Arc<CpuAccessibleBuffer<[Vertex]>> {
	device.create_buffer([
		Vertex { pos: [-0.5, 0.5, 0.0 ], col: [ 0.75, 0.75, 0.75, 1.0 ] },
		Vertex { pos: [ 0.5,-0.5, 0.0 ], col: [ 0.75, 0.75, 0.75, 1.0 ] },
		Vertex { pos: [ 0.5, 0.5, 0.0 ], col: [ 0.75, 0.75, 0.75, 1.0 ] },
		Vertex { pos: [-0.5,-0.5, 0.0 ], col: [ 0.75, 0.75, 0.75, 1.0 ] },
		Vertex { pos: [ 0.5,-0.5, 0.0 ], col: [ 0.75, 0.75, 0.75, 1.0 ] },
		Vertex { pos: [-0.5, 0.5, 0.0 ], col: [ 0.75, 0.75, 0.75, 1.0 ] },
	].iter().cloned()).unwrap()
}

/// Generate a buffer with cube geometry.
pub fn generate_cube(device: &Device) -> Arc<CpuAccessibleBuffer<[Vertex]>> {
	device.create_buffer([
		// Z+
		Vertex { pos: [-0.5, 0.5, 0.5 ], col: [ 0.5, 0.5, 1.0, 1.0 ] },
		Vertex { pos: [ 0.5,-0.5, 0.5 ], col: [ 0.5, 0.5, 1.0, 1.0 ] },
		Vertex { pos: [ 0.5, 0.5, 0.5 ], col: [ 0.5, 0.5, 1.0, 1.0 ] },
		Vertex { pos: [-0.5,-0.5, 0.5 ], col: [ 0.5, 0.5, 1.0, 1.0 ] },
		Vertex { pos: [ 0.5,-0.5, 0.5 ], col: [ 0.5, 0.5, 1.0, 1.0 ] },
		Vertex { pos: [-0.5, 0.5, 0.5 ], col: [ 0.5, 0.5, 1.0, 1.0 ] },
		// Z-
		Vertex { pos: [-0.5, 0.5, -0.5 ], col: [ 0.5, 0.5, 0.0, 1.0 ] },
		Vertex { pos: [ 0.5, 0.5, -0.5 ], col: [ 0.5, 0.5, 0.0, 1.0 ] },
		Vertex { pos: [ 0.5,-0.5, -0.5 ], col: [ 0.5, 0.5, 0.0, 1.0 ] },
		Vertex { pos: [-0.5,-0.5, -0.5 ], col: [ 0.5, 0.5, 0.0, 1.0 ] },
		Vertex { pos: [-0.5, 0.5, -0.5 ], col: [ 0.5, 0.5, 0.0, 1.0 ] },
		Vertex { pos: [ 0.5,-0.5, -0.5 ], col: [ 0.5, 0.5, 0.0, 1.0 ] },
	].iter().cloned()).unwrap()
}