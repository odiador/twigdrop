use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Color,
};

use crate::app::App;

pub fn render_transparent_ascii(
    buf: &mut ratatui::buffer::Buffer,
    ascii: &str,
    area: Rect,
    color: Color,
) {
    let lines: Vec<&str> = ascii.trim_matches('\n').lines().collect();
    if lines.is_empty() {
        return;
    }

    let max_width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16;
    let height = lines.len() as u16;

    let start_x = area.x + area.width.saturating_sub(max_width) / 2;
    let start_y = area.y + area.height.saturating_sub(height) / 2;

    for (row_idx, line) in lines.iter().enumerate() {
        let y = start_y + row_idx as u16;
        if y >= area.bottom() {
            break;
        }

        for (x, ch) in (start_x..).zip(line.chars()) {
            if x >= area.right() {
                break;
            }
            if ch != ' ' {
                buf[(x, y)].set_char(ch).set_fg(color);
            }
        }
    }
}

pub fn render_main_menu(f: &mut Frame, app: &App) {
    let area = f.area();
    let buf = f.buffer_mut();

    crate::ui::components::apply_dimmed_backdrop(buf, area);

    let ascii_logo = r#"
 ████████╗██╗    ██╗██╗ ██████╗ ██████╗ ██████╗  ██████╗ ██████╗ 
 ╚══██╔══╝██║    ██║██║██╔════╝ ██╔══██╗██╔══██╗██╔═══██╗██╔══██╗
    ██║   ██║ █╗ ██║██║██║  ███╗██║  ██║██████╔╝██║   ██║██████╔╝
    ██║   ██║███╗██║██║██║   ██║██║  ██║██╔══██╗██║   ██║██╔═══╝ 
    ██║   ╚███╔███╔╝██║╚██████╔╝██████╔╝██║  ██║╚██████╔╝██║     
    ╚═╝    ╚══╝╚══╝ ╚═╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚═╝     
"#;

    let opt_options = r#"
  ___  ____ _____ ___ ___  _  _ ___ 
 / _ \|  _ \_   _|_ _/ _ \| \| / __|
| (_) | |_) || |  | | (_) | .` \__ \
 \___/| .__/ |_| |___\___/|_|\_|___/
      |_|                           
"#;

    let opt_help = r#"
 _  _ ___ _    ___ 
| || | __| |  | _ \
| __ | _|| |__|  _/
|_||_|___|____|_|  
"#;

    let opt_quit = r#"
  ___  _   _ ___ _____ 
 / _ \| | | |_ _|_   _|
| (_) | |_| || |  | |  
 \__\_\\___/|___| |_|  
"#;

    let total_height = 8 + 2 + 6 + 5 + 5;
    let v_margin = area.height.saturating_sub(total_height) / 2;

    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(v_margin),
            Constraint::Length(8),
            Constraint::Length(2),
            Constraint::Length(6),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Min(0),
        ])
        .split(area);

    render_transparent_ascii(buf, ascii_logo, v_chunks[1], Color::Cyan);

    let options_ascii = [opt_options, opt_help, opt_quit];
    for (i, opt) in options_ascii.iter().enumerate() {
        let is_selected = i == app.ui.main_menu_state.selected;
        let color = if is_selected {
            Color::White
        } else {
            Color::Rgb(60, 60, 80)
        };

        render_transparent_ascii(buf, opt, v_chunks[3 + i], color);
    }
}
