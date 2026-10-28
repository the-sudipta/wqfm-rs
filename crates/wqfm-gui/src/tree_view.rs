//! Draws a species tree as a rectangular cladogram, leaves on the right.

use eframe::egui::{self, Align2, FontId, Pos2, Sense, Stroke, Vec2};
use wqfm::newick::Tree;

use crate::theme;

const ROW: f32 = 24.0;

/// Draws `tree` filling the width of `ui`; one row per leaf.
pub fn show(ui: &mut egui::Ui, tree: &Tree) {
    let leaves = tree.leaf_names().len().max(1);
    let depth = depth_of(tree, tree.root).max(1);
    let width = ui.available_width().max(320.0);
    let height = ROW * leaves as f32 + 16.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let painter = ui.painter_at(rect);
    let label_room = 150.0_f32.min(width * 0.35);
    let step = (width - label_room - 24.0) / depth as f32;
    let mut next_row = 0usize;
    let mut layout = vec![Pos2::ZERO; tree.nodes.len()];
    place(tree, tree.root, 0, &mut next_row, &mut layout, rect, step);
    let line = Stroke::new(2.0, theme::MUTED);
    for (i, node) in tree.nodes.iter().enumerate() {
        if node.children.is_empty() {
            continue;
        }
        let p = layout[i];
        let ys: Vec<f32> = node.children.iter().map(|&c| layout[c].y).collect();
        let (top, bottom) = (
            ys.iter().copied().fold(f32::MAX, f32::min),
            ys.iter().copied().fold(f32::MIN, f32::max),
        );
        painter.line_segment([Pos2::new(p.x, top), Pos2::new(p.x, bottom)], line);
        for &c in &node.children {
            let q = layout[c];
            painter.line_segment([Pos2::new(p.x, q.y), q], line);
        }
    }
    for (i, node) in tree.nodes.iter().enumerate() {
        let p = layout[i];
        if node.children.is_empty() {
            painter.circle_filled(p, 5.0, theme::TEAL);
            painter.text(
                p + Vec2::new(10.0, 0.0),
                Align2::LEFT_CENTER,
                node.label.as_deref().unwrap_or(""),
                FontId::proportional(14.0),
                theme::INK,
            );
        } else {
            painter.circle_filled(p, 3.5, theme::INDIGO);
        }
    }
}

/// Leaves get rows in order, at the full depth; inner nodes sit at their
/// depth, in the middle of their children.
fn place(
    tree: &Tree,
    n: usize,
    depth: usize,
    next_row: &mut usize,
    layout: &mut [Pos2],
    rect: egui::Rect,
    step: f32,
) -> f32 {
    let x = rect.left() + 12.0 + depth as f32 * step;
    let y = if tree.nodes[n].children.is_empty() {
        let y = rect.top() + 8.0 + ROW * (*next_row as f32 + 0.5);
        *next_row += 1;
        layout[n] = Pos2::new(rect.left() + 12.0 + max_x_steps(tree) as f32 * step, y);
        return y;
    } else {
        let ys: Vec<f32> = tree.nodes[n]
            .children
            .clone()
            .into_iter()
            .map(|c| place(tree, c, depth + 1, next_row, layout, rect, step))
            .collect();
        (ys.iter().copied().fold(f32::MAX, f32::min) + ys.iter().copied().fold(f32::MIN, f32::max))
            / 2.0
    };
    layout[n] = Pos2::new(x, y);
    y
}

fn max_x_steps(tree: &Tree) -> usize {
    depth_of(tree, tree.root)
}

fn depth_of(tree: &Tree, n: usize) -> usize {
    tree.nodes[n]
        .children
        .iter()
        .map(|&c| 1 + depth_of(tree, c))
        .max()
        .unwrap_or(0)
}
