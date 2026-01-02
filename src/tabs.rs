use oelung::{anyhow, soft, Component, ComponentInterface, FlexRowBuilder, Grid};
use smallvec::SmallVec;
use smol_str::SmolStr;

pub struct Tabs<'a> {
    pub tabs: TabsList<'a>,
    pub selected_index: usize,
}

impl<'a> Tabs<'a> {
    pub fn new(tabs: TabsList<'a>, selected_index: usize) -> Self {
        Self {
            tabs,
            selected_index,
        }
    }
}

impl<'a> ComponentInterface for &'a Tabs<'_> {
    fn render(&self, _grid: Grid) -> Result<Component<'_>, anyhow::Error> {
        Ok(soft! {
            %FlexColumn
              children => [
                {
                    let mut flex_row = FlexRowBuilder::default();
                    for tab in &self.tabs {
                        flex_row = flex_row.child(soft! {
                            %Text &tab.label
                        });
                    }
                    flex_row.build().unwrap()
                },
                self.tabs[self.selected_index].component.clone()
              ]
        })
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
