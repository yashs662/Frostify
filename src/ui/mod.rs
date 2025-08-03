use crate::{
    app::AppEvent,
    ui::{
        ecs::{
            NamedRef,
            builders::modal::ModalBuilder,
            components::{NotchPosition, NotchType},
        },
        layout::{FlexDirection, FlexValue},
    },
    wgpu_ctx::WgpuCtx,
};
use animation::{
    AnimationConfig, AnimationDirection, AnimationRange, AnimationType, AnimationWhen,
    EasingFunction,
};
use color::Color;
use ecs::{
    EntityId, GradientColorStop, GradientType,
    builders::{
        EntityBuilder,
        background::{
            BackgroundBuilder, BackgroundColorConfig, BackgroundGradientConfig, FrostedGlassConfig,
        },
        button::ButtonBuilder,
        container::ContainerBuilder,
        image::{ImageBuilder, ScaleMode},
        text::TextBuilder,
    },
};
use layout::{AlignItems, Anchor, BorderRadius, Edges, JustifyContent};
use std::time::Duration;
use strum_macros::EnumString;

pub mod animation;
pub mod asset;
pub mod color;
pub mod ecs;
pub mod geometry;
pub mod img_utils;
pub mod layout;
pub mod z_index_manager;

#[derive(Debug, Clone, Copy, PartialEq, Default, EnumString)]
pub enum UiView {
    #[default]
    Splash,
    Login,
    Home,
    Test,
}

pub fn create_fancy_background_gradient(
    wgpu_ctx: &mut WgpuCtx,
    layout_context: &mut layout::LayoutContext,
) -> EntityId {
    let fancy_background_container_id = ContainerBuilder::new()
        .with_debug_name("Fancy Background Gradient Container")
        .with_fixed_position(Anchor::Center)
        .with_z_index(-1)
        .build(
            &mut layout_context.world,
            &mut layout_context.z_index_manager,
        );

    let gradient_composition_0_id = BackgroundBuilder::with_gradient(BackgroundGradientConfig {
        color_stops: vec![
            GradientColorStop {
                color: Color::Transparent,
                position: 0.0,
            },
            GradientColorStop {
                color: Color::Crimson,
                position: 0.5,
            },
            GradientColorStop {
                color: Color::Transparent,
                position: 1.0,
            },
        ],
        angle: 90.0,
        gradient_type: GradientType::Linear,
        center: None,
        radius: None,
    })
    .with_fixed_position(Anchor::Center)
    .with_debug_name("Fancy Background Gradient Composition 0")
    .build(
        &mut layout_context.world,
        wgpu_ctx,
        &mut layout_context.z_index_manager,
    );

    let gradient_composition_1_id = BackgroundBuilder::with_gradient(BackgroundGradientConfig {
        color_stops: vec![
            GradientColorStop {
                color: Color::Crimson.darken(0.2),
                position: 0.0,
            },
            GradientColorStop {
                color: Color::Transparent,
                position: 1.0,
            },
        ],
        angle: 90.0,
        gradient_type: GradientType::Radial,
        center: Some((1.0, 1.0)),
        radius: Some(0.4),
    })
    .with_z_index(1)
    .with_fixed_position(Anchor::Center)
    .with_debug_name("Fancy Background Gradient Composition 1")
    .build(
        &mut layout_context.world,
        wgpu_ctx,
        &mut layout_context.z_index_manager,
    );

    let gradient_composition_2_id = BackgroundBuilder::with_gradient(BackgroundGradientConfig {
        color_stops: vec![
            GradientColorStop {
                color: Color::Crimson.darken(0.2),
                position: 0.0,
            },
            GradientColorStop {
                color: Color::Transparent,
                position: 1.0,
            },
        ],
        angle: 90.0,
        gradient_type: GradientType::Radial,
        center: Some((0.0, 0.0)),
        radius: Some(0.4),
    })
    .with_z_index(1)
    .with_fixed_position(Anchor::Center)
    .with_debug_name("Fancy Background Gradient Composition 2")
    .build(
        &mut layout_context.world,
        wgpu_ctx,
        &mut layout_context.z_index_manager,
    );

    let frosted_glass_id = BackgroundBuilder::with_frosted_glass(FrostedGlassConfig {
        tint_color: Color::Black,
        blur_radius: 50.0,
        tint_intensity: 0.0,
    })
    .with_debug_name("Fancy Background Frosted Glass")
    .with_fixed_position(Anchor::Center)
    .build(
        &mut layout_context.world,
        wgpu_ctx,
        &mut layout_context.z_index_manager,
    );

    layout_context.add_child_to_parent(fancy_background_container_id, gradient_composition_0_id);
    layout_context.add_child_to_parent(fancy_background_container_id, gradient_composition_1_id);
    layout_context.add_child_to_parent(fancy_background_container_id, gradient_composition_2_id);
    layout_context.add_child_to_parent(fancy_background_container_id, frosted_glass_id);

    fancy_background_container_id
}

