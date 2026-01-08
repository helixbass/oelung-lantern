use crossterm::style::Color;
use itertools::{EitherOrBoth, Itertools};
use oelung::{backend::Cell, BackendMemory, Position, RowOrColumnNumber};
use squalid::{_d, regex, EverythingExt};

pub fn assert_expected_prefix<TString: AsRef<str>>(
    memory_backend: &BackendMemory,
    expected_prefix: &[TString],
) {
    let expected_prefix = expected_prefix
        .into_iter()
        .map(|expected| expected.as_ref().to_owned())
        .collect::<Vec<_>>();
    let expected_screen_states = expected_prefix
        .into_iter()
        .map(|expected_screen| ExpectedScreenState::from(&*expected_screen))
        .collect::<Vec<_>>();

    let expected_prefix_len = expected_screen_states.len();
    expected_screen_states
        .into_iter()
        .zip_longest(memory_backend.rendered_grids.iter())
        .take(expected_prefix_len)
        .for_each(|either_or_both| {
            let EitherOrBoth::Both(expected_screen_state, rendered_grid) = either_or_both else {
                panic!("Should have expected screen state and rendered grid");
            };
            assert_eq!(
                rendered_grid_to_styled_chunks(rendered_grid),
                expected_screen_state.contents
            );
        });
}

pub fn assert_expected_screen_contents(
    memory_backend: &BackendMemory,
    expected_screen_contents: &str,
) {
    assert_expected_screen_contents_rendered_grid(&memory_backend.grid, expected_screen_contents)
}

pub fn assert_expected_screen_contents_rendered_grid(
    rendered_grid: &[Vec<Cell>],
    expected_screen_contents: &str,
) {
    let expected_screen_state: ExpectedScreenState = expected_screen_contents.into();
    assert_eq!(
        rendered_grid_to_styled_chunks(rendered_grid),
        expected_screen_state.contents
    );
}

pub struct ExpectedScreenState {
    pub contents: Vec<Vec<StyledChunk>>,
    pub cursor_position: Option<Position>,
}

impl From<&str> for ExpectedScreenState {
    fn from(value: &str) -> Self {
        strip_trailing_newline(value)
            .split("\n")
            .map(parse_line)
            .fold(
                (_d(), None),
                |mut accum: (Vec<Vec<StyledChunk>>, Option<Position>), (line_chunks, cursor)| {
                    accum.0.push(line_chunks);
                    if let Some(cursor) = cursor {
                        if accum.1.is_some() {
                            panic!("Rendered more than one cursor");
                        }
                        accum.1 = Some(cursor);
                    }
                    accum
                },
            )
            .thrush(|(contents, cursor_position)| Self {
                contents,
                cursor_position,
            })
    }
}

pub fn strip_trailing_newline(file_contents: &str) -> &str {
    if file_contents.ends_with("\n") {
        &file_contents[..file_contents.len() - 1]
    } else {
        file_contents
    }
}

