use vidra_core::types::ShapeType;
use vidra_core::{Color, Duration};
use vidra_ir::asset::{Asset, AssetId, AssetType};
use vidra_ir::layer::{Layer, LayerContent, LayerId};
use vidra_ir::project::{Project, ProjectSettings};
use vidra_ir::scene::{Scene, SceneId};
use vidra_render::RenderPipeline;

/// Helper to run a project and return its content hash hex string.
fn run_conformance_test(project: &Project) -> String {
    let result =
        RenderPipeline::render(project).expect("render should succeed in conformance test");
    result.content_hash().to_hex()
}

fn create_base_project() -> Project {
    let mut proj = Project::new(ProjectSettings::custom(320, 240, 30.0));
    // Provide some assets inside tests/assets
    proj.assets.register(Asset::new(
        AssetId::new("test_image"),
        AssetType::Image,
        "tests/assets/test_image.png",
    ));
    proj.assets.register(Asset::new(
        AssetId::new("test_video"),
        AssetType::Video,
        "tests/assets/test_video.mp4",
    ));
    proj
}

fn create_scene() -> Scene {
    Scene::new(SceneId::new("main"), Duration::from_seconds(0.5))
}

#[test]
fn test_conformance_01_solid_color() {
    let mut proj = create_base_project();
    let mut scene = create_scene();

    scene.add_layer(Layer::new(
        LayerId::new("bg"),
        LayerContent::Solid { color: Color::RED },
    ));

    proj.add_scene(scene);

    let hash = run_conformance_test(&proj);
    // Replace with expected hash after first run
    assert_eq!(
        hash, "c7c5873d19b7369633a68d93c2de8ca30cff670ec9d74271b0442e40c3a17d03",
        "conformance hash mismatch"
    );
}

#[test]
fn test_conformance_02_text_basic() {
    let mut proj = create_base_project();
    let mut scene = create_scene();

    scene.add_layer(
        Layer::new(
            LayerId::new("t1"),
            LayerContent::Text {
                text: "Conformance".into(),
                font_family: "Inter".into(),
                font_size: 48.0,
                color: Color::WHITE,
            },
        )
        .with_position(50.0, 100.0),
    );

    proj.add_scene(scene);
    let hash = run_conformance_test(&proj);
    assert_eq!(
        hash,
        "e6746fe99e68c874876f75a8c0c0d9c53091c1ed6f715d9bf9206c4dc69e5bd6"
    );
}

#[test]
fn test_conformance_03_text_multiline() {
    let mut proj = create_base_project();
    let mut scene = create_scene();

    scene.add_layer(
        Layer::new(
            LayerId::new("t2"),
            LayerContent::Text {
                text: "Line 1\nLine 2".into(),
                font_family: "Inter".into(),
                font_size: 24.0,
                color: Color::BLUE,
            },
        )
        .with_position(10.0, 50.0),
    );

    proj.add_scene(scene);
    let hash = run_conformance_test(&proj);
    assert_eq!(
        hash,
        "66641260ed38f612bd630df4073f5dc246d547e0cf3186d375bb49112fab4057"
    );
}

#[test]
fn test_conformance_04_shape_rect() {
    let mut proj = create_base_project();
    let mut scene = create_scene();

    scene.add_layer(
        Layer::new(
            LayerId::new("rect"),
            LayerContent::Shape {
                shape: ShapeType::Rect {
                    width: 100.0,
                    height: 80.0,
                    corner_radius: 0.0,
                },
                fill: Some(Color::GREEN),
                stroke: None,
                stroke_width: 0.0,
            },
        )
        .with_position(20.0, 20.0),
    );

    proj.add_scene(scene);
    let hash = run_conformance_test(&proj);
    assert_eq!(
        hash,
        "620cf9d91d18b5054b7a03fddf953ba7df4d8baf941028d41145f4aa2ee6ed86"
    );
}

#[test]
fn test_conformance_05_shape_circle() {
    let mut proj = create_base_project();
    let mut scene = create_scene();

    scene.add_layer(
        Layer::new(
            LayerId::new("circle"),
            LayerContent::Shape {
                shape: ShapeType::Circle { radius: 50.0 },
                fill: Some(Color::rgb(1.0, 0.0, 1.0)),
                stroke: None,
                stroke_width: 0.0,
            },
        )
        .with_position(100.0, 100.0),
    );

    proj.add_scene(scene);
    let hash = run_conformance_test(&proj);
    assert_eq!(
        hash,
        "3dcab9e040f084b89db9600d8b3ba3e2af5d903a3182e2a3d86e9595d6a8f7c3"
    );
}