pub fn create_splash_ui(wgpu_ctx: &mut WgpuCtx, layout_context: &mut layout::LayoutContext) {
    let main_container_id = ContainerBuilder::new()
        .with_debug_name("Splash Screen Main Container")
        .with_direction(FlexDirection::Column)
        .with_align_items(AlignItems::Center)
        .with_justify_content(JustifyContent::Center)
        .build(
            &mut layout_context.world,
            &mut layout_context.z_index_manager,
        );

    let splash_background_container_id = create_fancy_background_gradient(wgpu_ctx, layout_context);
    layout_context.add_child_to_parent(main_container_id, splash_background_container_id);

    let nav_bar_container_id = create_nav_bar(wgpu_ctx, layout_context, true);
    layout_context.add_child_to_parent(main_container_id, nav_bar_container_id);

    let splash_content_container_id = ContainerBuilder::new()
        .with_debug_name("Splash Screen Content Container")
        .with_direction(FlexDirection::Column)
        .with_align_items(AlignItems::Center)
        .with_justify_content(JustifyContent::Center)
        .build(
            &mut layout_context.world,
            &mut layout_context.z_index_manager,
        );

    layout_context.add_child_to_parent(main_container_id, splash_content_container_id);

    let logo_id = ImageBuilder::new("frostify_logo.png")
        .with_size(150, 150)
        .with_margin(Edges::vertical(10.0))
        .with_debug_name("Logo")
        .build(
            &mut layout_context.world,
            wgpu_ctx,
            &mut layout_context.z_index_manager,
        );

    let welcome_text_id = TextBuilder::new()
        .with_debug_name("Welcome text")
        .with_text("Welcome to Frostify!!!")
        .with_font_size(24.0)
        .with_color(Color::White)
        .set_fit_to_size()
        .build(
            &mut layout_context.world,
            wgpu_ctx,
            &mut layout_context.z_index_manager,
        );

    let loading_text_id = TextBuilder::new()
        .with_debug_name("Loading text")
        .with_text("Loading...")
        .with_color(Color::White)
        .set_fit_to_size()
        .build(
            &mut layout_context.world,
            wgpu_ctx,
            &mut layout_context.z_index_manager,
        );

    layout_context.add_child_to_parent(splash_content_container_id, logo_id);
    layout_context.add_child_to_parent(splash_content_container_id, welcome_text_id);
    layout_context.add_child_to_parent(splash_content_container_id, loading_text_id);
}

pub fn create_test_ui(wgpu_ctx: &mut WgpuCtx, layout_context: &mut layout::LayoutContext) {
    let main_container_id = ContainerBuilder::new()
        .with_debug_name("Test UI Main Container")
        .with_direction(FlexDirection::Column)
        .with_align_items(AlignItems::Center)
        .with_justify_content(JustifyContent::Center)
        .build(
            &mut layout_context.world,
            &mut layout_context.z_index_manager,
        );

    let nav_bar_id = create_nav_bar(wgpu_ctx, layout_context, true);
    let sub_container_id = ContainerBuilder::new()
        .with_debug_name("Test UI Sub Container")
        .with_direction(FlexDirection::Column)
        .with_align_items(AlignItems::Center)
        .with_justify_content(JustifyContent::Center)
        .build(
            &mut layout_context.world,
            &mut layout_context.z_index_manager,
        );

    layout_context.add_child_to_parent(main_container_id, nav_bar_id);
    layout_context.add_child_to_parent(main_container_id, sub_container_id);

    let sample_text = TextBuilder::new()
        .with_debug_name("Sample Text")
        .with_text("Test text")
        .with_font_size(24.0)
        .with_color(Color::White)
        .with_size(100, 100)
        .set_fit_to_size()
        .build(
            &mut layout_context.world,
            wgpu_ctx,
            &mut layout_context.z_index_manager,
        );

    let notch_test_id = BackgroundBuilder::with_color(BackgroundColorConfig {
        color: Color::Crimson,
    })
    .with_debug_name("Notch Test Background")
    .with_size(500, 300)
    .with_border_radius(BorderRadius::all(10.0))
    .with_border(2.0, Color::White)
    .with_notch(
        NotchType::Bottom,
        NotchPosition::Center,
        20.0,
        100.0,
        150.0,
        0.0,
    )
    .build(
        &mut layout_context.world,
        wgpu_ctx,
        &mut layout_context.z_index_manager,
    );

    let notch_test_id_2 = BackgroundBuilder::with_gradient(BackgroundGradientConfig {
        color_stops: vec![
            GradientColorStop {
                color: Color::Crimson,
                position: 0.5,
            },
            GradientColorStop {
                color: Color::Black,
                position: 1.0,
            },
        ],
        angle: 90.0,
        gradient_type: GradientType::Linear,
        center: None,
        radius: None,
    })
    .with_debug_name("Notch Test Background 2")
    .with_size(500, 300)
    .with_border_radius(BorderRadius::all(10.0))
    .with_border(2.0, Color::White)
    .with_margin(Edges::top(5.0))
    .with_notch(
        NotchType::Bottom,
        NotchPosition::Center,
        20.0,
        100.0,
        150.0,
        0.0,
    )
    .build(
        &mut layout_context.world,
        wgpu_ctx,
        &mut layout_context.z_index_manager,
    );

    let sample_text_2 = TextBuilder::new()
        .with_debug_name("Sample Text 2")
        .with_text("Test text 2")
        .with_font_size(24.0)
        .with_color(Color::Red)
        .with_size(100, 100)
        .set_fit_to_size()
        .build(
            &mut layout_context.world,
            wgpu_ctx,
            &mut layout_context.z_index_manager,
        );

    layout_context.add_child_to_parent(sub_container_id, sample_text);
    layout_context.add_child_to_parent(sub_container_id, notch_test_id);
    layout_context.add_child_to_parent(sub_container_id, notch_test_id_2);
    layout_context.add_child_to_parent(sub_container_id, sample_text_2);
}

