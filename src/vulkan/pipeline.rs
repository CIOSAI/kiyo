use std::collections::HashMap;
use std::{fmt, fs};
use ash::vk;
use ash::vk::ShaderModule;
use log::{info};

pub trait Pipeline {
    fn handle(&self) -> vk::Pipeline;
    fn bind_point(&self) -> vk::PipelineBindPoint;
    fn layout(&self) -> vk::PipelineLayout;
}

pub fn create_shader_module(device: &ash::Device, code: Vec<u32>) -> ShaderModule {
    let shader_module_create_info = vk::ShaderModuleCreateInfo::default()
        .code(unsafe { std::slice::from_raw_parts(code.as_ptr(), code.len()) });

    unsafe {
        device
            .create_shader_module(&shader_module_create_info, None)
            .expect("Failed to create shader module")
    }
}

#[derive(Debug)]
pub enum PipelineErr {
    ShaderCompilation(String)
}

impl fmt::Display for PipelineErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PipelineErr::ShaderCompilation(ref err) => {
                write!(f, "{}", err)
            },
        }
    }
}

/**
 * Load a shader from a file and compile it into SPIR-V.
 */
pub fn load_shader_code(source_file: String, macros: &HashMap<&str, &dyn ToString>) -> Result<Vec<u32>, PipelineErr>
{
    use shaderc;

    let shader_kind = match source_file.split(".").last() {
        Some("vert") => shaderc::ShaderKind::Vertex,
        Some("frag") => shaderc::ShaderKind::Fragment,
        Some("comp") => shaderc::ShaderKind::Compute,
        _ => panic!("Unknown shader type")
    };

    let source = fs::read_to_string(source_file.clone()).expect(format!("Failed to read file: {}", source_file).as_str());

    let compiler = shaderc::Compiler::new().unwrap();
    let mut options = shaderc::CompileOptions::new().unwrap();
    options.add_macro_definition("EP", Some("main"));
    for ( k, v ) in macros {
        options.add_macro_definition(k, Some(v.to_string().as_str()));
    }

    let binary_result = compiler.compile_into_spirv(
        source.as_str(),
        shader_kind,
        source_file.as_str(),
        "main",
        Some(&options)
    );

    match binary_result {
        Ok(result) => {
            info!("Successfully compiled shader: {}", source_file);
            Ok(result.as_binary().to_vec())
        },
        Err(error) => {
            Err(PipelineErr::ShaderCompilation(error.to_string()))
        }
    }
}