#[test]
fn test_conformance_06_image() {
    let mut proj = create_base_project();
    let mut scene = create_scene();

    scene.add_layer(
        Layer::new(
            LayerId::new("img"),
            LayerContent::Image {
                asset_id: AssetId::new("test_image"),
            },
        )
        .with_position(10.0, 10.0),
    );

    proj.add_scene(scene);
    let hash = run_conformance_test(&proj);
    assert_eq!(
        hash,
        "15e2d2f3e5b9962787871594f71578388ab924a5523eb326c8db33ad60693e03"
    );
}

#[test]
fn test_conformance_07_video() {
    let mut proj = create_base_project();
    let mut scene = create_scene();

    scene.add_layer(
        Layer::new(
            LayerId::new("vid"),
            LayerContent::Video {
                asset_id: AssetId::new("test_video"),
                trim_start: Duration::from_seconds(0.0),
                trim_end: None,
            },
        )
        .with_position(50.0, 50.0),
    );

    proj.add_scene(scene);
    let hash = run_conformance_test(&proj);
    assert_eq!(
        hash,
        "01212691fe98b8d149d457225f35d8ec3f7f2b597cd930a7da2f9f6d6cddc68f"
    );
}

#[test]
fn test_conformance_08_opacity() {
    let mut proj = create_base_project();
    let mut scene = create_scene();

    scene.add_layer(Layer::new(
        LayerId::new("bg"),
        LayerContent::Solid {
            color: Color::WHITE,
        },
    ));

    scene.add_layer(
        Layer::new(
            LayerId::new("rect_half_trans"),
            LayerContent::Shape {
                shape: ShapeType::Rect {
                    width: 100.0,
                    height: 100.0,
                    corner_radius: 0.0,
                },
                fill: Some(Color::RED),
                stroke: None,
                stroke_width: 0.0,
            },
        )
        .with_opacity(0.5)
        .with_position(50.0, 50.0),
    );

    proj.add_scene(scene);
    let hash = run_conformance_test(&proj);
    assert_eq!(
        hash,
        "4a1210478daa6709cb4e46d6e98d849cbedcec91aebf4a71d83cd7268be734ea"
    );
}

#[test]
fn test_conformance_09_hierarchy() {
    let mut proj = create_base_project();
    let mut scene = create_scene();

    let mut parent = Layer::new(
        LayerId::new("parent"),
        LayerContent::Solid {
            color: Color::TRANSPARENT,
        },
    )
    .with_position(100.0, 100.0);

    let child = Layer::new(
        LayerId::new("child"),
        LayerContent::Shape {
            shape: ShapeType::Circle { radius: 20.0 },
            fill: Some(Color::BLUE),
            stroke: None,
            stroke_width: 0.0,
        },
    )
    .with_position(10.0, 10.0);

    parent.add_child(child);
    scene.add_layer(parent);

    proj.add_scene(scene);
    let hash = run_conformance_test(&proj);
    assert_eq!(
        hash,
        "3dd3e9e783d5b968c63e6ae8eaf90cffe27800aec3f2a6010bdf24f84dc83440"
    );
}

#[test]
fn test_conformance_10_animation_position() {
    let mut proj = create_base_project();
    let mut scene = create_scene();

    use vidra_core::types::Easing;
    use vidra_ir::animation::{AnimatableProperty, Animation};

    scene.add_layer(
        Layer::new(
            LayerId::new("rect"),
            LayerContent::Shape {
                shape: ShapeType::Rect {
                    width: 20.0,
                    height: 20.0,
                    corner_radius: 0.0,
                },
                fill: Some(Color::WHITE),
                stroke: None,
                stroke_width: 0.0,
            },
        )
        .with_animation(Animation::from_to(
            AnimatableProperty::PositionX,
            0.0,
            200.0,
            Duration::from_seconds(0.5),
            Easing::Linear,
        )),
    );

    proj.add_scene(scene);
    let hash = run_conformance_test(&proj);
    assert_eq!(
        hash,
        "6452b348833efa154554f952815a837ea805a0f529d0cf44c8695eb8b146c524"
    );
}