fn parse_line_to_pieces(line: &str) -> Vec<LinePiece> {
    if line.is_empty() {
        return _d();
    }
    let mut next_pos = 0;
    let mut ret: Vec<LinePiece> = _d();
    while let Some(index) = line[next_pos..].find('<') {
        if index > 0 {
            ret.push(LinePiece::Text(line[next_pos..next_pos + index].to_owned()));
        }
        let left_caret_pos = next_pos + index;
        if regex!(r#"^color="#).is_match(&line[left_caret_pos + 1..]) {
            let left_curly_pos = left_caret_pos + 7;
            let Some(match_) = regex!(r#"^\{.+\}>"#).find(&line[left_curly_pos..]) else {
                panic!("expected tag value");
            };
            let match_len = match_.len();
            let color_len = match_len - 3;
            let color = parse_color(&line[left_curly_pos + 1..left_curly_pos + 1 + color_len]);
            let one_after_right_caret_pos = left_curly_pos + match_len;
            ret.push(LinePiece::StyleOpenTag(Style {
                foreground_color: color,
                background_color: Color::Reset,
            }));
            let Some(match_) = regex!(r#"^.+</>"#).find(&line[one_after_right_caret_pos..]) else {
                panic!("expected closing tag");
            };
            let match_len = match_.len();
            let close_left_caret_pos = one_after_right_caret_pos + (match_len - 3);
            ret.push(LinePiece::Text(
                line[one_after_right_caret_pos..close_left_caret_pos].to_owned(),
            ));
            ret.push(LinePiece::StyleCloseTag);
            next_pos = one_after_right_caret_pos + match_len;
        } else if regex!(r#"^cursor/>"#).is_match(&line[left_caret_pos + 1..]) {
            ret.push(LinePiece::CursorTag);
            next_pos = left_caret_pos + 9;
        } else {
            panic!("expected style tag");
        }
    }
    if next_pos < line.len() {
        ret.push(LinePiece::Text(line[next_pos..].to_owned()));
    }
    ret
}

enum LinePiece {
    Text(String),
    CursorTag,
    StyleOpenTag(Style),
    StyleCloseTag,
}

fn parse_line(line: &str) -> (Vec<StyledChunk>, Option<RowOrColumnNumber>) {
    #[derive(Default)]
    enum CurrentState {
        #[default]
        Default,
        SawText(String),
        SawTextAndJustSawCursor(String),
        InsideStyleTag(Style),
        SawTextInsideStyleTag(String, Style),
        SawTextInsideStyleTagAndJustSawCursor(String, Style),
    }
    let mut ret: Vec<StyledChunk> = _d();
    let mut cursor_position: Option<RowOrColumnNumber> = _d();
    let mut current_state: CurrentState = _d();
    for line_piece in parse_line_to_pieces(line) {
        match (line_piece, current_state) {
            (LinePiece::Text(text), CurrentState::Default) => {
                current_state = CurrentState::SawText(text);
            }
            (LinePiece::Text(more_text), CurrentState::SawTextAndJustSawCursor(text)) => {
                current_state = CurrentState::SawText(format!("{text}{more_text}"));
            }
            (LinePiece::Text(text), CurrentState::InsideStyleTag(style)) => {
                current_state = CurrentState::SawTextInsideStyleTag(text, style);
            }
            (
                LinePiece::Text(more_text),
                CurrentState::SawTextInsideStyleTagAndJustSawCursor(text, style),
            ) => {
                current_state =
                    CurrentState::SawTextInsideStyleTag(format!("{text}{more_text}"), style);
            }
            (LinePiece::CursorTag, CurrentState::Default) => {
                assert!(cursor_position.is_none());
                cursor_position = Some(ret.iter().map(|chunk| chunk.contents.len()).sum());
            }
            (LinePiece::CursorTag, CurrentState::SawText(text)) => {
                assert!(cursor_position.is_none());
                cursor_position =
                    Some(ret.iter().map(|chunk| chunk.contents.len()).sum() + text.len());
                current_state = CurrentState::SawTextAndJustSawCursor(text);
            }
            (LinePiece::CursorTag, CurrentState::InsideStyleTag(style)) => {
                assert!(cursor_position.is_none());
                cursor_position = Some(ret.iter().map(|chunk| chunk.contents.len()).sum());
                current_state = CurrentState::InsideStyleTag(style);
            }
            (LinePiece::CursorTag, CurrentState::SawTextInsideStyleTag(text, style)) => {
                assert!(cursor_position.is_none());
                cursor_position =
                    Some(ret.iter().map(|chunk| chunk.contents.len()).sum() + text.len());
                current_state = CurrentState::SawTextInsideStyleTagAndJustSawCursor(text, style);
            }
            (LinePiece::StyleOpenTag(style), CurrentState::Default) => {
                current_state = CurrentState::InsideStyleTag(style);
            }
            (LinePiece::StyleOpenTag(style), CurrentState::SawText(text))
            | (LinePiece::StyleOpenTag(style), CurrentState::SawTextAndJustSawCursor(text)) => {
                ret.push(StyledChunk {
                    contents: text,
                    style: Style {
                        foreground_color: Color::Reset,
                        background_color: Color::Reset,
                    },
                });
                current_state = CurrentState::InsideStyleTag(style);
            }
            (LinePiece::StyleCloseTag, CurrentState::SawTextInsideStyleTag(text, style))
            | (
                LinePiece::StyleCloseTag,
                CurrentState::SawTextInsideStyleTagAndJustSawCursor(text, style),
            ) => {
                ret.push(StyledChunk {
                    contents: text,
                    style,
                });
                current_state = CurrentState::Default;
            }
            _ => panic!("invalid line state"),
        }
    }
    match current_state {
        CurrentState::Default => {}
        CurrentState::SawText(text) => {
            ret.push(StyledChunk {
                contents: text,
                style: Style {
                    foreground_color: Color::Reset,
                    background_color: Color::Reset,
                },
            });
        }
        CurrentState::InsideStyleTag(_) | CurrentState::SawTextInsideStyleTag(_, _) => {
            panic!("unclosed style tag")
        }
    }
    (ret, cursor_position)
}

