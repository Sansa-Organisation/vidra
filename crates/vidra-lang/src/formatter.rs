use crate::ast::*;

pub struct Formatter {
    indent_level: usize,
    output: String,
}

impl Formatter {
    pub fn format(ast: &ProjectNode) -> String {
        let mut formatter = Formatter {
            indent_level: 0,
            output: String::new(),
        };
        formatter.format_project(ast);
        formatter.output
    }

    fn indent(&mut self) {
        self.output.push_str(&"    ".repeat(self.indent_level));
    }

    fn push_line(&mut self, text: &str) {
        self.indent();
        self.output.push_str(text);
        self.output.push('\n');
    }

    fn push(&mut self, text: &str) {
        self.output.push_str(text);
    }

    fn format_project(&mut self, proj: &ProjectNode) {
        self.indent();
        self.push(&format!("project({}, {}, {}) {{\n", proj.width, proj.height, proj.fps.trunc()));
        self.indent_level += 1;

        let mut first = true;
        
        for comp in &proj.components {
            if !first {
                self.output.push('\n');
            }
            self.format_component(comp);
            first = false;
        }

        for scene in &proj.scenes {
            if !first {
                self.output.push('\n');
            }
            self.format_scene(scene);
            first = false;
        }

        self.indent_level -= 1;
        self.push_line("}");
    }

    fn format_component(&mut self, comp: &ComponentNode) {
        let params_str = comp.props.iter()
            .map(|p| format!("{}: {}", p.name, p.type_name))
            .collect::<Vec<_>>()
            .join(", ");
        self.push_line(&format!("component({}{}) {{", comp.name, if params_str.is_empty() { String::new() } else { format!(", {}", params_str) }));
        self.indent_level += 1;
        for item in &comp.items {
            self.format_layer_block_item(item);
        }
        self.indent_level -= 1;
        self.push_line("}");
    }

    fn format_scene(&mut self, scene: &SceneNode) {
        self.push_line(&format!("scene(\"{}\", {}s) {{", scene.name, scene.duration));
        self.indent_level += 1;
        for item in &scene.items {
            self.format_layer_block_item(item);
        }
        self.indent_level -= 1;
        self.push_line("}");
    }

    fn format_layer_block_item(&mut self, item: &LayerBlockItem) {
        match item {
            LayerBlockItem::Layer(layer) => self.format_layer(layer),
            LayerBlockItem::If { condition, then_branch, else_branch, .. } => {
                let cond_str = self.format_value(condition);
                self.indent();
                self.push(&format!("if ({}) {{\n", cond_str));
                self.indent_level += 1;
                for b in then_branch {
                    self.format_layer_block_item(b);
                }
                self.indent_level -= 1;
                
                if let Some(eb) = else_branch {
                    self.indent();
                    self.push("} else {\n");
                    self.indent_level += 1;
                    for b in eb {
                        self.format_layer_block_item(b);
                    }
                    self.indent_level -= 1;
                }
                self.push_line("}");
            }
        }
    }

    fn format_layer(&mut self, layer: &LayerNode) {
        self.indent();
        self.push(&format!("layer(\"{}\") {{\n", layer.name));
        self.indent_level += 1;
        
        self.format_layer_content(&layer.content);
        
        for prop in &layer.properties {
            self.format_property(prop);
        }

        for child in &layer.children {
            self.format_layer_block_item(child);
        }

        self.indent_level -= 1;
        self.push_line("}");
    }

