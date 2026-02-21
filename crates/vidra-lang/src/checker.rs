use crate::ast::*;
use crate::lexer::Span;

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub file: String,
    pub span: Span,
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = match self.severity {
            DiagnosticSeverity::Error => "error",
            DiagnosticSeverity::Warning => "warning",
            DiagnosticSeverity::Info => "info",
        };
        write!(f, "{prefix}: {} at {}:{}:{}", self.message, self.file, self.span.line, self.span.column)
    }
}

/// Static Type Checker and Linter that traverses the parsed AST before compilation to IR.
/// Ensures all properties, bindings, and function calls are well-typed, and checks for potential issues.
pub struct TypeChecker {
    diagnostics: Vec<Diagnostic>,
    file: String,
    components: HashMap<String, ComponentNode>,
    used_components: std::collections::HashSet<String>,
    current_scope_layers: std::collections::HashSet<String>,
}

impl TypeChecker {
    pub fn new(file: impl Into<String>) -> Self {
        Self {
            diagnostics: Vec::new(),
            file: file.into(),
            components: HashMap::new(),
            used_components: std::collections::HashSet::new(),
            current_scope_layers: std::collections::HashSet::new(),
        }
    }

    /// Run the type checker on the project AST. Returns diagnostics, Error if there are any hard errors.
    pub fn check(mut self, project: &ProjectNode) -> Result<Vec<Diagnostic>, Vec<Diagnostic>> {
        self.check_project(project);
        
        let has_errors = self.diagnostics.iter().any(|d| d.severity == DiagnosticSeverity::Error);
        if has_errors {
            Err(self.diagnostics)
        } else {
            Ok(self.diagnostics)
        }
    }

    fn add_diagnostic(&mut self, severity: DiagnosticSeverity, message: impl Into<String>, span: &Span) {
        self.diagnostics.push(Diagnostic {
            severity,
            message: message.into(),
            file: self.file.clone(),
            span: span.clone(),
        });
    }

    fn type_error(&mut self, message: impl Into<String>, span: &Span) {
        self.add_diagnostic(DiagnosticSeverity::Error, message, span);
    }

    fn check_project(&mut self, proj: &ProjectNode) {
        // Collect components
        for comp in &proj.components {
            if self.components.contains_key(&comp.name) {
                self.type_error(format!("duplicate component definition '{}'", comp.name), &comp.span);
            }
            self.components.insert(comp.name.clone(), comp.clone());
        }

        // Check components internally
        for comp in &proj.components {
            self.current_scope_layers.clear();
            for item in &comp.items {
                self.check_layer_block_item(item);
            }
        }

        // Check scenes
        for scene in &proj.scenes {
            self.check_scene(scene);
        }

        let unused: Vec<_> = self.components.iter()
            .filter(|(name, _)| !self.used_components.contains(*name))
            .map(|(name, comp)| (name.clone(), comp.span.clone()))
            .collect();

        for (name, span) in unused {
            self.add_diagnostic(DiagnosticSeverity::Warning, format!("component '{}' is never used", name), &span);
        }

        // Check assets
        for asset in &proj.assets {
            match asset.asset_type.as_str() {
                "font" | "image" | "video" | "audio" => {}
                _ => self.type_error(format!("unknown asset type '{}'", asset.asset_type), &asset.span),
            }
        }
    }

    fn check_scene(&mut self, scene: &SceneNode) {
        self.current_scope_layers.clear();
        for item in &scene.items {
            self.check_layer_block_item(item);
        }
    }

    fn check_layer_block_item(&mut self, item: &crate::ast::LayerBlockItem) {
        match item {
            crate::ast::LayerBlockItem::Layer(layer) => self.check_layer(layer),
            crate::ast::LayerBlockItem::If { condition: _, then_branch, else_branch, .. } => {
                // Technically condition can be a variable Identifier or literal (usually checked at runtime/compile-eval time)
                // We just recursively check the branches
                for child_item in then_branch {
                    self.check_layer_block_item(child_item);
                }
                if let Some(else_items) = else_branch {
                    for child_item in else_items {
                        self.check_layer_block_item(child_item);
                    }
                }
            }
        }
    }

    fn check_layer(&mut self, layer: &LayerNode) {
        if !self.current_scope_layers.insert(layer.name.clone()) {
            self.add_diagnostic(DiagnosticSeverity::Warning, format!("duplicate layer name '{}' in current scope", layer.name), &layer.span);
        }

        self.check_layer_content(&layer.content, &layer.span);
        
        for prop in &layer.properties {
            self.check_property(prop);
        }
        
        for child in &layer.children {
            self.check_layer_block_item(child);
        }
    }

