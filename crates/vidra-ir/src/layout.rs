use serde::{Deserialize, Serialize};

/// A layout constraint that positions a layer relative to the viewport or another layer.
/// Constraints are resolved at render-time by the layout solver, allowing the same
/// scene to adapt to different aspect ratios without manual repositioning.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LayoutConstraint {
    /// Center the layer along one or both axes.
    /// `center(horizontal)`, `center(vertical)`, `center(both)`
    Center(CenterAxis),

    /// Pin a layer edge to the viewport edge with an optional margin.
    /// `pin(top, 20)`, `pin(left)`, `pin(bottom, 40)`
    Pin { edge: Edge, margin: f64 },

    /// Position this layer below another layer with optional spacing.
    /// `below("title", 10)`
    Below { anchor_layer: String, spacing: f64 },

    /// Position this layer above another layer with optional spacing.
    /// `above("subtitle", 10)`
    Above { anchor_layer: String, spacing: f64 },

    /// Position this layer to the right of another layer with optional spacing.
    /// `rightOf("logo", 20)`
    RightOf { anchor_layer: String, spacing: f64 },

    /// Position this layer to the left of another layer with optional spacing.
    /// `leftOf("logo", 20)`
    LeftOf { anchor_layer: String, spacing: f64 },

    /// Fill the available width/height of the viewport, with optional padding.
    /// `fill(horizontal, 40)` — stretch to viewport width minus 40px on each side.
    Fill { axis: FillAxis, padding: f64 },

    /// Set an explicit size (width, height). Overrides content-intrinsic sizing.
    /// `size(400, 300)`
    Size { width: f64, height: f64 },
}

/// Axis for centering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CenterAxis {
    Horizontal,
    Vertical,
    Both,
}

/// An edge of the viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Edge {
    Top,
    Bottom,
    Left,
    Right,
}

/// Axis for fill constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FillAxis {
    Horizontal,
    Vertical,
    Both,
}

/// Resolved layout result for a single layer — concrete pixel coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolvedLayout {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// The layout solver. Resolves a set of layer constraints into concrete positions.
pub struct LayoutSolver;

