use anyhow::Result;
use std::fs;

#[derive(Debug, Clone, Copy)]
pub struct TemplateInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub default_file: &'static str,
}

const TEMPLATE_SOCIAL_POST: &str = r#"
project(1080, 1920, 60) {
    scene("main", 15s) {
        layer("background") {
            solid(#1A1A1A)
        }

        layer("text_content") {
            text("Social Post Template", font: "Inter", size: 96, color: #FFFFFF)
            position(540, 960)
            
            animate(ScaleX) {
                0.0s -> 0.0 ~ EaseOut
                1.0s -> 1.0
                14.0s -> 1.0 ~ EaseIn
                15.0s -> 0.0
            }
            animate(ScaleY) {
                0.0s -> 0.0 ~ EaseOut
                1.0s -> 1.0
                14.0s -> 1.0 ~ EaseIn
                15.0s -> 0.0
            }
        }
    }
}
"#;

const TEMPLATE_LOWER_THIRD: &str = r#"
project(1920, 1080, 60) {
    scene("lower_third", 5s) {
        layer("container") {
            position(150, 850)
            
            layer("bg_bar") {
                shape(Rectangle(width: 600, height: 120, radius: 10))
                position(300, 60)
                fill(#007AFF)

                animate(PositionX) {
                    0.0s -> -300.0 ~ EaseOut
                    0.8s -> 300.0
                    4.2s -> 300.0 ~ EaseIn
                    5.0s -> -300.0
                }
            }

            layer("name_text") {
                text("John Doe", font: "Inter", size: 60, color: #FFFFFF)
                position(300, 45)

                animate(Opacity) {
                    0.7s -> 0.0 ~ Linear
                    1.2s -> 1.0
                    3.8s -> 1.0 ~ Linear
                    4.3s -> 0.0
                }
            }
            
            layer("title_text") {
                text("Software Engineer", font: "Inter", size: 30, color: #E0E0E0)
                position(300, 90)

                animate(Opacity) {
                    0.9s -> 0.0 ~ Linear
                    1.4s -> 1.0
                    3.6s -> 1.0 ~ Linear
                    4.1s -> 0.0
                }
            }
        }
    }
}
"#;

const TEMPLATE_BRANDED_INTRO: &str = r#"
project(1920, 1080, 60) {
    scene("intro", 3s) {
        layer("bg") {
            solid(#000000)
        }
        
        layer("logo") {
            text("BRAND", font: "Inter", size: 150, color: #FFFFFF)
            position(960, 500)
            
            animate(ScaleX) {
                0.0s -> 0.8 ~ EaseOut
                1.5s -> 1.0
            }
            animate(ScaleY) {
                0.0s -> 0.8 ~ EaseOut
                1.5s -> 1.0
            }
            animate(Opacity) {
                0.0s -> 0.0 ~ EaseIn
                1.0s -> 1.0
                2.5s -> 1.0 ~ EaseOut
                3.0s -> 0.0
            }
        }
        
        layer("tagline") {
            text("Built with Vidra", font: "Inter", size: 40, color: #888888)
            position(960, 600)
            
            animate(Opacity) {
                0.5s -> 0.0 ~ linear
                1.5s -> 1.0
                2.5s -> 1.0 ~ EaseOut
                3.0s -> 0.0
            }
        }
    }
}
"#;

pub fn available_templates() -> Vec<TemplateInfo> {
    vec![
        TemplateInfo {
            name: "social-post",
            description: "1080x1920 portrait template",
            default_file: "social_post.vidra",
        },
        TemplateInfo {
            name: "lower-third",
            description: "Broadcast-style lower third graphic",
            default_file: "lower_third.vidra",
        },
        TemplateInfo {
            name: "branded-intro",
            description: "Clean fade-in logo reveal",
            default_file: "branded_intro.vidra",
        },
    ]
}

pub fn execute_add(template_name: &str) -> Result<()> {
    let (content, default_name) = match template_name.to_lowercase().as_str() {
        "social-post" => (TEMPLATE_SOCIAL_POST, "social_post.vidra"),
        "lower-third" => (TEMPLATE_LOWER_THIRD, "lower_third.vidra"),
        "branded-intro" => (TEMPLATE_BRANDED_INTRO, "branded_intro.vidra"),
        _ => {
            println!("❌ Unknown template: '{}'", template_name);
            println!("\nAvailable templates:");
            for t in available_templates() {
                println!("  - {} ({})", t.name, t.description);
            }
            return Ok(());
        }
    };

    let cwd = std::env::current_dir()?;
    let out_path = cwd.join(default_name);
    
    // Check if it already exists to avoid silently overwriting
    if out_path.exists() {
        // Find variant name like lower_third_1.vidra
        println!("⚠️ File {} already exists.", default_name);
        let mut idx = 1;
        let mut alt_path = out_path.clone();
        while alt_path.exists() {
            alt_path = cwd.join(format!("{}_{}.vidra", template_name.replace("-", "_"), idx));
            idx += 1;
        }
        println!("Writing instead to: {}", alt_path.file_name().unwrap_or_default().to_string_lossy());
        fs::write(&alt_path, content.trim_start())?;
    } else {
        fs::write(&out_path, content.trim_start())?;
        println!("✅ Added template '{}' into {}", template_name, default_name);
    }

    Ok(())
}
