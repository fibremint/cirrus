fn main() {
  let mut codegen = tauri_build::CodegenContext::new();
  if !cfg!(feature = "custom-protocol") {
    codegen = codegen.dev();
  }
  codegen.build();
  tauri_build::build();
}
