use crate::{About, ChangeModel, Exit, Open, Repository, SaveInput, SaveOutput};
use gpui::{Menu, MenuItem, SharedString};

pub fn model_menu(
    models: impl IntoIterator<Item = impl Into<SharedString>>,
    selected_model: Option<impl Into<SharedString>>,
) -> Menu {
    let selected_model = selected_model.map(Into::into);

    Menu {
        name: t!("model").into(),
        items: models
            .into_iter()
            .map(|model| {
                let model = model.into();
                let checked = selected_model
                    .as_ref()
                    .map_or(false, |selected| selected == &model);

                MenuItem::action(&model, ChangeModel::new(model.to_string())).checked(checked)
            })
            .collect::<Vec<_>>(),
    }
}

pub fn file_menu() -> Menu {
    Menu {
        name: t!("file").into(),
        items: vec![
            MenuItem::action(t!("open"), Open),
            MenuItem::Separator,
            MenuItem::submenu(Menu {
                name: t!("save").into(),
                items: vec![
                    MenuItem::action(t!("input"), SaveInput),
                    MenuItem::action(t!("output"), SaveOutput),
                ],
            }),
            MenuItem::Separator,
            MenuItem::action(t!("exit"), Exit),
        ],
    }
}

pub fn help_menu() -> Menu {
    Menu {
        name: t!("help").into(),
        items: vec![
            MenuItem::action(t!("repository"), Repository),
            MenuItem::Separator,
            MenuItem::action(t!("about"), About),
        ],
    }
}