impl LayoutSolver {
    /// Resolve constraints for all layers in a scene.
    ///
    /// # Arguments
    /// * `viewport_width` - The target viewport width
    /// * `viewport_height` - The target viewport height
    /// * `layers` - Slice of `(layer_id, intrinsic_width, intrinsic_height, constraints)` tuples
    ///
    /// # Returns
    /// A vector of `(layer_id, ResolvedLayout)` tuples with final pixel positions.
    pub fn solve(
        viewport_width: f64,
        viewport_height: f64,
        layers: &[(String, f64, f64, Vec<LayoutConstraint>)],
    ) -> Vec<(String, ResolvedLayout)> {
        // First pass: resolve Size and Fill constraints, then Pin and Center.
        // Second pass: resolve relational constraints (Below, Above, RightOf, LeftOf).
        let mut results: Vec<(String, ResolvedLayout)> = Vec::new();

        // Pass 1: absolute constraints
        for (layer_id, iw, ih, constraints) in layers {
            let mut layout = ResolvedLayout {
                x: 0.0,
                y: 0.0,
                width: *iw,
                height: *ih,
            };

            // Apply Size override first
            for c in constraints {
                if let LayoutConstraint::Size { width, height } = c {
                    layout.width = *width;
                    layout.height = *height;
                }
            }

            // Apply Fill
            for c in constraints {
                if let LayoutConstraint::Fill { axis, padding } = c {
                    match axis {
                        FillAxis::Horizontal => {
                            layout.width = viewport_width - padding * 2.0;
                            layout.x = *padding;
                        }
                        FillAxis::Vertical => {
                            layout.height = viewport_height - padding * 2.0;
                            layout.y = *padding;
                        }
                        FillAxis::Both => {
                            layout.width = viewport_width - padding * 2.0;
                            layout.height = viewport_height - padding * 2.0;
                            layout.x = *padding;
                            layout.y = *padding;
                        }
                    }
                }
            }

            // Apply Pin
            for c in constraints {
                if let LayoutConstraint::Pin { edge, margin } = c {
                    match edge {
                        Edge::Top => layout.y = *margin,
                        Edge::Bottom => layout.y = viewport_height - layout.height - margin,
                        Edge::Left => layout.x = *margin,
                        Edge::Right => layout.x = viewport_width - layout.width - margin,
                    }
                }
            }

            // Apply Center (overrides pin on that axis)
            for c in constraints {
                if let LayoutConstraint::Center(axis) = c {
                    match axis {
                        CenterAxis::Horizontal => {
                            layout.x = (viewport_width - layout.width) / 2.0;
                        }
                        CenterAxis::Vertical => {
                            layout.y = (viewport_height - layout.height) / 2.0;
                        }
                        CenterAxis::Both => {
                            layout.x = (viewport_width - layout.width) / 2.0;
                            layout.y = (viewport_height - layout.height) / 2.0;
                        }
                    }
                }
            }

            results.push((layer_id.clone(), layout));
        }

        // Pass 2: relational constraints (depend on other layers' resolved positions)
        // We iterate multiple times to resolve chains (A below B, C below A).
        for _pass in 0..3 {
            for i in 0..results.len() {
                let (_, constraints) = {
                    let (ref id, _, _, ref constraints) = layers[i];
                    (id.clone(), constraints.clone())
                };

                for c in &constraints {
                    match c {
                        LayoutConstraint::Below {
                            anchor_layer,
                            spacing,
                        } => {
                            if let Some(anchor) = results.iter().find(|(id, _)| id == anchor_layer)
                            {
                                let anchor_layout = anchor.1;
                                results[i].1.y = anchor_layout.y + anchor_layout.height + spacing;
                            }
                        }
                        LayoutConstraint::Above {
                            anchor_layer,
                            spacing,
                        } => {
                            if let Some(anchor) = results.iter().find(|(id, _)| id == anchor_layer)
                            {
                                let anchor_layout = anchor.1;
                                results[i].1.y = anchor_layout.y - results[i].1.height - spacing;
                            }
                        }
                        LayoutConstraint::RightOf {
                            anchor_layer,
                            spacing,
                        } => {
                            if let Some(anchor) = results.iter().find(|(id, _)| id == anchor_layer)
                            {
                                let anchor_layout = anchor.1;
                                results[i].1.x = anchor_layout.x + anchor_layout.width + spacing;
                            }
                        }
                        LayoutConstraint::LeftOf {
                            anchor_layer,
                            spacing,
                        } => {
                            if let Some(anchor) = results.iter().find(|(id, _)| id == anchor_layer)
                            {
                                let anchor_layout = anchor.1;
                                results[i].1.x = anchor_layout.x - results[i].1.width - spacing;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_center_both() {
        let layers = vec![(
            "title".to_string(),
            200.0,
            50.0,
            vec![LayoutConstraint::Center(CenterAxis::Both)],
        )];
        let results = LayoutSolver::solve(1920.0, 1080.0, &layers);
        assert_eq!(results.len(), 1);
        assert!((results[0].1.x - 860.0).abs() < 0.01); // (1920-200)/2
        assert!((results[0].1.y - 515.0).abs() < 0.01); // (1080-50)/2
    }

    #[test]
    fn test_pin_top_left() {
        let layers = vec![(
            "logo".to_string(),
            100.0,
            100.0,
            vec![
                LayoutConstraint::Pin {
                    edge: Edge::Top,
                    margin: 20.0,
                },
                LayoutConstraint::Pin {
                    edge: Edge::Left,
                    margin: 30.0,
                },
            ],
        )];
        let results = LayoutSolver::solve(1920.0, 1080.0, &layers);
        assert!((results[0].1.x - 30.0).abs() < 0.01);
        assert!((results[0].1.y - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_pin_bottom_right() {
        let layers = vec![(
            "cta".to_string(),
            200.0,
            60.0,
            vec![
                LayoutConstraint::Pin {
                    edge: Edge::Bottom,
                    margin: 40.0,
                },
                LayoutConstraint::Pin {
                    edge: Edge::Right,
                    margin: 50.0,
                },
            ],
        )];
        let results = LayoutSolver::solve(1920.0, 1080.0, &layers);
        assert!((results[0].1.x - (1920.0 - 200.0 - 50.0)).abs() < 0.01);
        assert!((results[0].1.y - (1080.0 - 60.0 - 40.0)).abs() < 0.01);
    }

    #[test]
    fn test_below_constraint() {
        let layers = vec![
            (
                "title".to_string(),
                400.0,
                50.0,
                vec![
                    LayoutConstraint::Center(CenterAxis::Horizontal),
                    LayoutConstraint::Pin {
                        edge: Edge::Top,
                        margin: 100.0,
                    },
                ],
            ),
            (
                "subtitle".to_string(),
                300.0,
                30.0,
                vec![
                    LayoutConstraint::Center(CenterAxis::Horizontal),
                    LayoutConstraint::Below {
                        anchor_layer: "title".to_string(),
                        spacing: 20.0,
                    },
                ],
            ),
        ];
        let results = LayoutSolver::solve(1920.0, 1080.0, &layers);
        // title: y=100, height=50, so subtitle should be at y = 100 + 50 + 20 = 170
        assert!((results[1].1.y - 170.0).abs() < 0.01);
    }

    #[test]
    fn test_fill_horizontal() {
        let layers = vec![(
            "bg".to_string(),
            100.0,
            100.0,
            vec![LayoutConstraint::Fill {
                axis: FillAxis::Horizontal,
                padding: 40.0,
            }],
        )];
        let results = LayoutSolver::solve(1920.0, 1080.0, &layers);
        assert!((results[0].1.width - (1920.0 - 80.0)).abs() < 0.01); // 1840
        assert!((results[0].1.x - 40.0).abs() < 0.01);
    }

    #[test]
    fn test_multi_aspect_adaptation() {
        // Same constraints, different viewports — demonstrate responsive behavior
        let constraints = vec![
            LayoutConstraint::Center(CenterAxis::Horizontal),
            LayoutConstraint::Pin {
                edge: Edge::Top,
                margin: 50.0,
            },
        ];

        // 16:9
        let r1 = LayoutSolver::solve(
            1920.0,
            1080.0,
            &[("title".to_string(), 400.0, 60.0, constraints.clone())],
        );
        assert!((r1[0].1.x - 760.0).abs() < 0.01); // (1920-400)/2

        // 9:16
        let r2 = LayoutSolver::solve(
            1080.0,
            1920.0,
            &[("title".to_string(), 400.0, 60.0, constraints.clone())],
        );
        assert!((r2[0].1.x - 340.0).abs() < 0.01); // (1080-400)/2

        // Both pinned at top with margin 50
        assert!((r1[0].1.y - 50.0).abs() < 0.01);
        assert!((r2[0].1.y - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_size_override() {
        let layers = vec![(
            "box".to_string(),
            100.0,
            100.0,
            vec![
                LayoutConstraint::Size {
                    width: 500.0,
                    height: 300.0,
                },
                LayoutConstraint::Center(CenterAxis::Both),
            ],
        )];
        let results = LayoutSolver::solve(1920.0, 1080.0, &layers);
        assert!((results[0].1.width - 500.0).abs() < 0.01);
        assert!((results[0].1.height - 300.0).abs() < 0.01);
        assert!((results[0].1.x - 710.0).abs() < 0.01); // (1920-500)/2
    }
}
