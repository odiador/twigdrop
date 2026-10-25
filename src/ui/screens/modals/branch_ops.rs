use ratatui::{
    Frame,
    layout::Alignment,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

pub fn render_confirm_delete(
    f: &mut Frame,
    names: &[String],
    locale: &crate::i18n::Locale,
    theme: &crate::ui::theme::Theme,
) {
    let full_area = f.area();
    crate::ui::components::apply_dimmed_backdrop(f.buffer_mut(), full_area, theme);

    let (mx, my) = crate::ui::theme::MODAL_MD;
    let inner = crate::ui::components::centered_rect(mx, my, full_area);

    f.render_widget(Clear, inner);

    let branch_list = if names.len() > 3 {
        format!("{} branches (including {})", names.len(), names[0])
    } else {
        names.join(", ")
    };

    let block = Block::default()
        .title(Line::from(locale.branches.confirm_unpushed_title).alignment(Alignment::Center))
        .borders(Borders::ALL)
        .border_style(theme.danger.add_modifier(Modifier::BOLD))
        .style(Style::default().bg(theme.danger_surface));

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            locale.branches.confirm_unique_msg,
            theme.warning.add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center),
        Line::from(""),
        Line::from(Span::styled(branch_list, theme.accent2)).alignment(Alignment::Center),
        Line::from(""),
        Line::from(locale.branches.confirm_data_loss).alignment(Alignment::Center),
        Line::from(""),
        Line::from(locale.branches.confirm_prompt).alignment(Alignment::Center),
    ];
    let p = Paragraph::new(text).block(block);
    f.render_widget(p, inner);
}

pub fn render_create_branch(f: &mut Frame, input: &str, theme: &crate::ui::theme::Theme) {
    let full_area = f.area();
    crate::ui::components::apply_dimmed_backdrop(f.buffer_mut(), full_area, theme);
    let (mx, my) = crate::ui::theme::MODAL_SM;
    let inner = crate::ui::components::centered_rect(mx, my, full_area);

    f.render_widget(Clear, inner);

    let block = Block::default()
        .title(Line::from(" Create New Branch ").alignment(Alignment::Left))
        .borders(Borders::ALL)
        .border_style(theme.accent2);

    let p = Paragraph::new(format!(
        "\nName: {input}\n\n(Enter to create, Esc to cancel)"
    ))
    .block(block)
    .alignment(Alignment::Center);
    f.render_widget(p, inner);
}

pub fn render_rename_branch(
    f: &mut Frame,
    old: &str,
    input: &str,
    theme: &crate::ui::theme::Theme,
) {
    let full_area = f.area();
    crate::ui::components::apply_dimmed_backdrop(f.buffer_mut(), full_area, theme);
    let (mx, my) = crate::ui::theme::MODAL_SM;
    let inner = crate::ui::components::centered_rect(mx, my, full_area);

    f.render_widget(Clear, inner);

    let block = Block::default()
        .title(Line::from(format!(" Rename Branch: {old} ")).alignment(Alignment::Left))
        .borders(Borders::ALL)
        .border_style(theme.accent2);

    let p = Paragraph::new(format!(
        "\nNew name: {input}\n\n(Enter to rename, Esc to cancel)"
    ))
    .block(block)
    .alignment(Alignment::Center);
    f.render_widget(p, inner);
}
