pub mod lexer;
pub mod ast;
pub mod parser;
pub mod codegen;

use vidra_core::VidraError;

/// Compile VidraFX source code into WGSL compute shader source.
pub fn compile(src: &str) -> Result<String, VidraError> {
    let mut lexer = lexer::Lexer::new(src);
    let tokens = lexer.tokenize()?;
    let mut parser = parser::Parser::new(tokens, src);
    let ast = parser.parse()?;
    
    let mut codegen = codegen::Codegen::new();
    let wgsl = codegen.generate(&ast)?;
    
    Ok(wgsl)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_fire_glow() {
        let src = r#"
        @effect fireGlow(intensity: 1.0) {
            let flames = noise(3.0, 0.5)
                -> mask(0.3, 0.7)
                -> brightness(intensity)
                -> tint(#FF661A)
            blend(source(), flames, 0.5)
        }
        "#;

        let wgsl = compile(src).expect("Compilation failed");
        assert!(wgsl.contains("fn fbm"));
        assert!(wgsl.contains("smoothstep")); // mask
        assert!(wgsl.contains("textureLoad")); // source
        assert!(wgsl.contains("mix")); // blend
        
        println!("Generated WGSL:\n{}", wgsl);
    }
}
