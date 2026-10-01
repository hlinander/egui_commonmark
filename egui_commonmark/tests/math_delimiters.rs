use egui_commonmark::{CommonMarkCache, CommonMarkViewer, MathDelimiters};
use std::cell::RefCell;

fn render(delimiters: MathDelimiters, enabled: bool, scrollable: bool) -> Vec<(bool, String)> {
    let ctx = egui::Context::default();
    let mut cache = CommonMarkCache::default();
    let formulas = RefCell::new(Vec::new());
    let render_math = |ui: &mut egui::Ui, source: &str, inline: bool| {
        formulas.borrow_mut().push((inline, source.to_owned()));
        ui.label(source);
    };
    // The second frame exercises the scrollable viewer's cached layout path.
    for _ in 0..2 {
        formulas.borrow_mut().clear();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let viewer = CommonMarkViewer::new()
                    .math_delimiters(delimiters)
                    .render_math_fn(enabled.then_some(&render_math));
                let source = r"$x$ \(y\) \[z\] $$w$$";
                if scrollable {
                    viewer.show_scrollable("math", ui, &mut cache, source);
                } else {
                    viewer.show(ui, &mut cache, source);
                }
            });
        });
    }
    formulas.into_inner()
}

#[test]
fn math_delimiters_apply_to_both_viewers() {
    for scrollable in [false, true] {
        assert_eq!(
            render(MathDelimiters::MathJax, true, scrollable),
            [(true, "y".into()), (false, "z".into()), (false, "w".into())]
        );
        assert_eq!(
            render(MathDelimiters::default(), true, scrollable),
            [(true, "x".into()), (false, "w".into())]
        );
        assert!(render(MathDelimiters::MathJax, false, scrollable).is_empty());
    }
}
