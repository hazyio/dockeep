use gpui_kit::{
    App, SharedString,
    component::setting::{
        NumberFieldOptions, SettingField, SettingGroup, SettingItem, SettingPage,
    },
};

use crate::{
    config::{AppConfig, ImageFormat},
    utils::save_debouncer::SaveDebouncer,
};
pub struct CapturePage {}

impl CapturePage {
    pub fn page(debouncer: SaveDebouncer, default_config: &AppConfig) -> SettingPage {
        let capture_setting_defaults = default_config.capture_setting.clone();

        SettingPage::new(t!("title.capture"))
            .resettable(true)
            .group(
                SettingGroup::new()
                    .title(t!("title.screenshot_settings"))
                    .item(
                        SettingItem::new(
                            t!("title.capture_from_surface"),
                            SettingField::switch(
                                {
                                    let debouncer = debouncer.clone();
                                    move |_: &App| {
                                        debouncer
                                            .config
                                            .borrow()
                                            .capture_setting
                                            .capture_from_surface
                                    }
                                },
                                {
                                    let debouncer = debouncer.clone();
                                    move |val: bool, cx: &mut App| {
                                        debouncer
                                            .config
                                            .borrow_mut()
                                            .capture_setting
                                            .capture_from_surface = val;

                                        debouncer.schedule(cx);
                                    }
                                },
                            )
                            .default_value(capture_setting_defaults.capture_from_surface),
                        )
                        .description(t!("description.capture_from_surface", min = 10).to_string()),
                    )
                    .item(SettingItem::new(
                        t!("label.format"),
                        SettingField::dropdown(
                            ImageFormat::all()
                                .iter()
                                .map(|lang| (lang.to_value().into(), lang.to_string().into()))
                                .collect(),
                            {
                                let debouncer = debouncer.clone();
                                move |_: &App| {
                                    SharedString::from(
                                        debouncer
                                            .config
                                            .borrow()
                                            .capture_setting
                                            .image_format
                                            .to_value(),
                                    )
                                }
                            },
                            {
                                let debouncer = debouncer.clone();
                                move |val: SharedString, cx: &mut App| {
                                    debouncer.config.borrow_mut().capture_setting.image_format =
                                        ImageFormat::from_value(&val.to_string());

                                    debouncer.schedule(cx);
                                }
                            },
                        )
                        .default_value(capture_setting_defaults.image_format.to_value()),
                    ))
                    .item(
                        SettingItem::new(
                            t!("label.crop_timeout"),
                            SettingField::number_input(
                                NumberFieldOptions {
                                    min: 0.0,
                                    max: 5400.0,
                                    step: 1.0,
                                },
                                {
                                    let debouncer = debouncer.clone();
                                    move |_: &App| {
                                        debouncer.config.borrow().capture_setting.crop_timeout
                                            as f64
                                    }
                                },
                                {
                                    let debouncer = debouncer.clone();
                                    move |val: f64, cx: &mut App| {
                                        debouncer
                                            .config
                                            .borrow_mut()
                                            .capture_setting
                                            .crop_timeout = val as u64;
                                        debouncer.schedule(cx);
                                    }
                                },
                            )
                            .default_value(capture_setting_defaults.crop_timeout as f64),
                        )
                        .description(t!("description.crop_timeout").to_string()),
                    )
                    .item(
                        SettingItem::new(
                            t!("label.quality"),
                            SettingField::number_input(
                                NumberFieldOptions {
                                    min: 0.0,
                                    max: 100.0,
                                    step: 1.0,
                                },
                                {
                                    let debouncer = debouncer.clone();
                                    move |_: &App| {
                                        debouncer.config.borrow().capture_setting.quality as f64
                                    }
                                },
                                {
                                    let debouncer = debouncer.clone();
                                    move |val: f64, cx: &mut App| {
                                        debouncer.config.borrow_mut().capture_setting.quality =
                                            val as u8;
                                        debouncer.schedule(cx);
                                    }
                                },
                            )
                            .default_value(capture_setting_defaults.quality),
                        )
                        .description(t!("description.min_quality", min = 10).to_string()),
                    ),
            )
            .group(
                SettingGroup::new()
                    .title(t!("title.desktop_capture_settings"))
                    .item(
                        SettingItem::new(
                            t!("label.width"),
                            SettingField::number_input(
                                NumberFieldOptions {
                                    min: 0.0,
                                    max: 3840.0,
                                    step: 1.0,
                                },
                                {
                                    let debouncer = debouncer.clone();
                                    move |_: &App| {
                                        debouncer
                                            .config
                                            .borrow()
                                            .capture_setting
                                            .desktop_capture_sizing
                                            .width
                                    }
                                },
                                {
                                    let debouncer = debouncer.clone();
                                    move |val: f64, cx: &mut App| {
                                        debouncer
                                            .config
                                            .borrow_mut()
                                            .capture_setting
                                            .desktop_capture_sizing
                                            .width = val;
                                        debouncer.schedule(cx);
                                    }
                                },
                            )
                            .default_value(capture_setting_defaults.desktop_capture_sizing.width),
                        )
                        .description(t!("description.min_width", min = 800).to_string()),
                    )
                    .item(
                        SettingItem::new(
                            t!("label.height"),
                            SettingField::number_input(
                                NumberFieldOptions {
                                    min: 0.0,
                                    max: 2160.0,
                                    step: 1.0,
                                },
                                {
                                    let debouncer = debouncer.clone();
                                    move |_: &App| {
                                        debouncer
                                            .config
                                            .borrow()
                                            .capture_setting
                                            .desktop_capture_sizing
                                            .height
                                    }
                                },
                                {
                                    let debouncer = debouncer.clone();
                                    move |val: f64, cx: &mut App| {
                                        debouncer
                                            .config
                                            .borrow_mut()
                                            .capture_setting
                                            .desktop_capture_sizing
                                            .height = val;
                                        debouncer.schedule(cx);
                                    }
                                },
                            )
                            .default_value(capture_setting_defaults.desktop_capture_sizing.height),
                        )
                        .description(t!("description.min_height", min = 600).to_string()),
                    ),
            )
            .group(
                SettingGroup::new()
                    .title(t!("title.mobile_capture_settings"))
                    .item(
                        SettingItem::new(
                            t!("label.width"),
                            SettingField::number_input(
                                NumberFieldOptions {
                                    min: 0.0,
                                    max: 480.0,
                                    step: 1.0,
                                },
                                {
                                    let debouncer = debouncer.clone();
                                    move |_: &App| {
                                        debouncer
                                            .config
                                            .borrow()
                                            .capture_setting
                                            .mobile_capture_sizing
                                            .width
                                    }
                                },
                                {
                                    let debouncer = debouncer.clone();
                                    move |val: f64, cx: &mut App| {
                                        debouncer
                                            .config
                                            .borrow_mut()
                                            .capture_setting
                                            .mobile_capture_sizing
                                            .width = val;
                                        debouncer.schedule(cx);
                                    }
                                },
                            )
                            .default_value(capture_setting_defaults.mobile_capture_sizing.width),
                        )
                        .description(t!("description.min_width", min = 320).to_string()),
                    )
                    .item(
                        SettingItem::new(
                            t!("label.height"),
                            SettingField::number_input(
                                NumberFieldOptions {
                                    min: 0.0,
                                    max: 1024.0,
                                    step: 1.0,
                                },
                                {
                                    let debouncer = debouncer.clone();
                                    move |_: &App| {
                                        debouncer
                                            .config
                                            .borrow()
                                            .capture_setting
                                            .mobile_capture_sizing
                                            .height
                                    }
                                },
                                {
                                    let debouncer = debouncer.clone();
                                    move |val: f64, cx: &mut App| {
                                        debouncer
                                            .config
                                            .borrow_mut()
                                            .capture_setting
                                            .mobile_capture_sizing
                                            .height = val;
                                        debouncer.schedule(cx);
                                    }
                                },
                            )
                            .default_value(capture_setting_defaults.mobile_capture_sizing.height),
                        )
                        .description(t!("description.min_height", min = 480).to_string()),
                    ),
            )
    }
}
