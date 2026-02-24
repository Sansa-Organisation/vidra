//! VidraScript parser — tokens → AST.

use crate::ast::*;
use crate::lexer::{Span, Token, TokenKind};
use vidra_core::VidraError;

/// The VidraScript parser.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    file: String,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, file: impl Into<String>) -> Self {
        Self {
            tokens,
            pos: 0,
            file: file.into(),
        }
    }

    /// Parse the token stream into a ProjectNode.
    pub fn parse(&mut self) -> Result<ProjectNode, VidraError> {
        self.skip_newlines();
        self.parse_project()
    }

    fn peek(&self) -> &TokenKind {
        self.tokens
            .get(self.pos)
            .map(|t| &t.kind)
            .unwrap_or(&TokenKind::Eof)
    }

    fn current_span(&self) -> Span {
        self.tokens
            .get(self.pos)
            .map(|t| t.span)
            .unwrap_or(Span::new(0, 0, 0, 0))
    }

    fn advance(&mut self) -> &Token {
        let token = &self.tokens[self.pos.min(self.tokens.len() - 1)];
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        token
    }

    fn expect(&mut self, expected: &TokenKind) -> Result<&Token, VidraError> {
        if std::mem::discriminant(self.peek()) == std::mem::discriminant(expected) {
            Ok(self.advance())
        } else {
            let span = self.current_span();
            Err(VidraError::parse(
                format!("expected {}, got {}", expected, self.peek()),
                &self.file,
                span.line,
                span.column,
            ))
        }
    }

    fn skip_newlines(&mut self) {
        while self.peek() == &TokenKind::Newline {
            self.advance();
        }
    }

    /// Parse: `project(width, height, fps) { ... }`
    /// or:   `project(WIDTHxHEIGHT, fps) { ... }`
    fn parse_project(&mut self) -> Result<ProjectNode, VidraError> {
        let span = self.current_span();
        self.expect(&TokenKind::Project)?;
        self.expect(&TokenKind::LeftParen)?;

        // Parse width
        let width = self.parse_number()? as u32;
        self.skip_newlines();

        // Could be 'x' for WIDTHxHEIGHT or ',' for WIDTH, HEIGHT
        let height;
        if self.peek() == &TokenKind::Identifier("x".into()) {
            self.advance(); // skip 'x'
            height = self.parse_number()? as u32;
        } else {
            self.expect(&TokenKind::Comma)?;
            self.skip_newlines();
            height = self.parse_number()? as u32;
        }

        self.skip_newlines();
        self.expect(&TokenKind::Comma)?;
        self.skip_newlines();

        // Parse fps — could be a number or a duration-like "30fps"
        let fps = self.parse_number()?;

        self.skip_newlines();
        self.expect(&TokenKind::RightParen)?;
        self.skip_newlines();
        self.expect(&TokenKind::LeftBrace)?;
        self.skip_newlines();

        // Parse scenes and components and imports
        let mut scenes = Vec::new();
        let mut components = Vec::new();
        let mut imports = Vec::new();
        let mut assets = Vec::new();
        let mut layout_rules = Vec::new();
        let mut variables = Vec::new();
        while self.peek() != &TokenKind::RightBrace && self.peek() != &TokenKind::Eof {
            self.skip_newlines();
            if self.peek() == &TokenKind::RightBrace {
                break;
            }
            if self.peek() == &TokenKind::Import {
                imports.push(self.parse_import()?);
            } else if self.peek() == &TokenKind::Component {
                components.push(self.parse_component()?);
            } else if self.peek() == &TokenKind::Asset {
                assets.push(self.parse_asset()?);
            } else if self.peek() == &TokenKind::Layout {
                layout_rules.push(self.parse_layout_rules()?);
            } else if self.peek() == &TokenKind::At {
                // Peek ahead to see if it's `@var`
                let next_token = &self.tokens[(self.pos + 1).min(self.tokens.len() - 1)];
                if let TokenKind::Identifier(id) = &next_token.kind {
                    if id == "var" {
                        variables.push(self.parse_var_def()?);
                    } else {
                        scenes.push(self.parse_scene()?);
                    }
                } else {
                    scenes.push(self.parse_scene()?);
                }
            } else {
                scenes.push(self.parse_scene()?);
            }
            self.skip_newlines();
        }

        self.expect(&TokenKind::RightBrace)?;

        Ok(ProjectNode {
            width,
            height,
            fps,
            imports,
            assets,
            layout_rules,
            variables,
            scenes,
            components,
            span,
        })
    }

    /// Parse an imported module: just imports and components
    pub fn parse_module(&mut self) -> Result<(Vec<ImportNode>, Vec<AssetNode>, Vec<ComponentNode>), VidraError> {
        let mut imports = Vec::new();
        let mut assets = Vec::new();
        let mut components = Vec::new();

        self.skip_newlines();
        while self.peek() != &TokenKind::Eof {
            if self.peek() == &TokenKind::Import {
                imports.push(self.parse_import()?);
            } else if self.peek() == &TokenKind::Asset {
                assets.push(self.parse_asset()?);
            } else if self.peek() == &TokenKind::Component {
                components.push(self.parse_component()?);
            } else {
                let span = self.current_span();
                return Err(VidraError::parse(
                    format!("expected import or component in module, got {}", self.peek()),
                    &self.file,
                    span.line,
                    span.column,
                ));
            }
            self.skip_newlines();
        }

        Ok((imports, assets, components))
    }

    /// Parse an asset block: `asset("id", font, "path")` or `asset(font, "id", "path")` 
    /* syntax: `asset(font, "Inter", "assets/Inter.ttf")` */
    fn parse_asset(&mut self) -> Result<AssetNode, VidraError> {
        let span = self.current_span();
        self.expect(&TokenKind::Asset)?;
        self.expect(&TokenKind::LeftParen)?;
        self.skip_newlines();

        let asset_type = self.parse_identifier()?;
        self.skip_newlines();
        self.expect(&TokenKind::Comma)?;
        self.skip_newlines();

        let id = self.parse_string()?;
        self.skip_newlines();
        self.expect(&TokenKind::Comma)?;
        self.skip_newlines();

        let path = self.parse_string()?;
        self.skip_newlines();
        self.expect(&TokenKind::RightParen)?;
        
        Ok(AssetNode {
            asset_type,
            id,
            path,
            span,
        })
    }

    /// Parse an import: `import "filename.vidra"`
    fn parse_import(&mut self) -> Result<ImportNode, VidraError> {
        let span = self.current_span();
        self.expect(&TokenKind::Import)?;
        self.skip_newlines();
        let path = self.parse_string()?;
        
        Ok(ImportNode {
            path,
            span,
        })
    }

    /// Parse layout rules: `layout rules { when aspect(16:9) { ... } }`
    fn parse_layout_rules(&mut self) -> Result<LayoutRulesNode, VidraError> {
        let span = self.current_span();
        self.expect(&TokenKind::Layout)?;
        self.expect(&TokenKind::Rules)?;
        self.skip_newlines();
        self.expect(&TokenKind::LeftBrace)?;
        self.skip_newlines();

        let mut rules = Vec::new();
        while self.peek() != &TokenKind::RightBrace && self.peek() != &TokenKind::Eof {
            self.skip_newlines();
            if self.peek() == &TokenKind::RightBrace {
                break;
            }
            rules.push(self.parse_layout_rule()?);
            self.skip_newlines();
        }

        self.expect(&TokenKind::RightBrace)?;
        self.skip_newlines();

        Ok(LayoutRulesNode {
            rules,
            span,
        })
    }

    /// Parse a variable definition: `@var name = value`
    fn parse_var_def(&mut self) -> Result<VarDefNode, VidraError> {
        let span = self.current_span();
        self.expect(&TokenKind::At)?;
        let kw = self.parse_identifier()?;
        if kw != "var" {
            return Err(VidraError::parse(
                format!("expected 'var', got '{}'", kw),
                &self.file,
                span.line,
                span.column,
            ));
        }
        
        let name = self.parse_identifier()?;
        self.skip_newlines();
        self.expect(&TokenKind::Equals)?;
        self.skip_newlines();
        let value = self.parse_value()?;
        
        Ok(VarDefNode {
            name,
            value,
            span,
        })
    }

    /// Parse a single layout rule: `when aspect(16:9) { ... }`
    fn parse_layout_rule(&mut self) -> Result<LayoutRuleNode, VidraError> {
        let span = self.current_span();
        self.expect(&TokenKind::When)?;
        self.expect(&TokenKind::Aspect)?;
        self.expect(&TokenKind::LeftParen)?;
        self.skip_newlines();
        // Parse "16:9" or similar
        // Because of the lexer, it might parse 16, Colon, 9. So we just parse until right paren.
        let mut aspect = String::new();
        while self.peek() != &TokenKind::RightParen && self.peek() != &TokenKind::Eof {
            aspect.push_str(&self.peek().to_string());
            self.advance();
        }
        self.expect(&TokenKind::RightParen)?;
        self.skip_newlines();
        
        // Block
        self.expect(&TokenKind::LeftBrace)?;
        self.skip_newlines();

        let mut items = Vec::new();
        while self.peek() != &TokenKind::RightBrace && self.peek() != &TokenKind::Eof {
            self.skip_newlines();
            if self.peek() == &TokenKind::RightBrace {
                break;
            }
            items.push(self.parse_layer_block_item()?);
            self.skip_newlines();
        }

        self.expect(&TokenKind::RightBrace)?;
        self.skip_newlines();

        Ok(LayoutRuleNode {
            aspect,
            items,
            span,
        })
    }

    /// Parse a component: `component("Name", prop: String = "val") { ... }`
    fn parse_component(&mut self) -> Result<ComponentNode, VidraError> {
        let span = self.current_span();
        self.expect(&TokenKind::Component)?;
        self.expect(&TokenKind::LeftParen)?;
        self.skip_newlines();

        let name = self.parse_string()?;
        self.skip_newlines();

        let mut props = Vec::new();
        let mut version = None;
        
        while self.peek() == &TokenKind::Comma {
            self.advance();
            self.skip_newlines();
            
            // Reached parenthesis even after a trailing comma?
            if self.peek() == &TokenKind::RightParen {
                break;
            }

            let prop_span = self.current_span();
            let prop_name = self.parse_identifier()?;
            self.skip_newlines();
            
            self.expect(&TokenKind::Colon)?;
            self.skip_newlines();
            
            if prop_name == "version" {
                version = Some(self.parse_string()?);
                self.skip_newlines();
                continue;
            }

            let type_name = self.parse_identifier()?;
            self.skip_newlines();

            let mut default_value = None;
            if self.peek() == &TokenKind::Equals {
                self.advance();
                self.skip_newlines();
                default_value = Some(self.parse_value()?);
                self.skip_newlines();
            }

            props.push(ComponentPropDef {
                name: prop_name,
                type_name,
                default_value,
                span: prop_span,
            });
        }

        self.expect(&TokenKind::RightParen)?;
        self.skip_newlines();
        self.expect(&TokenKind::LeftBrace)?;
        self.skip_newlines();

        let mut items = Vec::new();
        let mut variants = Vec::new();
        while self.peek() != &TokenKind::RightBrace && self.peek() != &TokenKind::Eof {
            self.skip_newlines();
            if self.peek() == &TokenKind::RightBrace {
                break;
            }
            if self.peek() == &TokenKind::Variant {
                variants.push(self.parse_variant()?);
            } else {
                items.push(self.parse_layer_block_item()?);
            }
            self.skip_newlines();
        }

        self.expect(&TokenKind::RightBrace)?;

        Ok(ComponentNode {
            name,
            props,
            version,
            items,
            variants,
            span,
        })
    }

    /// Parse a variant: `variant("dark", color: #333, text: #FFF)`
    fn parse_variant(&mut self) -> Result<VariantNode, VidraError> {
        let span = self.current_span();
        self.expect(&TokenKind::Variant)?;
        self.expect(&TokenKind::LeftParen)?;
        self.skip_newlines();
        let name = self.parse_string()?;
        self.skip_newlines();
        
        let overrides = if self.peek() == &TokenKind::Comma {
            self.advance();
            self.skip_newlines();
            self.parse_named_args_list()?
        } else {
            Vec::new()
        };
        
        self.expect(&TokenKind::RightParen)?;
        self.skip_newlines();
        
        Ok(VariantNode {
            name,
            overrides,
            span,
        })
    }

    /// Parse: `scene("name", duration) { ... }`
    fn parse_scene(&mut self) -> Result<SceneNode, VidraError> {
        let span = self.current_span();
        self.expect(&TokenKind::Scene)?;
        self.expect(&TokenKind::LeftParen)?;
        self.skip_newlines();

        let name = self.parse_string()?;
        self.skip_newlines();
        self.expect(&TokenKind::Comma)?;
        self.skip_newlines();

        let duration = self.parse_value()?;
        self.skip_newlines();

        self.expect(&TokenKind::RightParen)?;
        self.skip_newlines();
        self.expect(&TokenKind::LeftBrace)?;
        self.skip_newlines();

        let mut items = Vec::new();
        while self.peek() != &TokenKind::RightBrace && self.peek() != &TokenKind::Eof {
            self.skip_newlines();
            if self.peek() == &TokenKind::RightBrace {
                break;
            }
            items.push(self.parse_layer_block_item()?);
            self.skip_newlines();
        }

        self.expect(&TokenKind::RightBrace)?;

        Ok(SceneNode {
            name,
            duration,
            items,
            span,
        })
    }

    fn parse_layer_block_item(&mut self) -> Result<LayerBlockItem, VidraError> {
        if self.peek() == &TokenKind::If {
            self.parse_if_item()
        } else if self.peek() == &TokenKind::Transition {
            self.parse_transition_item()
        } else if self.peek() == &TokenKind::Identifier("animate".to_string()) {
            self.parse_animate_stagger_item()
        } else {
            Ok(LayerBlockItem::Layer(self.parse_layer()?))
        }
    }

    fn parse_transition_item(&mut self) -> Result<LayerBlockItem, VidraError> {
        let span = self.current_span();
        self.expect(&TokenKind::Transition)?;
        self.expect(&TokenKind::LeftParen)?;
        self.skip_newlines();

        let transition_type = self.parse_string()?;
        self.skip_newlines();

        self.expect(&TokenKind::Comma)?;
        self.skip_newlines();

        let duration = self.parse_value()?;
        self.skip_newlines();

        let mut easing = None;
        if self.peek() == &TokenKind::Comma {
            self.advance();
            self.skip_newlines();
            let args = self.parse_named_args_list()?;
            for arg in args {
                if arg.name == "ease" {
                    if let ValueNode::String(s) = arg.value {
                        easing = Some(s);
                    }
                }
            }
        }

        self.expect(&TokenKind::RightParen)?;

        Ok(LayerBlockItem::Transition {
            transition_type,
            duration,
            easing,
            span,
        })
    }

    fn parse_animate_stagger_item(&mut self) -> Result<LayerBlockItem, VidraError> {
        let span = self.current_span();
        self.advance(); // consume `animate`
        self.expect(&TokenKind::Dot)?;
        let ident = self.parse_identifier()?;
        if ident != "stagger" {
            let s = self.current_span();
            return Err(VidraError::parse(
                format!("expected 'stagger' after 'animate.', got {}", ident),
                &self.file,
                s.line,
                s.column,
            ));
        }

        self.expect(&TokenKind::LeftParen)?;
        self.skip_newlines();
        let args = self.parse_named_args_list()?;
        self.expect(&TokenKind::RightParen)?;
        self.skip_newlines();

        self.expect(&TokenKind::LeftBrace)?;
        self.skip_newlines();

        let mut animations = Vec::new();
        while self.peek() != &TokenKind::RightBrace && self.peek() != &TokenKind::Eof {
            self.skip_newlines();
            if self.peek() == &TokenKind::RightBrace {
                break;
            }
            animations.push(self.parse_property()?);
            self.skip_newlines();
        }

        self.expect(&TokenKind::RightBrace)?;

        Ok(LayerBlockItem::AnimationStagger {
            args,
            animations,
            span,
        })
    }

    fn parse_if_item(&mut self) -> Result<LayerBlockItem, VidraError> {
        let span = self.current_span();
        self.expect(&TokenKind::If)?;
        self.expect(&TokenKind::LeftParen)?;
        self.skip_newlines();
        let condition = self.parse_value()?;
        self.skip_newlines();
        self.expect(&TokenKind::RightParen)?;
        self.skip_newlines();

        self.expect(&TokenKind::LeftBrace)?;
        self.skip_newlines();

        let mut then_branch = Vec::new();
        while self.peek() != &TokenKind::RightBrace && self.peek() != &TokenKind::Eof {
            self.skip_newlines();
            if self.peek() == &TokenKind::RightBrace { break; }
            then_branch.push(self.parse_layer_block_item()?);
            self.skip_newlines();
        }
        self.expect(&TokenKind::RightBrace)?;
        self.skip_newlines();

        let mut else_branch = None;
        if self.peek() == &TokenKind::Else {
            self.advance();
            self.skip_newlines();
            
            // Handle `else if` transparently as wrapping recursion if needed? No, just keep it simple block `else { ... }` for now
            self.expect(&TokenKind::LeftBrace)?;
            self.skip_newlines();
            let mut branch = Vec::new();
            while self.peek() != &TokenKind::RightBrace && self.peek() != &TokenKind::Eof {
                self.skip_newlines();
                if self.peek() == &TokenKind::RightBrace { break; }
                branch.push(self.parse_layer_block_item()?);
                self.skip_newlines();
            }
            self.expect(&TokenKind::RightBrace)?;
            self.skip_newlines();
            else_branch = Some(branch);
        }

        Ok(LayerBlockItem::If {
            condition,
            then_branch,
            else_branch,
            span,
        })
    }

    /// Parse: `layer("name") { content properties* }`
    fn parse_layer(&mut self) -> Result<LayerNode, VidraError> {
        let span = self.current_span();
        self.expect(&TokenKind::Layer)?;
        self.expect(&TokenKind::LeftParen)?;
        let name = self.parse_string()?;
        self.expect(&TokenKind::RightParen)?;
        self.skip_newlines();
        self.expect(&TokenKind::LeftBrace)?;
        self.skip_newlines();

        // Parse layer content (first statement inside the block)
        let mut content = LayerContentNode::Empty;
        
        let is_content = match self.peek() {
            TokenKind::Text | TokenKind::Image | TokenKind::Video | TokenKind::Audio | TokenKind::Solid | TokenKind::Shape | TokenKind::Shader | TokenKind::Slot | TokenKind::TTS | TokenKind::AutoCaption => true,
            TokenKind::Identifier(name) if name != "position" && name != "animation" && name != "size" && name != "scale" => true,
            _ => false,
        };

        if is_content {
            content = self.parse_layer_content()?;
            self.skip_newlines();
        }

        // Parse remaining properties
        let mut properties = Vec::new();
        let mut children = Vec::new();
        while self.peek() != &TokenKind::RightBrace && self.peek() != &TokenKind::Eof {
            self.skip_newlines();
            if self.peek() == &TokenKind::RightBrace {
                break;
            }
            // Check if it's a nested layer
            if self.peek() == &TokenKind::Layer || self.peek() == &TokenKind::If {
                children.push(self.parse_layer_block_item()?);
            } else {
                properties.push(self.parse_property()?);
            }
            self.skip_newlines();
        }

        self.expect(&TokenKind::RightBrace)?;

        Ok(LayerNode {
            name,
            content,
            properties,
            children,
            span,
        })
    }

    /// Parse layer content: `text(...)`, `image(...)`, `solid(...)`, etc.
    fn parse_layer_content(&mut self) -> Result<LayerContentNode, VidraError> {
        match self.peek().clone() {
            TokenKind::Text => {
                self.advance();
                self.expect(&TokenKind::LeftParen)?;
                let text = self.parse_value()?;
                let args = self.parse_trailing_named_args()?;
                self.expect(&TokenKind::RightParen)?;
                Ok(LayerContentNode::Text { text, args })
            }
            TokenKind::Image => {
                self.advance();
                self.expect(&TokenKind::LeftParen)?;
                let path = self.parse_value()?;
                let args = self.parse_trailing_named_args()?;
                self.expect(&TokenKind::RightParen)?;
                Ok(LayerContentNode::Image { path, args })
            }
            TokenKind::Video => {
                self.advance();
                self.expect(&TokenKind::LeftParen)?;
                let path = self.parse_value()?;
                let args = self.parse_trailing_named_args()?;
                self.expect(&TokenKind::RightParen)?;
                Ok(LayerContentNode::Video { path, args })
            }
            TokenKind::Audio => {
                self.advance();
                self.expect(&TokenKind::LeftParen)?;
                let path = self.parse_value()?;
                let args = self.parse_trailing_named_args()?;
                self.expect(&TokenKind::RightParen)?;
                Ok(LayerContentNode::Audio { path, args })
            }
            TokenKind::TTS => {
                self.advance();
                self.expect(&TokenKind::LeftParen)?;
                let text = self.parse_value()?;
                
                // Expect a comma, then the voice
                self.expect(&TokenKind::Comma)?;
                
                let voice = self.parse_value()?;
                let args = self.parse_trailing_named_args()?;
                self.expect(&TokenKind::RightParen)?;
                Ok(LayerContentNode::TTS { text, voice, args })
            }
            TokenKind::AutoCaption => {
                self.advance();
                self.expect(&TokenKind::LeftParen)?;
                let audio_source = self.parse_value()?;
                let args = self.parse_trailing_named_args()?;
                self.expect(&TokenKind::RightParen)?;
                Ok(LayerContentNode::AutoCaption { audio_source, args })
            }
            TokenKind::Solid => {
                self.advance();
                self.expect(&TokenKind::LeftParen)?;
                let color = self.parse_value()?;
                self.expect(&TokenKind::RightParen)?;
                Ok(LayerContentNode::Solid { color })
            }
            TokenKind::Slot => {
                self.advance();
                self.expect(&TokenKind::LeftParen)?;
                self.expect(&TokenKind::RightParen)?;
                Ok(LayerContentNode::Slot)
            }
            TokenKind::Shape => {
                self.advance();
                self.expect(&TokenKind::LeftParen)?;
                let shape_type = self.parse_value()?;
                let shape_type_str = match shape_type {
                    ValueNode::String(s) => s,
                    ValueNode::Identifier(s) => s,
                    _ => return Err(VidraError::parse(
                        format!("expected string or identifier for shape type"),
                        &self.file,
                        self.current_span().line,
                        self.current_span().column,
                    )),
                };
                let args = self.parse_trailing_named_args()?;
                self.expect(&TokenKind::RightParen)?;
                Ok(LayerContentNode::Shape { shape_type: shape_type_str, args })
            }
            TokenKind::Shader => {
                self.advance();
                self.expect(&TokenKind::LeftParen)?;
                let path = self.parse_value()?;
                let args = self.parse_trailing_named_args()?;
                self.expect(&TokenKind::RightParen)?;
                Ok(LayerContentNode::Shader { path, args })
            }
            TokenKind::Identifier(name) => {
                // E.g., CustomComponent(prop: "value")
                self.advance();
                self.expect(&TokenKind::LeftParen)?;
                // the first arg might be named if all are named, or we might just use parse_trailing_named_args if we assume all args to components are named
                // let's just parse them as named args directly.
                let mut args = Vec::new();
                if self.peek() != &TokenKind::RightParen {
                    args = self.parse_named_args_list()?;
                }
                self.expect(&TokenKind::RightParen)?;
                Ok(LayerContentNode::Component { name, args })
            }
            _ => {
                let span = self.current_span();
                Err(VidraError::parse(
                    format!(
                        "expected layer content (text, image, video, audio, solid, shape, component), got {}",
                        self.peek()
                    ),
                    &self.file,
                    span.line,
                    span.column,
                ))
            }
        }
    }

    /// Parse a property: `position(...)`, `animation(...)`, or generic function call.
    fn parse_property(&mut self) -> Result<PropertyNode, VidraError> {
        let span = self.current_span();
        match self.peek().clone() {
            TokenKind::Identifier(ref name) if name == "position" => {
                self.advance();
                self.expect(&TokenKind::LeftParen)?;
                let x = self.parse_value()?;
                self.expect(&TokenKind::Comma)?;
                self.skip_newlines();
                let y = self.parse_value()?;
                self.expect(&TokenKind::RightParen)?;
                Ok(PropertyNode::Position { x, y, span })
            }
            TokenKind::Identifier(ref name) if name == "animate" => {
                self.advance(); // consume `animate`
                self.expect(&TokenKind::Dot)?;
                let ident = self.parse_identifier()?;
                if ident == "group" {
                    self.skip_newlines();
                    self.expect(&TokenKind::LeftBrace)?;
                    self.skip_newlines();
                    let mut animations = Vec::new();
                    while self.peek() != &TokenKind::RightBrace && self.peek() != &TokenKind::Eof {
                        self.skip_newlines();
                        if self.peek() == &TokenKind::RightBrace { break; }
                        animations.push(self.parse_property()?);
                        self.skip_newlines();
                    }
                    self.expect(&TokenKind::RightBrace)?;
                    Ok(PropertyNode::AnimationGroup { animations, span })
                } else if ident == "sequence" {
                    self.skip_newlines();
                    self.expect(&TokenKind::LeftBrace)?;
                    self.skip_newlines();
                    let mut animations = Vec::new();
                    while self.peek() != &TokenKind::RightBrace && self.peek() != &TokenKind::Eof {
                        self.skip_newlines();
                        if self.peek() == &TokenKind::RightBrace { break; }
                        animations.push(self.parse_property()?);
                        self.skip_newlines();
                    }
                    self.expect(&TokenKind::RightBrace)?;
                    Ok(PropertyNode::AnimationSequence { animations, span })
                } else {
                    Err(VidraError::parse(
                        format!("expected 'group' or 'sequence' after 'animate.', got {}", ident),
                        &self.file, span.line, span.column
                    ))
                }
            }
            TokenKind::Identifier(ref name) if name == "wait" => {
                self.advance();
                self.expect(&TokenKind::LeftParen)?;
                let duration = self.parse_value()?;
                self.expect(&TokenKind::RightParen)?;
                Ok(PropertyNode::Wait { duration, span })
            }
            TokenKind::Animation => {
                self.advance();
                self.expect(&TokenKind::LeftParen)?;
                let property = self.parse_identifier()?;
                let args = self.parse_trailing_named_args()?;
                self.expect(&TokenKind::RightParen)?;
                Ok(PropertyNode::Animation {
                    property,
                    args,
                    span,
                })
            }
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                self.expect(&TokenKind::LeftParen)?;
                let mut args = Vec::new();
                let mut named_args = Vec::new();

                while self.peek() != &TokenKind::RightParen && self.peek() != &TokenKind::Eof {
                    self.skip_newlines();
                    // Try named arg first
                    if let Some(na) = self.try_parse_named_arg()? {
                        named_args.push(na);
                    } else {
                        args.push(self.parse_value()?);
                    }
                    self.skip_newlines();
                    if self.peek() == &TokenKind::Comma {
                        self.advance();
                    }
                    self.skip_newlines();
                }
                self.expect(&TokenKind::RightParen)?;
                Ok(PropertyNode::FunctionCall {
                    name,
                    args,
                    named_args,
                    span,
                })
            }
            _ => Err(VidraError::parse(
                format!("expected property or function call, got {}", self.peek()),
                &self.file,
                span.line,
                span.column,
            )),
        }
    }

    // --- Helper parsers ---

    fn parse_number(&mut self) -> Result<f64, VidraError> {
        match self.peek().clone() {
            TokenKind::NumberLiteral(n) => {
                self.advance();
                Ok(n)
            }
            _ => {
                let span = self.current_span();
                Err(VidraError::parse(
                    format!("expected number, got {}", self.peek()),
                    &self.file,
                    span.line,
                    span.column,
                ))
            }
        }
    }

    fn parse_string(&mut self) -> Result<String, VidraError> {
        match self.peek().clone() {
            TokenKind::StringLiteral(s) => {
                self.advance();
                Ok(s)
            }
            _ => {
                let span = self.current_span();
                Err(VidraError::parse(
                    format!("expected string, got {}", self.peek()),
                    &self.file,
                    span.line,
                    span.column,
                ))
            }
        }
    }

    /// Parse an identifier. Also accepts certain keywords that can act as
    /// identifiers in named-argument positions (e.g. `from:`, `to:`).
    fn parse_identifier(&mut self) -> Result<String, VidraError> {
        match self.peek().clone() {
            TokenKind::Identifier(s) => {
                self.advance();
                Ok(s)
            }
            // Keywords that can be used as named-arg keys
            TokenKind::From => {
                self.advance();
                Ok("from".to_string())
            }
            TokenKind::Text => {
                self.advance();
                Ok("text".to_string())
            }
            TokenKind::Image => {
                self.advance();
                Ok("image".to_string())
            }
            TokenKind::Video => {
                self.advance();
                Ok("video".to_string())
            }
            TokenKind::Scene => {
                self.advance();
                Ok("scene".to_string())
            }
            TokenKind::Layer => {
                self.advance();
                Ok("layer".to_string())
            }
            TokenKind::Variant => {
                self.advance();
                Ok("variant".to_string())
            }
            TokenKind::Component => {
                self.advance();
                Ok("component".to_string())
            }
            _ => {
                let span = self.current_span();
                Err(VidraError::parse(
                    format!("expected identifier, got {}", self.peek()),
                    &self.file,
                    span.line,
                    span.column,
                ))
            }
        }
    }

    fn parse_value(&mut self) -> Result<ValueNode, VidraError> {
        let _span = self.current_span();
        if self.peek() == &TokenKind::LeftBracket {
            self.advance();
            let mut items = Vec::new();
            while self.peek() != &TokenKind::RightBracket && self.peek() != &TokenKind::Eof {
                self.skip_newlines();
                if self.peek() == &TokenKind::RightBracket { break; }
                items.push(self.parse_value()?);
                self.skip_newlines();
                if self.peek() == &TokenKind::Comma { self.advance(); }
                self.skip_newlines();
            }
            self.expect(&TokenKind::RightBracket)?;
            return Ok(ValueNode::Array(items));
        }

        match self.peek().clone() {
            TokenKind::StringLiteral(s) => {
                self.advance();
                Ok(ValueNode::String(s))
            }
            TokenKind::NumberLiteral(n) => {
                self.advance();
                Ok(ValueNode::Number(n))
            }
            TokenKind::DurationLiteral(d) => {
                self.advance();
                Ok(ValueNode::Duration(d))
            }
            TokenKind::ColorLiteral(c) => {
                self.advance();
                Ok(ValueNode::Color(c))
            }
            TokenKind::Identifier(s) => {
                self.advance();
                Ok(ValueNode::Identifier(s))
            }
            TokenKind::At => {
                self.advance(); // consume '@'
                match self.peek() {
                    TokenKind::Identifier(b) if b == "brand" => {
                        self.advance(); // consume 'brand'
                        self.expect(&TokenKind::Dot)?;
                        match self.peek().clone() {
                            TokenKind::Identifier(key) => {
                                self.advance(); // consume key
                                Ok(ValueNode::BrandReference(key))
                            }
                            _ => {
                                let span = self.current_span();
                                Err(VidraError::parse(
                                    "expected identifier after '@brand.'",
                                    &self.file,
                                    span.line,
                                    span.column,
                                ))
                            }
                        }
                    }
                    _ => {
                        let span = self.current_span();
                        Err(VidraError::parse(
                            "expected 'brand' after '@'",
                            &self.file,
                            span.line,
                            span.column,
                        ))
                    }
                }
            }
            _ => {
                let span = self.current_span();
                Err(VidraError::parse(
                    format!("expected value, got {}", self.peek()),
                    &self.file,
                    span.line,
                    span.column,
                ))
            }
        }
    }

    /// Parse trailing named args: `, name: value, name: value`
    fn parse_trailing_named_args(&mut self) -> Result<Vec<NamedArg>, VidraError> {
        let mut args = Vec::new();
        while self.peek() == &TokenKind::Comma {
            self.advance();
            self.skip_newlines();
            if self.peek() == &TokenKind::RightParen {
                break;
            }
            let span = self.current_span();
            let name = self.parse_identifier()?;
            self.expect(&TokenKind::Colon)?;
            self.skip_newlines();
            let value = self.parse_value()?;
            args.push(NamedArg { name, value, span });
        }
        Ok(args)
    }

    /// Parse a comma-separated list of named args, starting WITHOUT a leading comma
    fn parse_named_args_list(&mut self) -> Result<Vec<NamedArg>, VidraError> {
        let mut args = Vec::new();
        loop {
            self.skip_newlines();
            if self.peek() == &TokenKind::RightParen {
                break;
            }
            
            let span = self.current_span();
            let name = self.parse_identifier()?;
            self.skip_newlines();
            self.expect(&TokenKind::Colon)?;
            self.skip_newlines();
            let value = self.parse_value()?;
            args.push(NamedArg { name, value, span });
            
            self.skip_newlines();
            if self.peek() == &TokenKind::Comma {
                self.advance();
            } else {
                break; // must be RightParen handled next loop, or error
            }
        }
        Ok(args)
    }

    /// Try to parse a named arg. Returns None if the next tokens don't form `name: value`.
    fn try_parse_named_arg(&mut self) -> Result<Option<NamedArg>, VidraError> {
        // Check if the current token could be a named arg key
        let is_ident_like = matches!(
            self.peek(),
            TokenKind::Identifier(_)
                | TokenKind::From
                | TokenKind::Text
                | TokenKind::Image
                | TokenKind::Video
                | TokenKind::Scene
                | TokenKind::Layer
                | TokenKind::Variant
                | TokenKind::Component
        );
        if is_ident_like {
            let saved_pos = self.pos;
            let span = self.current_span();
            let name = self.parse_identifier()?;
            if self.peek() == &TokenKind::Colon {
                self.advance();
                self.skip_newlines();
                let value = self.parse_value()?;
                return Ok(Some(NamedArg { name, value, span }));
            } else {
                // Not a named arg, restore position
                self.pos = saved_pos;
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(src: &str) -> ProjectNode {
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens, "test.vidra");
        parser.parse().unwrap()
    }

    #[test]
    fn test_parse_minimal_project() {
        let project = parse(
            r#"
            project(1920, 1080, 30) {
                scene("intro", 5s) {
                    layer("bg") {
                        solid(#000000)
                    }
                }
            }
        "#,
        );

        assert_eq!(project.width, 1920);
        assert_eq!(project.height, 1080);
        assert!((project.fps - 30.0).abs() < 0.001);
        assert_eq!(project.scenes.len(), 1);
        assert_eq!(project.scenes[0].name, "intro");
        assert_eq!(project.scenes[0].duration, ValueNode::Duration(5.0));
        assert_eq!(project.scenes[0].items.len(), 1);
        if let LayerBlockItem::Layer(layer) = &project.scenes[0].items[0] {
            assert_eq!(layer.name, "bg");
        } else {
            panic!("Expected layer");
        }
    }

    #[test]
    fn test_parse_text_layer() {
        let project = parse(
            r#"
            project(1920, 1080, 30) {
                scene("s", 3s) {
                    layer("title") {
                        text("Hello Vidra", font: "Inter Bold", size: 72, color: #FFFFFF)
                    }
                }
            }
        "#,
        );

        let layer = if let LayerBlockItem::Layer(l) = &project.scenes[0].items[0] { l } else { panic!("Expected layer") };
        match &layer.content {
            LayerContentNode::Text { text, args } => {
                assert_eq!(text, &ValueNode::String("Hello Vidra".into()));
                assert_eq!(args.len(), 3);
                assert_eq!(args[0].name, "font");
                assert_eq!(args[1].name, "size");
                assert_eq!(args[2].name, "color");
            }
            _ => panic!("expected text content"),
        }
    }

    #[test]
    fn test_parse_with_position() {
        let project = parse(
            r#"
            project(1920, 1080, 30) {
                scene("s", 2s) {
                    layer("title") {
                        text("Hi")
                        position(100, 200)
                    }
                }
            }
        "#,
        );

        let layer = if let LayerBlockItem::Layer(l) = &project.scenes[0].items[0] { l } else { panic!("Expected layer") };
        assert_eq!(layer.properties.len(), 1);
        match &layer.properties[0] {
            PropertyNode::Position { x, y: _, .. } => {
                if let ValueNode::Number(xv) = x {
                    assert!((xv - 100.0).abs() < 0.001);
                } else {
                    panic!("expected number for x");
                }
            }
            _ => panic!("expected position property"),
        }
    }

    #[test]
    fn test_parse_with_animation() {
        let project = parse(
            r#"
            project(1920, 1080, 30) {
                scene("s", 5s) {
                    layer("bg") {
                        solid(#0000FF)
                        animation(opacity, from: 0, to: 1, duration: 2s)
                    }
                }
            }
        "#,
        );

        let layer = if let LayerBlockItem::Layer(l) = &project.scenes[0].items[0] { l } else { panic!("Expected layer") };
        assert_eq!(layer.properties.len(), 1);
        match &layer.properties[0] {
            PropertyNode::Animation { property, args, .. } => {
                assert_eq!(property, "opacity");
                assert_eq!(args.len(), 3); // from, to, duration
            }
            _ => panic!("expected animation property"),
        }
    }

    #[test]
    fn test_parse_multiple_scenes() {
        let project = parse(
            r#"
            project(1920, 1080, 30) {
                scene("intro", 5s) {
                    layer("bg") { solid(#000000) }
                }
                scene("main", 10s) {
                    layer("bg") { solid(#FFFFFF) }
                }
            }
        "#,
        );
        assert_eq!(project.scenes.len(), 2);
        assert_eq!(project.scenes[0].name, "intro");
        assert_eq!(project.scenes[1].name, "main");
    }

    #[test]
    fn test_parse_assets() {
        let project = parse(
            r#"
            project(1920, 1080, 30) {
                asset(font, "Roboto", "./fonts/Roboto.ttf")
                asset(image, "logo", "./img/logo.png")
                
                scene("main", 5s) {
                    layer("bg") { solid(#FFFFFF) }
                }
            }
        "#,
        );
        assert_eq!(project.assets.len(), 2);
        
        let a1 = &project.assets[0];
        assert_eq!(a1.asset_type, "font");
        assert_eq!(a1.id, "Roboto");
        assert_eq!(a1.path, "./fonts/Roboto.ttf");

        let a2 = &project.assets[1];
        assert_eq!(a2.asset_type, "image");
        assert_eq!(a2.id, "logo");
        assert_eq!(a2.path, "./img/logo.png");
    }

    #[test]
    fn test_parse_error_missing_brace() {
        let src = r#"project(1920, 1080, 30) { scene("s", 1s)"#;
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens, "test.vidra");
        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_layout_rules() {
        let code = r#"
        project(1920, 1080, 30) {
            layout rules {
                when aspect(16:9) {
                    layer("title") { position(100, 200) }
                }
                when aspect(9:16) {
                    layer("title") { position(200, 400) }
                }
            }
            scene("main", 5s) {}
        }
        "#;
        let project = parse(code);

        assert_eq!(project.layout_rules.len(), 1);
        let rules_node = &project.layout_rules[0];
        assert_eq!(rules_node.rules.len(), 2);
        
        assert_eq!(rules_node.rules[0].aspect, "16:9");
        assert_eq!(rules_node.rules[0].items.len(), 1);
        
        assert_eq!(rules_node.rules[1].aspect, "9:16");
        assert_eq!(rules_node.rules[1].items.len(), 1);
    }
}