    fn format_layer_content(&mut self, content: &LayerContentNode) {
        match content {
            LayerContentNode::Text { text: text_content, args, .. } => {
                self.format_content_func("text", text_content, args);
            }
            LayerContentNode::Image { path, args, .. } => {
                self.format_content_func("image", path, args);
            }
            LayerContentNode::Video { path, args, .. } => {
                self.format_content_func("video", path, args);
            }
            LayerContentNode::Audio { path, args, .. } => {
                self.format_content_func("audio", path, args);
            }
            LayerContentNode::Shape { shape_type, args, .. } => {
                let type_ident = ValueNode::Identifier(shape_type.clone());
                self.format_content_func("shape", &type_ident, args);
            }
            LayerContentNode::Solid { color, .. } => {
                let col_str = self.format_value(color);
                self.push_line(&format!("solid({})", col_str));
            }
            LayerContentNode::Component { name, args, .. } => {
                let args_str = args.iter()
                    .map(|a| format!("{}: {}", a.name, self.format_value(&a.value)))
                    .collect::<Vec<_>>()
                    .join(", ");
                self.push_line(&format!("use(\"{}\"{})", name, if args.is_empty() { String::new() } else { format!(", {}", args_str) }));
            }
            LayerContentNode::Slot => {
                self.push_line("slot()");
            }
            LayerContentNode::TTS { text, voice, args } => {
                let text_str = self.format_value(text);
                let voice_str = self.format_value(voice);
                let args_str = args.iter()
                    .map(|a| format!("{}: {}", a.name, self.format_value(&a.value)))
                    .collect::<Vec<_>>()
                    .join(", ");
                if args.is_empty() {
                    self.push_line(&format!("tts({}, {})", text_str, voice_str));
                } else {
                    self.push_line(&format!("tts({}, {}, {})", text_str, voice_str, args_str));
                }
            }
            LayerContentNode::AutoCaption { audio_source, args } => {
                self.format_content_func("autocaption", audio_source, args);
            }
            LayerContentNode::Empty => {}
        }
    }

    fn format_content_func(&mut self, name: &str, primary_arg: &ValueNode, args: &[NamedArg]) {
        let primary = self.format_value(primary_arg);
        if args.is_empty() {
            self.push_line(&format!("{}({})", name, primary));
        } else {
            let args_str = args.iter()
                .map(|a| format!("{}: {}", a.name, self.format_value(&a.value)))
                .collect::<Vec<_>>()
                .join(", ");
            self.push_line(&format!("{}({}, {})", name, primary, args_str));
        }
    }

    fn format_property(&mut self, prop: &PropertyNode) {
        match prop {
            PropertyNode::Position { x, y, .. } => {
                let x_str = self.format_value(x);
                let y_str = self.format_value(y);
                self.push_line(&format!("position({}, {})", x_str, y_str));
            }
            PropertyNode::Animation { property, args, .. } => {
                let args_str = args.iter()
                    .map(|a| format!("{}: {}", a.name, self.format_value(&a.value)))
                    .collect::<Vec<_>>()
                    .join(", ");
                self.push_line(&format!("animation({}, {})", property, args_str));
            }
            PropertyNode::FunctionCall { name, args, named_args, .. } => {
                let args_str = args.iter()
                    .map(|a| self.format_value(a))
                    .collect::<Vec<_>>()
                    .join(", ");
                
                let kwargs_str = named_args.iter()
                    .map(|a| format!("{}: {}", a.name, self.format_value(&a.value)))
                    .collect::<Vec<_>>()
                    .join(", ");
                    
                if named_args.is_empty() {
                    self.push_line(&format!("{}({})", name, args_str));
                } else if args.is_empty() {
                    self.push_line(&format!("{}({})", name, kwargs_str));
                } else {
                    self.push_line(&format!("{}({}, {})", name, args_str, kwargs_str));
                }
            }
        }
    }

    fn format_value(&self, val: &ValueNode) -> String {
        match val {
            ValueNode::Number(value) => value.to_string(),
            ValueNode::String(value) => format!("\"{}\"", value.replace("\"", "\\\"")),
            ValueNode::Duration(value) => format!("{}s", value),
            ValueNode::Color(hex) => format!("#{}", hex),
            ValueNode::Identifier(name) => name.clone(),
            ValueNode::BrandReference(key) => format!("@brand.{}", key),
        }
    }
}
