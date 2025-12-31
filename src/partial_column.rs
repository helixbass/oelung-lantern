use oelung::{anyhow, Component, ComponentInterface, Grid};
use tracing::instrument;

pub struct PartialColumn<'a, TRenderRow: Fn(usize) -> Result<Component<'a>, anyhow::Error>> {
    pub top_line_num: usize,
    pub render_row: TRenderRow,
}

impl<'a, TRenderRow: Fn(usize) -> Result<Component<'a>, anyhow::Error>>
    PartialColumn<'a, TRenderRow>
{
    pub fn new(top_line_num: usize, render_row: TRenderRow) -> Self {
        Self {
            top_line_num,
            render_row,
        }
    }
}

impl<'a, TRenderRow: Fn(usize) -> Result<Component<'a>, anyhow::Error>> ComponentInterface
    for PartialColumn<'a, TRenderRow>
{
    #[instrument(level = "trace", skip(self, grid))]
    fn render<'b>(&self, grid: Grid) -> Result<Component<'b>, anyhow::Error> {
        unimplemented!()
    }

    fn flex_grow(&self) -> Option<f64> {
        Some(1.0)
    }
}