    fn check_layer_content(&mut self, content: &LayerContentNode, span: &Span) {
        match content {
            LayerContentNode::Text { text, args, .. } => {
                self.expect_string(text, span);
                for arg in args {
                    match arg.name.as_str() {
                        "font" => self.expect_string(&arg.value, &arg.span),
                        "size" => self.expect_number(&arg.value, &arg.span),
                        "color" => self.expect_color(&arg.value, &arg.span),
                        _ => self.type_error(format!("unknown property '{}' for text layer", arg.name), &arg.span),
                    }
                }
            }
            LayerContentNode::Image { path, args: _ } => {
                self.expect_string(path, span);
            }
            LayerContentNode::Video { path, args: _ } => {
                self.expect_string(path, span);
            }
            LayerContentNode::Audio { path, args: _ } => {
                self.expect_string(path, span);
            }
            LayerContentNode::TTS { text, voice, args: _ } => {
                self.expect_string(text, span);
                self.expect_string(voice, span);
            }
            LayerContentNode::AutoCaption { audio_source, args: _ } => {
                self.expect_string(audio_source, span);
            }
            LayerContentNode::Solid { color } => {
                self.expect_color(color, span);
            }
            LayerContentNode::Shape { shape_type, args } => {
                for arg in args {
                    match arg.name.as_str() {
                        "fill" => self.expect_color(&arg.value, &arg.span),
                        "stroke" => self.expect_color(&arg.value, &arg.span),
                        "strokeWidth" => self.expect_number(&arg.value, &arg.span),
                        _ => self.type_error(format!("unknown property '{}' for shape {}", arg.name, shape_type), &arg.span),
                    }
                }
            }
            LayerContentNode::Component { name, args } => {
                self.used_components.insert(name.clone());
                if let Some(comp_def) = self.components.get(name).cloned() {
                    // Check provided args against definition
                    for arg in args {
                        let prop_def = comp_def.props.iter().find(|p| p.name == arg.name);
                        if let Some(prop_def) = prop_def {
                            // Validate type
                            match prop_def.type_name.as_str() {
                                "String" => self.expect_string(&arg.value, &arg.span),
                                "Number" => self.expect_number(&arg.value, &arg.span),
                                "Duration" => self.expect_duration_or_number(&arg.value, &arg.span),
                                "Color" => self.expect_color(&arg.value, &arg.span),
                                _ => self.type_error(format!("unknown type '{}' for property '{}'", prop_def.type_name, arg.name), &arg.span),
                            }
                        } else {
                            self.type_error(format!("component '{}' has no property named '{}'", name, arg.name), &arg.span);
                        }
                    }
                    
                    // Check missing required props (props without defaults)
                    for prop_def in &comp_def.props {
                        if prop_def.default_value.is_none() && !args.iter().any(|a| a.name == prop_def.name) {
                            self.type_error(format!("missing required property '{}' on component '{}'", prop_def.name, name), span);
                        }
                    }
                } else {
                    self.type_error(format!("unknown component '{}'", name), span);
                }
            }
            LayerContentNode::Slot => {}
            LayerContentNode::Empty => {}
        }
        
        // Suppress unused warning on span
        let _ = span;
    }

    fn check_property(&mut self, prop: &PropertyNode) {
        match prop {
            PropertyNode::Position { x, y, span } => {
                self.expect_number(x, span);
                self.expect_number(y, span);
            }
            PropertyNode::Animation { property: property_name, args, span } => {
                // Check valid properties
                let valid_props = [
                    "opacity", "position.x", "positionX", "x",
                    "position.y", "positionY", "y",
                    "scale.x", "scaleX", "scale.y", "scaleY", "scale",
                    "rotation"
                ];
                
                if !valid_props.contains(&property_name.as_str()) {
                    self.type_error(format!("cannot animate unknown property '{}'", property_name), span);
                }

                for arg in args {
                    match arg.name.as_str() {
                        "from" => self.expect_number(&arg.value, &arg.span),
                        "to" => self.expect_number(&arg.value, &arg.span),
                        "duration" => self.expect_duration_or_number(&arg.value, &arg.span),
                        "delay" => self.expect_duration_or_number(&arg.value, &arg.span),
                        "ease" | "easing" => self.expect_identifier(&arg.value, &arg.span),
                        _ => self.type_error(format!("unknown animation parameter '{}'", arg.name), &arg.span),
                    }
                }
            }
            PropertyNode::FunctionCall { name, span, .. } => {
                // Unknown function calls
                self.type_error(format!("unknown function or property '{}'", name), span);
            }
        }
    }

    // --- Type Assertions ---

    fn get_type_name(&self, value: &ValueNode) -> &'static str {
        match value {
            ValueNode::String(_) => "String",
            ValueNode::Number(_) => "Number",
            ValueNode::Duration(_) => "Duration",
            ValueNode::Color(_) => "Color",
            ValueNode::Identifier(_) => "Identifier",
            ValueNode::BrandReference(_) => "BrandReference",
        }
    }

    fn expect_number(&mut self, value: &ValueNode, span: &Span) {
        match value {
            ValueNode::Number(_) | ValueNode::Duration(_) | ValueNode::Identifier(_) | ValueNode::BrandReference(_) => {}, // Duration can cast to number implicitly in some cases, but restrict if strict
            _ => self.type_error(format!("expected Number, got {}", self.get_type_name(value)), span),
        }
    }

    fn expect_string(&mut self, value: &ValueNode, span: &Span) {
        match value {
            ValueNode::String(_) | ValueNode::Identifier(_) | ValueNode::BrandReference(_) => {},
            _ => self.type_error(format!("expected String, got {}", self.get_type_name(value)), span),
        }
    }

    fn expect_color(&mut self, value: &ValueNode, span: &Span) {
        match value {
            ValueNode::Color(_) | ValueNode::Identifier(_) | ValueNode::BrandReference(_) => {},
            _ => self.type_error(format!("expected Color, got {}", self.get_type_name(value)), span),
        }
    }

    fn expect_duration_or_number(&mut self, value: &ValueNode, span: &Span) {
        match value {
            ValueNode::Duration(_) | ValueNode::Number(_) | ValueNode::Identifier(_) | ValueNode::BrandReference(_) => {},
            _ => self.type_error(format!("expected Duration or Number, got {}", self.get_type_name(value)), span),
        }
    }

    fn expect_identifier(&mut self, value: &ValueNode, span: &Span) {
        match value {
            ValueNode::Identifier(_) | ValueNode::BrandReference(_) => {},
            _ => self.type_error(format!("expected Identifier, got {}", self.get_type_name(value)), span),
        }
    }
}