pub fn create_login_ui(wgpu_ctx: &mut WgpuCtx, layout_context: &mut layout::LayoutContext) {
    let main_container_id = ContainerBuilder::new()
        .with_debug_name("Login Page Main Container")
        .with_direction(FlexDirection::Column)
        .with_align_items(AlignItems::Center)
        .with_justify_content(JustifyContent::Center)
        .build(
            &mut layout_context.world,
            &mut layout_context.z_index_manager,
        );

    let login_background_container_id = create_fancy_background_gradient(wgpu_ctx, layout_context);
    layout_context.add_child_to_parent(main_container_id, login_background_container_id);

    let nav_bar_container_id = create_nav_bar(wgpu_ctx, layout_context, true);
    layout_context.add_child_to_parent(main_container_id, nav_bar_container_id);

    let sub_container_id = ContainerBuilder::new()
        .with_debug_name("Login Page Sub Container")
        .with_direction(FlexDirection::Column)
        .with_align_items(AlignItems::Center)
        .with_justify_content(JustifyContent::Center)
        .with_z_index(1)
        .build(
            &mut layout_context.world,
            &mut layout_context.z_index_manager,
        );

    // Welcome label
    // TODO: Welcome label get clipped by the login button text figure-out why this is happening
    // https://github.com/grovesNL/glyphon/issues/141
    let welcome_text_id = TextBuilder::new()
        .with_debug_name("Welcome Text")
        .with_text("Welcome to Frostify")
        .with_font_size(24.0)
        .with_color(Color::White)
        .with_line_height(1.0)
        .set_fit_to_size()
        .build(
            &mut layout_context.world,
            wgpu_ctx,
            &mut layout_context.z_index_manager,
        );

    // Logo
    let logo_id = ImageBuilder::new("frostify_logo.png")
        .with_size(150, 150)
        .with_margin(Edges::vertical(10.0))
        .with_debug_name("Logo")
        .build(
            &mut layout_context.world,
            wgpu_ctx,
            &mut layout_context.z_index_manager,
        );

    // Login button
    let login_button_id = ButtonBuilder::new()
        .with_debug_name("Login Button")
        .with_size(FlexValue::Fixed(200.0), FlexValue::Fixed(50.0))
        .with_margin(Edges::all(10.0))
        .with_border_radius(BorderRadius::all(5.0))
        .with_border(1.0, Color::White)
        .with_background_frosted_glass(FrostedGlassConfig {
            tint_color: Color::Black,
            blur_radius: 5.0,
            tint_intensity: 0.5,
        })
        .with_animation(AnimationConfig {
            duration: Duration::from_millis(200),
            easing: EasingFunction::EaseOutExpo,
            direction: AnimationDirection::Alternate,
            animation_type: AnimationType::FrostedGlassTint {
                range: AnimationRange::new(Color::Black, Color::Crimson),
            },
            when: AnimationWhen::Hover,
        })
        .with_text("Login with Spotify")
        .with_text_color(Color::White)
        .with_click_event(AppEvent::Login)
        .with_animation(AnimationConfig {
            duration: Duration::from_millis(200),
            easing: EasingFunction::EaseOutExpo,
            direction: AnimationDirection::Alternate,
            animation_type: AnimationType::Scale {
                range: AnimationRange::new(1.0, 1.05),
                anchor: Anchor::Center,
            },
            when: AnimationWhen::Hover,
        })
        .build(layout_context, wgpu_ctx);

    layout_context.add_child_to_parent(sub_container_id, welcome_text_id);
    layout_context.add_child_to_parent(sub_container_id, logo_id);
    layout_context.add_child_to_parent(sub_container_id, login_button_id);

    layout_context.add_child_to_parent(main_container_id, sub_container_id);
}

