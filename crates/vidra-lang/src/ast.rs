//! VidraScript Abstract Syntax Tree (AST).

use crate::lexer::Span;

/// Top-level AST node: a project definition.
#[derive(Debug, Clone)]
pub struct ProjectNode {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub imports: Vec<ImportNode>,
    pub assets: Vec<AssetNode>,
    pub layout_rules: Vec<LayoutRulesNode>,
    pub variables: Vec<VarDefNode>,
    pub scenes: Vec<SceneNode>,
    pub components: Vec<ComponentNode>,
    pub span: Span,
}

/// An import definition.
#[derive(Debug, Clone)]
pub struct ImportNode {
    /// Relative or absolute path to the module.
    pub path: String,
    pub span: Span,
}

/// A variable definition (`@var name = value`)
#[derive(Debug, Clone)]
pub struct VarDefNode {
    pub name: String,
    pub value: ValueNode,
    pub span: Span,
}

/// An asset definition.
#[derive(Debug, Clone)]
pub struct AssetNode {
    /// The type of asset (Image, Video, Audio, Font)
    pub asset_type: String,
    /// The ID / reference name used in the project
    pub id: String,
    /// The path or url to the asset
    pub path: String,
    pub span: Span,
}

/// A scene definition.
#[derive(Debug, Clone)]
pub struct SceneNode {
    /// Scene name/ID.
    pub name: String,
    /// Duration in seconds or logic evaluating to seconds.
    pub duration: ValueNode,
    /// Items (layers and logic) in the scene.
    pub items: Vec<LayerBlockItem>,
    pub span: Span,
}

/// A reusable component definition.
#[derive(Debug, Clone)]
pub struct ComponentNode {
    /// Component name.
    pub name: String,
    /// Properties it accepts.
    pub props: Vec<ComponentPropDef>,
    /// Optional semantic version string.
    pub version: Option<String>,
    /// The template of items inside the component.
    pub items: Vec<LayerBlockItem>,
    pub variants: Vec<VariantNode>,
    pub span: Span,
}

/// A component variant definition.
#[derive(Debug, Clone)]
pub struct VariantNode {
    pub name: String,
    pub overrides: Vec<NamedArg>,
    pub span: Span,
}

/// A component property definition `prop name: Type = Default`.
#[derive(Debug, Clone)]
pub struct ComponentPropDef {
    pub name: String,
    pub type_name: String,
    pub default_value: Option<ValueNode>,
    pub span: Span,
}

/// A block of layout rules
#[derive(Debug, Clone)]
pub struct LayoutRulesNode {
    pub rules: Vec<LayoutRuleNode>,
    pub span: Span,
}

/// A layout rule definition (`when aspect(...) { ... }`)
#[derive(Debug, Clone)]
pub struct LayoutRuleNode {
    pub aspect: String,
    pub items: Vec<LayerBlockItem>,
    pub span: Span,
}

/// A layer definition.
#[derive(Debug, Clone)]
pub struct LayerNode {
    /// Layer name/ID.
    pub name: String,
    /// Layer content.
    pub content: LayerContentNode,
    /// Nested properties and animations.
    pub properties: Vec<PropertyNode>,
    /// Child layers (nested).
    pub children: Vec<LayerBlockItem>,
    pub span: Span,
}

/// An item inside a layer block (could be a layer, or some control flow)
#[derive(Debug, Clone)]
pub enum LayerBlockItem {
    Layer(LayerNode),
    If {
        condition: ValueNode,
        then_branch: Vec<LayerBlockItem>,
        else_branch: Option<Vec<LayerBlockItem>>,
        span: Span,
    },
    Transition {
        transition_type: String,
        duration: ValueNode,
        easing: Option<String>,
        span: Span,
    },
    AnimationStagger {
        args: Vec<NamedArg>,
        animations: Vec<PropertyNode>,
        span: Span,
    },
    ComponentUse {
        name: String,
        args: Vec<NamedArg>,
        span: Span,
    }
}

/// The content of a layer.
#[derive(Debug, Clone)]
pub enum LayerContentNode {
    Text {
        text: ValueNode,
        args: Vec<NamedArg>,
    },
    Image {
        path: ValueNode,
        args: Vec<NamedArg>,
    },
    Video {
        path: ValueNode,
        args: Vec<NamedArg>,
    },
    Audio {
        path: ValueNode,
        args: Vec<NamedArg>,
    },
    AutoCaption {
        audio_source: ValueNode,
        args: Vec<NamedArg>,
    },
    TTS {
        text: ValueNode,
        voice: ValueNode,
        args: Vec<NamedArg>,
    },
    Solid {
        color: ValueNode,
    },
    Shape {
        shape_type: String,
        args: Vec<NamedArg>,
    },
    /// A custom component instance
    Component {
        name: String,
        args: Vec<NamedArg>,
    },
    /// A slot for children within a component
    Slot,
    /// Empty content
    Empty,
}

/// A named argument (e.g., `font: "Inter"`, `size: 48`).
#[derive(Debug, Clone)]
pub struct NamedArg {
    pub name: String,
    pub value: ValueNode,
    pub span: Span,
}

/// A value in the AST.
#[derive(Debug, Clone, PartialEq)]
pub enum ValueNode {
    String(String),
    Number(f64),
    Duration(f64), // in seconds
    Color(String), // hex
    Identifier(String),
    BrandReference(String), // `@brand.key`
    Array(Vec<ValueNode>),
}

/// A property assignment or animation on a layer.
#[derive(Debug, Clone)]
pub enum PropertyNode {
    /// `position(x, y)`
    Position {
        x: ValueNode,
        y: ValueNode,
        span: Span,
    },
    /// `animation(property, from: val, to: val, ...)`
    Animation {
        property: String,
        args: Vec<NamedArg>,
        span: Span,
    },
    /// Generic function call (for enter/exit/etc.)
    FunctionCall {
        name: String,
        args: Vec<ValueNode>,
        named_args: Vec<NamedArg>,
        span: Span,
    },
    /// `animate.group { ... }`
    AnimationGroup {
        animations: Vec<PropertyNode>,
        span: Span,
    },
    /// `animate.sequence { ... }`
    AnimationSequence {
        animations: Vec<PropertyNode>,
        span: Span,
    },
    /// `wait(duration)`
    Wait {
        duration: ValueNode,
        span: Span,
    },
}
