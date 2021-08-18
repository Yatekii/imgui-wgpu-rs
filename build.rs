use std::{env, fs, path::Path};

use naga::{
    back::spv::{self, WriterFlags},
    front::glsl,
    valid::{Capabilities, ValidationFlags, Validator},
    FastHashMap,
};

fn compile_shader(
    input_path: &str,
    output_file_name: &str,
    stage: naga::ShaderStage,
    linear_output: bool,
) {
    // Load and parse the shader source.
    let module_src = fs::read_to_string(input_path).unwrap();

    let mut defines = FastHashMap::default();
    if linear_output {
        defines.insert("LINEAR_OUTPUT".to_string(), "true".to_string());
    }

    let mut parser = glsl::Parser::default();
    let module = parser
        .parse(&glsl::Options { stage, defines }, &module_src)
        .unwrap();

    // Validate the IR.
    let info =
        match Validator::new(ValidationFlags::default(), Capabilities::all()).validate(&module) {
            Ok(info) => Some(info),
            Err(error) => {
                eprintln!("{}", error);
                None
            }
        };

    // Write out as SPIR-V.
    let out_opts = spv::Options {
        flags: WriterFlags::empty(),
        ..Default::default()
    };
    let spv = spv::write_vec(&module, info.as_ref().unwrap(), &out_opts).unwrap();
    let bytes = spv
        .iter()
        .fold(Vec::with_capacity(spv.len() * 4), |mut v, w| {
            v.extend_from_slice(&w.to_le_bytes());
            v
        });

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join(output_file_name);
    fs::write(dest_path, bytes.as_slice()).unwrap();
}

fn main() {
    compile_shader(
        "src/imgui.vert",
        "imgui.vert.spv",
        naga::ShaderStage::Vertex,
        false,
    );
    compile_shader(
        "src/imgui.frag",
        "imgui-srgb.frag.spv",
        naga::ShaderStage::Fragment,
        false,
    );
    compile_shader(
        "src/imgui.frag",
        "imgui-linear.frag.spv",
        naga::ShaderStage::Fragment,
        true,
    );

    println!("cargo:rerun-if-changed=src/imgui.vert");
    println!("cargo:rerun-if-changed=src/imgui.frag");
}