pub fn create_app_ui(wgpu_ctx: &mut WgpuCtx, layout_context: &mut layout::LayoutContext) {
    // Main container
    let main_container_id = ContainerBuilder::new()
        .with_debug_name("Main Container")
        .with_direction(FlexDirection::Column)
        .with_align_items(AlignItems::Center)
        .with_justify_content(JustifyContent::Center)
        .build(
            &mut layout_context.world,
            &mut layout_context.z_index_manager,
        );

    let background_id = ImageBuilder::new("album_art.png")
        .with_scale_mode(ScaleMode::Cover)
        .with_debug_name("Main Background")
        .with_z_index(-2)
        .with_fixed_position(Anchor::Center)
        .build(
            &mut layout_context.world,
            wgpu_ctx,
            &mut layout_context.z_index_manager,
        );

    let frosted_glass_id = BackgroundBuilder::with_frosted_glass(FrostedGlassConfig {
        tint_color: Color::Black,
        blur_radius: 20.0,
        tint_intensity: 0.5,
    })
    .with_debug_name("Frosted Glass")
    .with_fixed_position(Anchor::Center)
    .with_z_index(-1)
    .build(
        &mut layout_context.world,
        wgpu_ctx,
        &mut layout_context.z_index_manager,
    );

    // Create nav bar using the extracted function
    let nav_bar_container_id = create_nav_bar(wgpu_ctx, layout_context, false);
    let player_container_id = create_player_bar(wgpu_ctx, layout_context);

    let app_container_id = ContainerBuilder::new()
        .with_debug_name("App Container")
        .with_direction(FlexDirection::Row)
        .build(
            &mut layout_context.world,
            &mut layout_context.z_index_manager,
        );

    let library_container_id = ContainerBuilder::new()
        .with_debug_name("Library Container")
        .with_size(FlexValue::Fixed(80.0), FlexValue::Fill)
        .with_margin(Edges::left(10.0))
        .with_direction(FlexDirection::Column)
        .with_z_index(1)
        .build(
            &mut layout_context.world,
            &mut layout_context.z_index_manager,
        );

    let library_background_id = BackgroundBuilder::with_frosted_glass(FrostedGlassConfig {
        tint_color: Color::Black,
        blur_radius: 20.0,
        tint_intensity: 0.5,
    })
    .with_debug_name("Library Background")
    .with_border_radius(BorderRadius::all(5.0))
    .with_border(1.0, Color::Black)
    .with_z_index(-1)
    .with_fixed_position(Anchor::Center)
    .build(
        &mut layout_context.world,
        wgpu_ctx,
        &mut layout_context.z_index_manager,
    );

    layout_context.add_child_to_parent(library_container_id, library_background_id);

    let library_child_container_id = ContainerBuilder::new()
        .with_debug_name("Library Child Container")
        .with_direction(FlexDirection::Column)
        .with_margin(Edges::all(5.0))
        .with_vertical_scroll()
        .build(
            &mut layout_context.world,
            &mut layout_context.z_index_manager,
        );

    for i in 0..15 {
        let image_name = if i % 2 == 0 {
            "album_art.png"
        } else {
            "test.png"
        };
        let mut image_builder = ImageBuilder::new(image_name)
            .with_debug_name(format!("Album Art {i}"))
            .with_size(FlexValue::Fixed(70.0), FlexValue::Fixed(70.0))
            .with_border_radius(BorderRadius::all(5.0))
            .with_shadow(Color::Black, (0.0, 0.0), 4.0, 0.4)
            .with_animation(AnimationConfig {
                duration: Duration::from_millis(150),
                direction: AnimationDirection::Alternate,
                easing: EasingFunction::EaseOutQuart,
                animation_type: AnimationType::Scale {
                    range: AnimationRange::new(1.0, 1.5),
                    anchor: Anchor::Left,
                },
                when: AnimationWhen::Hover,
            });

        if i == 0 {
            image_builder = image_builder.with_margin(Edges::bottom(5.0))
        } else if i == 14 {
            image_builder = image_builder.with_margin(Edges::top(5.0))
        } else {
            image_builder = image_builder.with_margin(Edges::vertical(5.0));
        }
        let image = image_builder.build(
            &mut layout_context.world,
            wgpu_ctx,
            &mut layout_context.z_index_manager,
        );

        layout_context.add_child_to_parent(library_child_container_id, image);
    }

    layout_context.add_child_to_parent(library_container_id, library_child_container_id);

    let main_area_container_id = ContainerBuilder::new()
        .with_debug_name("Main Area Container")
        .with_margin(Edges::horizontal(10.0))
        .with_direction(FlexDirection::Column)
        .build(
            &mut layout_context.world,
            &mut layout_context.z_index_manager,
        );

    let main_area_background_id = BackgroundBuilder::with_frosted_glass(FrostedGlassConfig {
        tint_color: Color::Black,
        blur_radius: 20.0,
        tint_intensity: 0.5,
    })
    .with_debug_name("Main Area Background")
    .with_border_radius(BorderRadius::all(5.0))
    .with_border(1.0, Color::Black)
    .with_fixed_position(Anchor::Center)
    .build(
        &mut layout_context.world,
        wgpu_ctx,
        &mut layout_context.z_index_manager,
    );

    layout_context.add_child_to_parent(main_area_container_id, main_area_background_id);

    let now_playing_container_id = ContainerBuilder::new()
        .with_debug_name("Now Playing Container")
        .with_size(FlexValue::Fixed(350.0), FlexValue::Fill)
        .with_margin(Edges::right(10.0))
        .with_direction(FlexDirection::Column)
        .build(
            &mut layout_context.world,
            &mut layout_context.z_index_manager,
        );

    let now_playing_background_id = BackgroundBuilder::with_frosted_glass(FrostedGlassConfig {
        tint_color: Color::Black,
        blur_radius: 20.0,
        tint_intensity: 0.5,
    })
    .with_debug_name("Now Playing Background")
    .with_border_radius(BorderRadius::all(5.0))
    .with_border(1.0, Color::Black)
    .with_fixed_position(Anchor::Center)
    .build(
        &mut layout_context.world,
        wgpu_ctx,
        &mut layout_context.z_index_manager,
    );

    layout_context.add_child_to_parent(now_playing_container_id, now_playing_background_id);

    // Add children to the app container
    layout_context.add_child_to_parent(app_container_id, library_container_id);
    layout_context.add_child_to_parent(app_container_id, main_area_container_id);
    layout_context.add_child_to_parent(app_container_id, now_playing_container_id);

    // Add children to the main container
    layout_context.add_child_to_parent(main_container_id, background_id);
    layout_context.add_child_to_parent(main_container_id, frosted_glass_id);
    layout_context.add_child_to_parent(main_container_id, nav_bar_container_id);
    layout_context.add_child_to_parent(main_container_id, app_container_id);
    layout_context.add_child_to_parent(main_container_id, player_container_id);
}

