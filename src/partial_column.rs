use oelung::{anyhow, Component, ComponentInterface, FlexColumnBuilder, Grid};
use tracing::instrument;

pub struct PartialColumn<'a, TRenderRow: 'a + Fn(usize) -> Result<Component<'a>, anyhow::Error>> {
    pub top_line_num: usize,
    pub render_row: TRenderRow,
}

impl<'a, TRenderRow: 'a + Fn(usize) -> Result<Component<'a>, anyhow::Error>>
    PartialColumn<'a, TRenderRow>
{
    pub fn new(top_line_num: usize, render_row: TRenderRow) -> Self {
        Self {
            top_line_num,
            render_row,
        }
    }
}

impl<'a, TRenderRow: 'a + Fn(usize) -> Result<Component<'a>, anyhow::Error>> ComponentInterface
    for PartialColumn<'a, TRenderRow>
{
    #[instrument(level = "trace", skip(self, grid))]
    fn render(&self, grid: Grid) -> Result<Component<'_>, anyhow::Error> {
        Ok({
            let mut flex_column = FlexColumnBuilder::default();
            for line_num in self.top_line_num..self.top_line_num + usize::from(grid.height) {
                flex_column = flex_column.child((self.render_row)(line_num)?);
            }
            flex_column.build().unwrap().into()
        })
    }

    fn flex_grow(&self) -> Option<f64> {
        Some(1.0)
    }
}
