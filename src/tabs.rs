use oelung::{anyhow, soft, Component, ComponentInterface, FlexRowBuilder, Grid};
use smallvec::SmallVec;
use smol_str::SmolStr;

pub struct Tabs<'a> {
    pub tabs: TabsList<'a>,
    pub selected_index: usize,
    pub flex_grow: Option<f64>,
}

impl<'a> Tabs<'a> {
    pub fn new(
        tabs: impl IntoIterator<Item = Tab<'a>>,
        selected_index: usize,
        flex_grow: Option<f64>,
    ) -> Self {
        Self {
            tabs: tabs.into_iter().collect(),
            selected_index,
            flex_grow,
        }
    }
}

impl<'a> ComponentInterface for Tabs<'_> {
    fn render(&self, _grid: Grid) -> Result<Component<'_>, anyhow::Error> {
        Ok(soft! {
            %FlexColumn
              children => [
                {
                    let mut flex_row = FlexRowBuilder::default();
                    for tab in &self.tabs {
                        flex_row = flex_row.child(soft! {
                            %Text
                              text => &tab.label
                              flex_grow => 1
                        });
                    }
                    flex_row.build().unwrap().into()
                },
                self.tabs[self.selected_index].component.clone()
              ]
              maybe_flex_grow => self.flex_grow
        })
    }

    fn flex_grow(&self) -> Option<f64> {
        self.flex_grow
    }
}

pub type TabsList<'a> = SmallVec<Tab<'a>, 10>;

pub struct Tab<'a> {
    pub label: SmolStr,
    pub component: Component<'a>,
}

impl<'a> Tab<'a> {
    pub fn new(label: SmolStr, component: Component<'a>) -> Self {
        Self { label, component }
    }
}