fn create_settings_modal(
    wgpu_ctx: &mut WgpuCtx,
    layout_context: &mut layout::LayoutContext,
) -> EntityId {
    ModalBuilder::new(NamedRef::SettingsModal)
        .with_debug_name("Settings Modal")
        .with_backdrop_frosted_glass(FrostedGlassConfig {
            tint_color: Color::Black,
            blur_radius: 10.0,
            tint_intensity: 0.9,
        })
        .with_border_radius(BorderRadius::all(10.0))
        .with_background_color(BackgroundColorConfig {
            color: Color::Black.lighten(0.001),
        })
        .with_shadow(Color::White, (0.0, 0.0), 10.0, 0.05)
        .with_backdrop_animation(AnimationConfig {
            duration: Duration::from_millis(300),
            direction: AnimationDirection::Forward,
            easing: EasingFunction::EaseOutCubic,
            animation_type: AnimationType::Opacity {
                range: AnimationRange::new(0.0, 1.0),
            },
            when: AnimationWhen::Entry,
        })
        .with_backdrop_animation(AnimationConfig {
            duration: Duration::from_millis(300),
            direction: AnimationDirection::Forward,
            easing: EasingFunction::EaseOutCubic,
            animation_type: AnimationType::Opacity {
                range: AnimationRange::new(1.0, 0.0),
            },
            when: AnimationWhen::Exit,
        })
        .with_animation(AnimationConfig {
            duration: Duration::from_millis(300),
            direction: AnimationDirection::Forward,
            easing: EasingFunction::EaseOutCubic,
            animation_type: AnimationType::Opacity {
                range: AnimationRange::new(0.0, 1.0),
            },
            when: AnimationWhen::Entry,
        })
        .with_animation(AnimationConfig {
            duration: Duration::from_millis(300),
            direction: AnimationDirection::Forward,
            easing: EasingFunction::EaseOutCubic,
            animation_type: AnimationType::Opacity {
                range: AnimationRange::new(1.0, 0.0),
            },
            when: AnimationWhen::Exit,
        })
        .build(layout_context, wgpu_ctx)
}

