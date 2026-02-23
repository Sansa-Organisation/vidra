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
        "0ee1a20bb1da2e181859372edd8566cf339f8144269ab6e328d6862cdf30f59f"
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
        "aa72688ff62828f6653b35215f5b0e38012102b8cfe5bd31db331211219be86e"
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
        "9eae250a82e0316571bc13e0daf399611aa4fc4a68588218a6d58ece858e2846"
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
        "f837edee1304af6d04201790e7ebddea0b06ba8d7cfc28aa35ddbf2e0db58844"
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
        "27255f724568ceebaa11f77ce60b40ee9ba630854fa63b9ec66d9f99783ba854"
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
        "1d6aa454c82a4f2e6c776f18904ceb969134e64bcc159d78b94292c5a7b3446e"
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
        "c608604e8ab0c9c054023d3e74c88d972fa3c324ae88a2658654d816088b52af"
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
        "e5683a38ab4ee14563c97bd8df7da49d3ba9dd6d47d44fc969a32d2dd8260cd1"
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
        "564db51aec9ee2530d0ed24541c62f541c5fa9db6258460e3b2497e5a8fb7567"
    );
}

#[test]
fn test_conformance_11_transitions() {
    let mut proj = create_base_project();
    
    let mut scene1 = Scene::new(SceneId::new("s1"), Duration::from_seconds(0.5));
    scene1.add_layer(Layer::new(
        LayerId::new("bg1"),
        LayerContent::Solid { color: Color::RED },
    ));
    proj.add_scene(scene1);

    let mut scene2 = Scene::new(SceneId::new("s2"), Duration::from_seconds(0.5));
    scene2.add_layer(Layer::new(
        LayerId::new("bg2"),
        LayerContent::Solid { color: Color::BLUE },
    ));
    scene2.transition = Some(vidra_ir::transition::Transition {
        effect: vidra_ir::transition::TransitionType::Wipe { direction: "right".to_string() },
        duration: Duration::from_seconds(0.5),
        easing: vidra_core::types::Easing::Linear,
    });
    proj.add_scene(scene2);

    let hash = run_conformance_test(&proj);
    assert_eq!(
        hash,
        "b1ddd99a4b6c0b195de2de751d04c9e8d02e364b292976726286a7900d884e1a"
    );
}