fn parse_color(text: &str) -> Color {
    match text {
        "Red" => Color::Red,
        _ => match regex!(r#"^Rgb\((\d+),\s*(\d+),\s*(\d+)\)$"#).captures(text) {
            Some(captures) => Color::Rgb {
                r: captures[1].parse().unwrap(),
                g: captures[2].parse().unwrap(),
                b: captures[3].parse().unwrap(),
            },
            None => unimplemented!(),
        },
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct StyledChunk {
    pub contents: String,
    pub style: Style,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Style {
    foreground_color: Color,
    background_color: Color,
}

fn rendered_row_to_styled_chunks(rendered_row: &[Cell]) -> Vec<StyledChunk> {
    let mut last_non_blank_cell_index: Option<usize> = _d();
    let blank_cell = Cell {
        content: ' ',
        foreground_color: Color::Reset,
        background_color: Color::Reset,
    };
    for cell_index in (0..rendered_row.len()).rev() {
        if rendered_row[cell_index] != blank_cell {
            last_non_blank_cell_index = Some(cell_index);
            break;
        }
    }
    let Some(last_non_blank_cell_index) = last_non_blank_cell_index else {
        return _d();
    };
    let mut ret: Vec<StyledChunk> = _d();
    let rendered_row = &rendered_row[..=last_non_blank_cell_index];
    let mut current_style: Option<Style> = _d();
    let mut current_chunk: String = _d();
    for cell in rendered_row {
        if current_style.is_none() {
            current_style = Some(Style {
                foreground_color: cell.foreground_color,
                background_color: cell.background_color,
            });
        }
        if current_style.unwrap().foreground_color == cell.foreground_color
            && current_style.unwrap().background_color == cell.background_color
        {
            current_chunk.push(cell.content);
        } else {
            current_style = Some(Style {
                foreground_color: cell.foreground_color,
                background_color: cell.background_color,
            });
            current_chunk = _d();
        }
    }
    if !current_chunk.is_empty() {
        ret.push(StyledChunk {
            contents: current_chunk,
            style: current_style.unwrap(),
        });
    }
    ret
}

fn rendered_grid_to_styled_chunks(rendered_grid: &[Vec<Cell>]) -> Vec<Vec<StyledChunk>> {
    let including_trailing_rows = rendered_grid
        .into_iter()
        .map(|row| rendered_row_to_styled_chunks(row))
        .collect::<Vec<_>>();
    including_trailing_rows
        .into_iter()
        .rev()
        .skip_while(|row| row.is_empty())
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}