fn create_nav_bar(
    wgpu_ctx: &mut WgpuCtx,
    layout_context: &mut layout::LayoutContext,
    no_background: bool,
) -> EntityId {
    let nav_bar_container_id = ContainerBuilder::new()
        .with_debug_name("Nav Bar Container")
        .with_size(FlexValue::Fill, FlexValue::Fixed(64.0))
        .with_direction(FlexDirection::Row)
        .with_align_items(AlignItems::Center)
        .with_justify_content(JustifyContent::End)
        .with_padding(Edges::all(10.0))
        .with_drag_event(AppEvent::DragWindow)
        .build(
            &mut layout_context.world,
            &mut layout_context.z_index_manager,
        );

    if !no_background {
        let nav_bar_background_id = BackgroundBuilder::with_frosted_glass(FrostedGlassConfig {
            tint_color: Color::Black,
            blur_radius: 2.0,
            tint_intensity: 0.5,
        })
        .with_debug_name("Nav Bar Background")
        .with_border(1.0, Color::Black)
        .with_border_radius(BorderRadius::all(5.0))
        .with_fixed_position(Anchor::Center)
        .build(
            &mut layout_context.world,
            wgpu_ctx,
            &mut layout_context.z_index_manager,
        );
        layout_context.add_child_to_parent(nav_bar_container_id, nav_bar_background_id);
    }

    let nav_buttons_container_id = ContainerBuilder::new()
        .with_debug_name("Nav Buttons Container")
        .with_direction(FlexDirection::Row)
        .with_align_items(AlignItems::Center)
        .with_justify_content(JustifyContent::SpaceBetween)
        .with_width(FlexValue::Fixed(128.0))
        .with_margin(Edges::all(5.0))
        .build(
            &mut layout_context.world,
            &mut layout_context.z_index_manager,
        );

    // Settings button
    let settings_button_id = ButtonBuilder::new()
        .with_debug_name("Settings Button")
        .with_content_padding(Edges::all(5.0))
        .with_size(FlexValue::Fixed(30.0), FlexValue::Fixed(30.0))
        .with_border_radius(BorderRadius::all(4.0))
        .with_click_event(AppEvent::OpenModal(NamedRef::SettingsModal))
        .with_background_color(BackgroundColorConfig {
            color: Color::Transparent,
        })
        .with_foreground_image("settings.png")
        .with_animation(AnimationConfig {
            duration: Duration::from_millis(200),
            easing: EasingFunction::EaseOutExpo,
            direction: AnimationDirection::Alternate,
            animation_type: AnimationType::Color {
                range: AnimationRange::new(Color::Transparent, Color::DarkGray),
            },
            when: AnimationWhen::Hover,
        })
        .build(layout_context, wgpu_ctx);

    // Minimize button
    let minimize_button_id = ButtonBuilder::new()
        .with_debug_name("Minimize Button")
        .with_content_padding(Edges::all(5.0))
        .with_size(FlexValue::Fixed(30.0), FlexValue::Fixed(30.0))
        .with_border_radius(BorderRadius::all(4.0))
        .with_click_event(AppEvent::Minimize)
        .with_background_color(BackgroundColorConfig {
            color: Color::Transparent,
        })
        .with_foreground_image("minimize.png")
        .with_animation(AnimationConfig {
            duration: Duration::from_millis(200),
            easing: EasingFunction::EaseOutExpo,
            direction: AnimationDirection::Alternate,
            animation_type: AnimationType::Color {
                range: AnimationRange::new(Color::Transparent, Color::DarkGray),
            },
            when: AnimationWhen::Hover,
        })
        .build(layout_context, wgpu_ctx);

    // Maximize button
    let maximize_button_id = ButtonBuilder::new()
        .with_debug_name("Maximize Button")
        .with_content_padding(Edges::all(5.0))
        .with_size(FlexValue::Fixed(30.0), FlexValue::Fixed(30.0))
        .with_border_radius(BorderRadius::all(4.0))
        .with_click_event(AppEvent::Maximize)
        .with_background_color(BackgroundColorConfig {
            color: Color::Transparent,
        })
        .with_foreground_image("maximize.png")
        .with_animation(AnimationConfig {
            duration: Duration::from_millis(200),
            easing: EasingFunction::EaseOutExpo,
            direction: AnimationDirection::Alternate,
            animation_type: AnimationType::Color {
                range: AnimationRange::new(Color::Transparent, Color::DarkGray),
            },
            when: AnimationWhen::Hover,
        })
        .build(layout_context, wgpu_ctx);

    // Close button
    let close_button_id = ButtonBuilder::new()
        .with_debug_name("Close Button")
        .with_size(FlexValue::Fixed(30.0), FlexValue::Fixed(30.0))
        .with_content_padding(Edges::all(5.0))
        .with_border_radius(BorderRadius::all(4.0))
        .with_click_event(AppEvent::Close)
        .with_background_color(BackgroundColorConfig {
            color: Color::Transparent,
        })
        .with_foreground_image("close.png")
        .with_animation(AnimationConfig {
            duration: Duration::from_millis(200),
            easing: EasingFunction::EaseOutExpo,
            direction: AnimationDirection::Alternate,
            animation_type: AnimationType::Color {
                range: AnimationRange::new(Color::Transparent, Color::Red),
            },
            when: AnimationWhen::Hover,
        })
        .build(layout_context, wgpu_ctx);

    let settings_modal_id = create_settings_modal(wgpu_ctx, layout_context);

    // Add buttons to the nav buttons container
    layout_context.add_child_to_parent(nav_buttons_container_id, settings_button_id);
    layout_context.add_child_to_parent(nav_buttons_container_id, minimize_button_id);
    layout_context.add_child_to_parent(nav_buttons_container_id, maximize_button_id);
    layout_context.add_child_to_parent(nav_buttons_container_id, close_button_id);

    // add the nav buttons container to the nav bar container
    layout_context.add_child_to_parent(nav_bar_container_id, nav_buttons_container_id);
    layout_context.add_child_to_parent(nav_bar_container_id, settings_modal_id);

    nav_bar_container_id
}

