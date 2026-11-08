use egui::{Pos2, Rect, Shape, Vec2};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

struct Geometry {
    available: Rect,
    border: Rect,
    separator: Rect,
    texts: Vec<(String, Rect)>,
    stripes: Vec<Rect>,
}

impl Default for Geometry {
    fn default() -> Self {
        Self {
            available: Rect::NOTHING,
            border: Rect::NOTHING,
            separator: Rect::NOTHING,
            texts: Vec::new(),
            stripes: Vec::new(),
        }
    }
}

impl Geometry {
    fn collect(&mut self, shape: &Shape) {
        match shape {
            Shape::Vec(shapes) => shapes.iter().for_each(|s| self.collect(s)),
            Shape::Rect(rect) if rect.stroke.width > 0.0 => self.border = rect.rect,
            Shape::Rect(rect) if rect.rect.height() < 100.0 => self.stripes.push(rect.rect),
            Shape::LineSegment { points, .. } if points[0].y == points[1].y => {
                self.separator = Rect::from_two_pos(points[0], points[1]);
            }
            Shape::Text(text) => self.texts.push((
                text.galley.text().trim().to_owned(),
                text.galley.rect.translate(text.pos.to_vec2()),
            )),
            _ => {}
        }
    }

    fn text(&self, value: &str) -> Rect {
        self.texts.iter().find(|(text, _)| text == value).unwrap().1
    }
}

fn render(
    ctx: &egui::Context,
    cache: &mut CommonMarkCache,
    markdown: &str,
    width: f32,
    fill: bool,
) -> Geometry {
    let mut geometry = Geometry::default();
    for _ in 0..2 {
        geometry = Geometry::default();
        let frame = ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(width, 2000.0))),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    geometry.available = ui.available_rect_before_wrap();
                    CommonMarkViewer::new()
                        .table_full_width(fill)
                        .show(ui, cache, markdown);
                });
            },
        );
        for shape in frame.shapes {
            geometry.collect(&shape.shape);
        }
    }
    geometry
}

fn close(actual: f32, expected: f32) {
    assert!((actual - expected).abs() <= 1.0, "{actual} != {expected}");
}

#[test]
fn outer_columns_meet_the_insets_and_interior_gaps_are_equal() {
    let ctx = egui::Context::default();
    let mut cache = CommonMarkCache::default();
    // Repeat the widest content in every row to expose the full column geometry.
    let markdown = "| Name | Description | Count |\n| --- | --- | --- |\n| Name | Description | Count |\n| Name | Description | Count |";
    for width in [240.0, 480.0, 900.0, 320.0, 900.0] {
        let g = render(&ctx, &mut cache, markdown, width, true);
        close(g.border.width(), g.available.width());
        let left_padding = g.text("Name").left() - g.separator.left();
        let right_padding = g.separator.right() - g.text("Count").right();
        close(left_padding, 8.0);
        close(left_padding, right_padding);
        close(
            g.text("Description").left() - g.text("Name").right(),
            g.text("Count").left() - g.text("Description").right(),
        );
        for (text, rect) in &g.texts {
            close(rect.left(), g.text(text).left());
        }
        let stripe = g.stripes.last().unwrap();
        close(stripe.left(), g.separator.left());
        close(stripe.right(), g.separator.right());
    }
}

#[test]
fn one_column_table_fills_width_and_aligns_left() {
    let g = render(
        &egui::Context::default(),
        &mut CommonMarkCache::default(),
        "| Value |\n| --- |\n| Value |",
        900.0,
        true,
    );
    close(g.border.width(), g.available.width());
    close(g.text("Value").left() - g.separator.left(), 8.0);
}

#[test]
fn constrained_tables_keep_base_padding_and_rows_do_not_overlap() {
    let ctx = egui::Context::default();
    let mut cache = CommonMarkCache::default();
    let markdown = "| Key | Detail |\n| --- | --- |\n| First | Several words that need wrapping across multiple lines in a narrow pane |\n| Second | More words for the following row |";
    for width in [200.0, 320.0, 440.0, 200.0] {
        let g = render(&ctx, &mut cache, markdown, width, true);
        close(g.border.width(), g.available.width());
        close(g.text("Key").left() - g.separator.left(), 8.0);
        let wrapped =
            g.text("Several words that need wrapping across multiple lines in a narrow pane");
        assert!(g.text("Second").top() >= wrapped.bottom());
        for (_, rect) in g.texts {
            assert!(rect.left() >= g.separator.left() + 7.5);
            assert!(rect.right() <= g.separator.right() - 7.5);
        }
    }
}

#[test]
fn default_tables_retain_content_width() {
    let ctx = egui::Context::default();
    let mut cache = CommonMarkCache::default();
    let markdown = "| A | B |\n| --- | --- |\n| A | B |";
    let narrow = render(&ctx, &mut cache, markdown, 320.0, false);
    let wide = render(&ctx, &mut cache, markdown, 900.0, false);
    close(narrow.border.width(), wide.border.width());
    assert!(wide.border.width() < wide.available.width() / 2.0);
}

#[test]
fn many_columns_fit_a_narrow_pane() {
    let g = render(
        &egui::Context::default(),
        &mut CommonMarkCache::default(),
        "| First | Second | Third | Fourth |\n| --- | --- | --- | --- |\n| 1 | 2 | 3 | 4 |",
        200.0,
        true,
    );
    close(g.border.width(), g.available.width());
    for (_, rect) in g.texts {
        assert!(rect.right() <= g.available.right());
    }
}

#[test]
fn last_column_moves_to_the_edge_without_changing_cell_text_alignment() {
    let g = render(
        &egui::Context::default(),
        &mut CommonMarkCache::default(),
        "| Name | Count |\n| --- | --- |\n| Item | 7 |",
        640.0,
        true,
    );
    close(g.separator.right() - g.text("Count").right(), 8.0);
    close(g.text("7").left(), g.text("Count").left());
    assert!(g.text("7").right() < g.text("Count").right());
}
