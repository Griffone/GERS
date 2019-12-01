pub mod vertex {
	gaclen_shader::shader!{
		ty: "vertex",
		path: "examples/cube/shader.vert",
	}

	// force recompilation on changes in shader source
	const bytes: &'static [u8] = include_bytes!("shader.vert");
}
pub mod fragment {
	gaclen_shader::shader!{
		ty: "fragment",
		path: "examples/cube/shader.frag",
	}

	// force recompilation on changes in shader source
	const bytes: &'static [u8] = include_bytes!("shader.frag");
}