fn create_player_bar(
    wgpu_ctx: &mut WgpuCtx,
    layout_context: &mut layout::LayoutContext,
) -> EntityId {
    let player_container_id = ContainerBuilder::new()
        .with_debug_name("Player Container")
        .with_size(FlexValue::Fill, FlexValue::Fixed(100.0))
        .with_direction(FlexDirection::Row)
        .with_align_items(AlignItems::Center)
        .with_justify_content(JustifyContent::SpaceBetween)
        .with_padding(Edges::all(10.0))
        .build(
            &mut layout_context.world,
            &mut layout_context.z_index_manager,
        );

    let player_container_background_id =
        BackgroundBuilder::with_frosted_glass(FrostedGlassConfig {
            tint_color: Color::Black,
            blur_radius: 20.0,
            tint_intensity: 0.5,
        })
        .with_debug_name("Player Container Background")
        .with_border_radius(BorderRadius::all(5.0))
        .with_border(1.0, Color::Black)
        .with_fixed_position(Anchor::Center)
        .build(
            &mut layout_context.world,
            wgpu_ctx,
            &mut layout_context.z_index_manager,
        );

    let song_info_container_id = ContainerBuilder::new()
        .with_debug_name("Song Info Container")
        .with_margin(Edges::all(5.0))
        .with_direction(FlexDirection::Row)
        .with_align_items(AlignItems::Center)
        .with_justify_content(JustifyContent::Start)
        .build(
            &mut layout_context.world,
            &mut layout_context.z_index_manager,
        );

    let current_song_album_art_id = ImageBuilder::new("album_art.png")
        .with_debug_name("Current Song Album Art")
        .with_z_index(1)
        .with_scale_mode(ScaleMode::ContainNoCenter)
        .with_shadow(Color::Black, (0.0, 0.0), 4.0, 0.4)
        .with_uniform_border_radius(5.0)
        .with_named_ref(NamedRef::CurrentSongAlbumArt)
        .build(
            &mut layout_context.world,
            wgpu_ctx,
            &mut layout_context.z_index_manager,
        );

    let current_song_info_id = TextBuilder::new()
        .with_debug_name("Current Song Info")
        .with_text("Song Name\nArtist Name")
        .with_margin(Edges::left(10.0))
        .with_color(Color::White)
        .set_fit_to_size()
        .with_z_index(1)
        .build(
            &mut layout_context.world,
            wgpu_ctx,
            &mut layout_context.z_index_manager,
        );

    layout_context.add_child_to_parent(song_info_container_id, current_song_album_art_id);
    layout_context.add_child_to_parent(song_info_container_id, current_song_info_id);

    let player_controls_container_id = ContainerBuilder::new()
        .with_debug_name("Player Controls Container")
        .with_size(FlexValue::Fraction(0.7), FlexValue::Fill)
        .with_margin(Edges::all(10.0))
        .with_fixed_position(Anchor::Center)
        .with_direction(FlexDirection::Column)
        .with_align_items(AlignItems::Center)
        .build(
            &mut layout_context.world,
            &mut layout_context.z_index_manager,
        );

    let player_control_buttons_container_id = ContainerBuilder::new()
        .with_debug_name("Player Controls Sub Container")
        .with_direction(FlexDirection::Row)
        .with_size(FlexValue::Fraction(0.5), FlexValue::Fraction(0.7))
        .with_justify_content(JustifyContent::SpaceAround)
        .with_align_items(AlignItems::Center)
        .build(
            &mut layout_context.world,
            &mut layout_context.z_index_manager,
        );

    let player_control_btns_size = 32.0;

    let shuffle_button_id = ButtonBuilder::new()
        .with_debug_name("Shuffle Button")
        .with_size(player_control_btns_size, player_control_btns_size)
        .with_click_event(AppEvent::Shuffle)
        .with_background_color(BackgroundColorConfig {
            color: Color::Transparent,
        })
        .with_content_padding(Edges::all(5.0))
        .with_border_radius(BorderRadius::all(5.0))
        .with_foreground_image("shuffle.png")
        .with_animation(AnimationConfig {
            duration: Duration::from_millis(200),
            easing: EasingFunction::EaseOutExpo,
            direction: AnimationDirection::Alternate,
            animation_type: AnimationType::Color {
                range: AnimationRange::new(Color::Transparent, Color::DarkGray),
            },
            when: AnimationWhen::Hover,
        })
        .build(layout_context, wgpu_ctx);

    let previous_button_id = ButtonBuilder::new()
        .with_debug_name("Previous Button")
        .with_size(player_control_btns_size, player_control_btns_size)
        .with_click_event(AppEvent::PreviousTrack)
        .with_background_color(BackgroundColorConfig {
            color: Color::Transparent,
        })
        .with_content_padding(Edges::all(5.0))
        .with_border_radius(BorderRadius::all(5.0))
        .with_foreground_image("skip-back.png")
        .with_animation(AnimationConfig {
            duration: Duration::from_millis(200),
            easing: EasingFunction::EaseOutExpo,
            direction: AnimationDirection::Alternate,
            animation_type: AnimationType::Color {
                range: AnimationRange::new(Color::Transparent, Color::DarkGray),
            },
            when: AnimationWhen::Hover,
        })
        .build(layout_context, wgpu_ctx);

    let play_button_id = ButtonBuilder::new()
        .with_debug_name("Play Button")
        .with_size(player_control_btns_size, player_control_btns_size)
        .with_click_event(AppEvent::PlayPause)
        .with_background_color(BackgroundColorConfig {
            color: Color::Transparent,
        })
        .with_content_padding(Edges::all(5.0))
        .with_border_radius(BorderRadius::all(5.0))
        .with_foreground_image("play.png")
        .with_animation(AnimationConfig {
            duration: Duration::from_millis(200),
            easing: EasingFunction::EaseOutExpo,
            direction: AnimationDirection::Alternate,
            animation_type: AnimationType::Color {
                range: AnimationRange::new(Color::Transparent, Color::DarkGray),
            },
            when: AnimationWhen::Hover,
        })
        .build(layout_context, wgpu_ctx);

    let next_button_id = ButtonBuilder::new()
        .with_debug_name("Next Button")
        .with_size(player_control_btns_size, player_control_btns_size)
        .with_click_event(AppEvent::NextTrack)
        .with_background_color(BackgroundColorConfig {
            color: Color::Transparent,
        })
        .with_content_padding(Edges::all(5.0))
        .with_border_radius(BorderRadius::all(5.0))
        .with_foreground_image("skip-forward.png")
        .with_animation(AnimationConfig {
            duration: Duration::from_millis(200),
            easing: EasingFunction::EaseOutExpo,
            direction: AnimationDirection::Alternate,
            animation_type: AnimationType::Color {
                range: AnimationRange::new(Color::Transparent, Color::DarkGray),
            },
            when: AnimationWhen::Hover,
        })
        .build(layout_context, wgpu_ctx);

    let repeat_button_id = ButtonBuilder::new()
        .with_debug_name("Repeat Button")
        .with_size(player_control_btns_size, player_control_btns_size)
        .with_click_event(AppEvent::Repeat)
        .with_background_color(BackgroundColorConfig {
            color: Color::Transparent,
        })
        .with_content_padding(Edges::all(5.0))
        .with_border_radius(BorderRadius::all(5.0))
        .with_foreground_image("repeat.png")
        .with_animation(AnimationConfig {
            duration: Duration::from_millis(200),
            easing: EasingFunction::EaseOutExpo,
            direction: AnimationDirection::Alternate,
            animation_type: AnimationType::Color {
                range: AnimationRange::new(Color::Transparent, Color::DarkGray),
            },
            when: AnimationWhen::Hover,
        })
        .build(layout_context, wgpu_ctx);

    layout_context.add_child_to_parent(player_control_buttons_container_id, shuffle_button_id);
    layout_context.add_child_to_parent(player_control_buttons_container_id, previous_button_id);
    layout_context.add_child_to_parent(player_control_buttons_container_id, play_button_id);
    layout_context.add_child_to_parent(player_control_buttons_container_id, next_button_id);
    layout_context.add_child_to_parent(player_control_buttons_container_id, repeat_button_id);

    layout_context.add_child_to_parent(
        player_controls_container_id,
        player_control_buttons_container_id,
    );

    let song_progress_container_id = ContainerBuilder::new()
        .with_debug_name("Song Progress Container")
        .with_height(FlexValue::Fixed(16.0))
        .with_align_items(AlignItems::Center)
        .build(
            &mut layout_context.world,
            &mut layout_context.z_index_manager,
        );

    // TODO: Temporarily use a color as multiple text components have weird behavior for now
    let song_start_time_id = BackgroundBuilder::with_color(BackgroundColorConfig {
        color: Color::DarkGray,
    })
    .with_debug_name("Song Progress Start Time")
    .with_border_radius(BorderRadius::all(5.0))
    .with_size(50, 16)
    .build(
        &mut layout_context.world,
        wgpu_ctx,
        &mut layout_context.z_index_manager,
    );

    let song_end_time_id = BackgroundBuilder::with_color(BackgroundColorConfig {
        color: Color::DarkGray,
    })
    .with_debug_name("Song Progress End Time")
    .with_border_radius(BorderRadius::all(5.0))
    .with_size(50, 16)
    .build(
        &mut layout_context.world,
        wgpu_ctx,
        &mut layout_context.z_index_manager,
    );

    let song_progress_slider_id = BackgroundBuilder::with_color(BackgroundColorConfig {
        color: Color::DarkGray,
    })
    .with_debug_name("Song Progress Slider")
    .with_border_radius(BorderRadius::all(999.0))
    .with_height(4)
    .with_margin(Edges::horizontal(10.0))
    .build(
        &mut layout_context.world,
        wgpu_ctx,
        &mut layout_context.z_index_manager,
    );

    layout_context.add_child_to_parent(song_progress_container_id, song_start_time_id);
    layout_context.add_child_to_parent(song_progress_container_id, song_progress_slider_id);
    layout_context.add_child_to_parent(song_progress_container_id, song_end_time_id);

    layout_context.add_child_to_parent(player_controls_container_id, song_progress_container_id);

    let volume_slider_container_id = ContainerBuilder::new()
        .with_debug_name("Volume Slider Container")
        .with_direction(FlexDirection::Row)
        .with_margin(Edges::all(5.0))
        .with_align_items(AlignItems::Center)
        .with_justify_content(JustifyContent::End)
        .build(
            &mut layout_context.world,
            &mut layout_context.z_index_manager,
        );

    let volume_slider_id = BackgroundBuilder::with_color(BackgroundColorConfig {
        color: Color::DarkGray,
    })
    .with_debug_name("Volume Slider")
    .with_border_radius(BorderRadius::all(999.0))
    .with_size(80, 4)
    .with_margin(Edges::custom(0.0, 10.0, 0.0, 5.0))
    .build(
        &mut layout_context.world,
        wgpu_ctx,
        &mut layout_context.z_index_manager,
    );

    let volume_icon_id = ImageBuilder::new("volume.png")
        .with_debug_name("Volume Icon")
        .with_size(16, 16)
        .with_margin(Edges::left(5.0))
        .with_scale_mode(ScaleMode::Contain)
        .build(
            &mut layout_context.world,
            wgpu_ctx,
            &mut layout_context.z_index_manager,
        );

    layout_context.add_child_to_parent(volume_slider_container_id, volume_icon_id);
    layout_context.add_child_to_parent(volume_slider_container_id, volume_slider_id);

    layout_context.add_child_to_parent(player_container_id, player_container_background_id);
    layout_context.add_child_to_parent(player_container_id, song_info_container_id);
    layout_context.add_child_to_parent(player_container_id, player_controls_container_id);
    layout_context.add_child_to_parent(player_container_id, volume_slider_container_id);

    player_container_id
}
