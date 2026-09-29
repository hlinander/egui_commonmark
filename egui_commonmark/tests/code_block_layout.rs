use egui::{Pos2, Rect, Shape, Vec2};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

const SQL: &str = "SELECT column_name, data_type, count(*)\nFROM information_schema.columns\nGROUP BY column_name, data_type;";

fn collect_text(shape: &Shape, output: &mut Vec<(String, Rect)>) {
    match shape {
        Shape::Text(text) => {
            let bounds = text
                .galley
                .rows
                .iter()
                .filter(|row| !row.glyphs.is_empty())
                .map(|row| row.rect().translate(text.pos.to_vec2()))
                .reduce(Rect::union)
                .unwrap_or(Rect::NOTHING);
            output.push((text.galley.text().trim_end().to_owned(), bounds));
        }
        Shape::Vec(shapes) => shapes.iter().for_each(|s| collect_text(s, output)),
        _ => {}
    }
}

fn render(markdown: &str, width: f32, nested: bool) -> (Rect, Vec<(String, Rect)>) {
    let ctx = egui::Context::default();
    let mut cache = CommonMarkCache::default();
    let mut available = Rect::NOTHING;
    let mut output = Vec::new();
    for _ in 0..2 {
        let frame = ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(width, 2000.0))),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let mut content = |ui: &mut egui::Ui| {
                        available = ui.available_rect_before_wrap();
                        let rendered = CommonMarkViewer::new().show(ui, &mut cache, markdown);
                        assert!(
                            rendered.response.rect.right() <= available.right() + 1.0,
                            "renderer exceeds container: {available:?} {:?}",
                            rendered.response.rect
                        );
                    };
                    if nested {
                        ui.indent("outer", |ui| {
                            ui.indent("inner", |ui| {
                                egui::Frame::new().inner_margin(8).show(ui, &mut content);
                            });
                        });
                    } else {
                        content(ui);
                    }
                });
            },
        );
        output.clear();
        for shape in frame.shapes {
            collect_text(&shape.shape, &mut output);
        }
    }
    (available, output)
}

#[test]
fn code_blocks_in_lists_have_separate_rows_and_fit_the_container() {
    for (prefix, indent) in [("1. ", "   "), ("- ", "  "), ("1. Outer\n   - ", "     ")] {
        for fenced in [true, false] {
            let code = if fenced {
                format!(
                    "{indent}```sql\n{indent}{}\n{indent}```",
                    SQL.replace('\n', &format!("\n{indent}"))
                )
            } else {
                format!(
                    "{indent}    {}",
                    SQL.replace('\n', &format!("\n{indent}    "))
                )
            };
            let markdown = format!("{prefix}Before code\n\n{code}\n\n{indent}After code\n");
            for width in [240.0, 480.0, 900.0] {
                for nested in [false, true] {
                    let (available, texts) = render(&markdown, width, nested);
                    let rect = |s: &str| texts.iter().find(|(text, _)| text == s).unwrap().1;
                    let before = rect("Before code");
                    let code = rect(SQL);
                    let after = rect("After code");
                    assert!(
                        code.top() >= before.bottom(),
                        "code starts beside prose: {width} {nested} {markdown:?}: {before:?} {code:?}"
                    );
                    assert!(
                        after.top() >= code.bottom(),
                        "following prose overlaps code: {markdown:?}"
                    );
                    assert!(
                        code.left() >= before.left() && code.left() < before.left() + 8.0,
                        "code indentation: {before:?} {code:?}"
                    );
                    assert!(
                        code.right() <= available.right() + 1.0,
                        "code exceeds container: {available:?} {code:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn inline_code_stays_in_its_paragraph() {
    let (_, texts) = render("Before `inline` after", 480.0, false);
    let top = |s: &str| {
        texts
            .iter()
            .find(|(text, _)| text.trim() == s)
            .unwrap()
            .1
            .top()
    };
    assert!((top("Before") - top("inline")).abs() < 4.0);
    assert!((top("after") - top("inline")).abs() < 4.0);
}

#[test]
fn tight_list_and_top_level_blocks_preserve_code() {
    for (prefix, indent) in [("", ""), ("1. ", "   "), ("- ", "  ")] {
        let markdown = format!(
            "{prefix}Before code\n{indent}```sql\n{indent}{}\n{indent}```\n{indent}After code\n",
            SQL.replace('\n', &format!("\n{indent}"))
        );
        for nested in [false, true] {
            let (_, texts) = render(&markdown, 320.0, nested);
            let rect = |s: &str| texts.iter().find(|(text, _)| text == s).unwrap().1;
            assert!(rect(SQL).top() >= rect("Before code").bottom());
            assert!(rect("After code").top() >= rect(SQL).bottom());
        }
    }
}
