use crate::{
    app::AppEvent,
    ui::layout::{FlexDirection, FlexValue},
    wgpu_ctx::WgpuCtx,
};
use animation::{
    AnimationConfig, AnimationDirection, AnimationType, AnimationWhen, EasingFunction,
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

pub mod animation;
pub mod asset;
pub mod color;
pub mod ecs;
pub mod img_utils;
pub mod layout;
pub mod text_renderer;
pub mod z_index_manager;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum UiView {
    #[default]
    Login,
    Home,
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

    let login_background_id = BackgroundBuilder::with_gradient(BackgroundGradientConfig {
        color_stops: vec![
            GradientColorStop {
                color: Color::Crimson,
                position: 0.0,
            },
            GradientColorStop {
                color: Color::MidnightBlue,
                position: 1.0,
            },
        ],
        angle: 90.0,
        gradient_type: GradientType::Linear,
        center: None,
        radius: None,
    })
    .with_debug_name("Login Page Background")
    .with_fixed_position(Anchor::Center)
    .build(
        &mut layout_context.world,
        wgpu_ctx,
        &mut layout_context.z_index_manager,
    );

    let nav_bar_container_id = create_nav_bar(wgpu_ctx, layout_context);

    layout_context
        .world
        .add_child_to_parent(main_container_id, login_background_id);
    layout_context
        .world
        .add_child_to_parent(main_container_id, nav_bar_container_id);

    let sub_container_id = ContainerBuilder::new()
        .with_debug_name("Login Page Sub Container")
        .with_direction(FlexDirection::Column)
        .with_align_items(AlignItems::Center)
        .with_justify_content(JustifyContent::Center)
        .build(
            &mut layout_context.world,
            &mut layout_context.z_index_manager,
        );

    // Welcome label
    let welcome_text_id = TextBuilder::new()
        .with_debug_name("Welcome Text")
        .with_text("Welcome to Frostify".to_string())
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
        .with_scale_mode(ScaleMode::Contain)
        .with_size(100, 100)
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
            opacity: 1.0,
            tint_intensity: 0.5,
        })
        .with_animation(AnimationConfig {
            duration: Duration::from_millis(200),
            easing: EasingFunction::EaseOutExpo,
            direction: AnimationDirection::Alternate,
            animation_type: AnimationType::FrostedGlassTint {
                from: Color::Black,
                to: Color::Crimson,
            },
            when: AnimationWhen::Hover,
        })
        .with_text("Login with Spotify".to_string())
        .with_text_color(Color::White)
        .with_click_event(AppEvent::Login)
        .with_animation(AnimationConfig {
            duration: Duration::from_millis(200),
            easing: EasingFunction::EaseOutExpo,
            direction: AnimationDirection::Alternate,
            animation_type: AnimationType::Scale {
                from: 1.0,
                to: 1.05,
                anchor: Anchor::Center,
            },
            when: AnimationWhen::Hover,
        })
        .build(
            &mut layout_context.world,
            wgpu_ctx,
            &mut layout_context.z_index_manager,
        );

    layout_context
        .world
        .add_child_to_parent(sub_container_id, welcome_text_id);
    layout_context
        .world
        .add_child_to_parent(sub_container_id, logo_id);
    layout_context
        .world
        .add_child_to_parent(sub_container_id, login_button_id);

    layout_context
        .world
        .add_child_to_parent(main_container_id, sub_container_id);
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

    let background_id = ImageBuilder::new("test.png")
        .with_scale_mode(ScaleMode::Cover)
        .with_debug_name("Background")
        .with_fixed_position(Anchor::Center)
        .build(
            &mut layout_context.world,
            wgpu_ctx,
            &mut layout_context.z_index_manager,
        );

    let frosted_glass_id = BackgroundBuilder::with_frosted_glass(FrostedGlassConfig {
        tint_color: Color::Black,
        blur_radius: 20.0,
        opacity: 1.0,
        tint_intensity: 0.5,
    })
    .with_debug_name("Frosted Glass")
    .with_fixed_position(Anchor::Center)
    .with_z_index(1)
    .build(
        &mut layout_context.world,
        wgpu_ctx,
        &mut layout_context.z_index_manager,
    );

    // Create nav bar using the extracted function
    let nav_bar_container_id = create_nav_bar(wgpu_ctx, layout_context);
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
        .build(
            &mut layout_context.world,
            &mut layout_context.z_index_manager,
        );

    let library_background_id = BackgroundBuilder::with_frosted_glass(FrostedGlassConfig {
        tint_color: Color::Black,
        blur_radius: 20.0,
        opacity: 1.0,
        tint_intensity: 0.5,
    })
    .with_debug_name("Library Background")
    .with_border_radius(BorderRadius::all(5.0))
    .with_border(1.0, Color::DarkGray.darken(0.05))
    .with_z_index(-1)
    .with_fixed_position(Anchor::Center)
    .build(
        &mut layout_context.world,
        wgpu_ctx,
        &mut layout_context.z_index_manager,
    );

    layout_context
        .world
        .add_child_to_parent(library_container_id, library_background_id);

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
            .with_scale_mode(ScaleMode::Contain)
            .with_debug_name(format!("Album Art {}", i))
            .with_size(FlexValue::Fixed(70.0), FlexValue::Fixed(70.0))
            .with_border_radius(BorderRadius::all(5.0))
            .with_shadow(Color::Black, (0.0, 0.0), 4.0, 0.4)
            .with_animation(AnimationConfig {
                duration: Duration::from_millis(150),
                direction: AnimationDirection::Alternate,
                easing: EasingFunction::EaseOutQuart,
                animation_type: AnimationType::Scale {
                    from: 1.0,
                    to: 1.5,
                    anchor: Anchor::Left,
                },
                when: AnimationWhen::Hover,
            })
            .with_clipping(true);

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

        layout_context
            .world
            .add_child_to_parent(library_child_container_id, image);
    }

    layout_context
        .world
        .add_child_to_parent(library_container_id, library_child_container_id);

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
        opacity: 1.0,
        tint_intensity: 0.5,
    })
    .with_debug_name("Main Area Background")
    .with_border_radius(BorderRadius::all(5.0))
    .with_border(1.0, Color::DarkGray.darken(0.05))
    .with_fixed_position(Anchor::Center)
    .build(
        &mut layout_context.world,
        wgpu_ctx,
        &mut layout_context.z_index_manager,
    );

    layout_context
        .world
        .add_child_to_parent(main_area_container_id, main_area_background_id);

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
        opacity: 1.0,
        tint_intensity: 0.5,
    })
    .with_debug_name("Now Playing Background")
    .with_border_radius(BorderRadius::all(5.0))
    .with_border(1.0, Color::DarkGray.darken(0.05))
    .with_fixed_position(Anchor::Center)
    .build(
        &mut layout_context.world,
        wgpu_ctx,
        &mut layout_context.z_index_manager,
    );

    layout_context
        .world
        .add_child_to_parent(now_playing_container_id, now_playing_background_id);

    // Add children to the app container
    layout_context
        .world
        .add_child_to_parent(app_container_id, library_container_id);
    layout_context
        .world
        .add_child_to_parent(app_container_id, main_area_container_id);
    layout_context
        .world
        .add_child_to_parent(app_container_id, now_playing_container_id);

    // Add children to the main container
    layout_context
        .world
        .add_child_to_parent(main_container_id, background_id);
    layout_context
        .world
        .add_child_to_parent(main_container_id, frosted_glass_id);
    layout_context
        .world
        .add_child_to_parent(main_container_id, nav_bar_container_id);
    layout_context
        .world
        .add_child_to_parent(main_container_id, app_container_id);
    layout_context
        .world
        .add_child_to_parent(main_container_id, player_container_id);
}

fn create_nav_bar(wgpu_ctx: &mut WgpuCtx, layout_context: &mut layout::LayoutContext) -> EntityId {
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

    let nav_bar_background_id = BackgroundBuilder::with_frosted_glass(FrostedGlassConfig {
        tint_color: Color::Black,
        blur_radius: 2.0,
        opacity: 1.0,
        tint_intensity: 0.5,
    })
    .with_debug_name("Nav Bar Background")
    .with_border(1.0, Color::DarkGray.darken(0.05))
    .with_border_radius(BorderRadius::all(5.0))
    .with_fixed_position(Anchor::Center)
    .build(
        &mut layout_context.world,
        wgpu_ctx,
        &mut layout_context.z_index_manager,
    );

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
                from: Color::Transparent,
                to: Color::DarkGray,
            },
            when: AnimationWhen::Hover,
        })
        .build(
            &mut layout_context.world,
            wgpu_ctx,
            &mut layout_context.z_index_manager,
        );

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
                from: Color::Transparent,
                to: Color::DarkGray,
            },
            when: AnimationWhen::Hover,
        })
        .build(
            &mut layout_context.world,
            wgpu_ctx,
            &mut layout_context.z_index_manager,
        );

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
                from: Color::Transparent,
                to: Color::Red,
            },
            when: AnimationWhen::Hover,
        })
        .build(
            &mut layout_context.world,
            wgpu_ctx,
            &mut layout_context.z_index_manager,
        );

    // Add buttons to the nav buttons container
    layout_context
        .world
        .add_child_to_parent(nav_buttons_container_id, minimize_button_id);
    layout_context
        .world
        .add_child_to_parent(nav_buttons_container_id, maximize_button_id);
    layout_context
        .world
        .add_child_to_parent(nav_buttons_container_id, close_button_id);

    // add the nav buttons container to the nav bar container
    layout_context
        .world
        .add_child_to_parent(nav_bar_container_id, nav_bar_background_id);
    layout_context
        .world
        .add_child_to_parent(nav_bar_container_id, nav_buttons_container_id);

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
            opacity: 1.0,
            tint_intensity: 0.5,
        })
        .with_debug_name("Player Container Background")
        .with_border_radius(BorderRadius::all(5.0))
        .with_border(1.0, Color::DarkGray.darken(0.05))
        .with_fixed_position(Anchor::Center)
        .build(
            &mut layout_context.world,
            wgpu_ctx,
            &mut layout_context.z_index_manager,
        );

    let current_song_album_art_id = ImageBuilder::new("album_art.png")
        .with_debug_name("Current Song Album Art")
        .with_z_index(1)
        .with_margin(Edges::all(10.0))
        .with_uniform_border_radius(5.0)
        .set_fit_to_size()
        .build(
            &mut layout_context.world,
            wgpu_ctx,
            &mut layout_context.z_index_manager,
        );

    let current_song_info_id = TextBuilder::new()
        .with_debug_name("Current Song Info")
        .with_text("Song Name\n\nArtist Name".to_string())
        .with_color(Color::White)
        .set_fit_to_size()
        .with_z_index(1)
        .build(
            &mut layout_context.world,
            wgpu_ctx,
            &mut layout_context.z_index_manager,
        );

    let song_info_container_id = ContainerBuilder::new()
        .with_debug_name("Song Info Container")
        .with_direction(FlexDirection::Row)
        .with_align_items(AlignItems::Center)
        .with_justify_content(JustifyContent::Start)
        .build(
            &mut layout_context.world,
            &mut layout_context.z_index_manager,
        );

    layout_context
        .world
        .add_child_to_parent(song_info_container_id, current_song_album_art_id);
    layout_context
        .world
        .add_child_to_parent(song_info_container_id, current_song_info_id);

    let player_controls_container_id = ContainerBuilder::new()
        .with_debug_name("Player Controls Container")
        .with_size(FlexValue::Fraction(0.6), FlexValue::Fill)
        .with_fixed_position(Anchor::Center)
        .with_direction(FlexDirection::Column)
        .with_align_items(AlignItems::Center)
        .with_justify_content(JustifyContent::Center)
        .build(
            &mut layout_context.world,
            &mut layout_context.z_index_manager,
        );

    let player_controls_sub_container_id = ContainerBuilder::new()
        .with_debug_name("Player Controls Sub Container")
        .with_direction(FlexDirection::Row)
        .with_align_items(AlignItems::Center)
        .with_justify_content(JustifyContent::Center)
        .with_padding(Edges::horizontal(20.0))
        .with_margin(Edges::top(10.0))
        .build(
            &mut layout_context.world,
            &mut layout_context.z_index_manager,
        );

    let player_control_btns_size = 20.0;

    let shuffle_button_id = ButtonBuilder::new()
        .with_debug_name("Shuffle Button")
        .with_size(player_control_btns_size, player_control_btns_size)
        .with_click_event(AppEvent::Shuffle)
        .with_background_color(BackgroundColorConfig {
            color: Color::Transparent,
        })
        .with_margin(Edges::right(10.0))
        .with_border_radius(BorderRadius::all(999.0))
        .with_foreground_image("shuffle.png")
        .with_animation(AnimationConfig {
            duration: Duration::from_millis(200),
            easing: EasingFunction::EaseOutExpo,
            direction: AnimationDirection::Alternate,
            animation_type: AnimationType::Color {
                from: Color::Transparent,
                to: Color::DarkGray,
            },
            when: AnimationWhen::Hover,
        })
        .build(
            &mut layout_context.world,
            wgpu_ctx,
            &mut layout_context.z_index_manager,
        );

    let previous_button_id = ButtonBuilder::new()
        .with_debug_name("Previous Button")
        .with_size(player_control_btns_size, player_control_btns_size)
        .with_click_event(AppEvent::PreviousTrack)
        .with_background_color(BackgroundColorConfig {
            color: Color::Transparent,
        })
        .with_margin(Edges::horizontal(10.0))
        .with_border_radius(BorderRadius::all(999.0))
        .with_foreground_image("skip-back.png")
        .with_animation(AnimationConfig {
            duration: Duration::from_millis(200),
            easing: EasingFunction::EaseOutExpo,
            direction: AnimationDirection::Alternate,
            animation_type: AnimationType::Color {
                from: Color::Transparent,
                to: Color::DarkGray,
            },
            when: AnimationWhen::Hover,
        })
        .build(
            &mut layout_context.world,
            wgpu_ctx,
            &mut layout_context.z_index_manager,
        );

    let play_button_id = ButtonBuilder::new()
        .with_debug_name("Play Button")
        .with_size(player_control_btns_size, player_control_btns_size)
        .with_click_event(AppEvent::PlayPause)
        .with_background_color(BackgroundColorConfig {
            color: Color::Transparent,
        })
        .with_margin(Edges::horizontal(10.0))
        .with_border_radius(BorderRadius::all(999.0))
        .with_foreground_image("play.png")
        .with_animation(AnimationConfig {
            duration: Duration::from_millis(200),
            easing: EasingFunction::EaseOutExpo,
            direction: AnimationDirection::Alternate,
            animation_type: AnimationType::Color {
                from: Color::Transparent,
                to: Color::DarkGray,
            },
            when: AnimationWhen::Hover,
        })
        .build(
            &mut layout_context.world,
            wgpu_ctx,
            &mut layout_context.z_index_manager,
        );

    let next_button_id = ButtonBuilder::new()
        .with_debug_name("Next Button")
        .with_size(player_control_btns_size, player_control_btns_size)
        .with_click_event(AppEvent::NextTrack)
        .with_background_color(BackgroundColorConfig {
            color: Color::Transparent,
        })
        .with_margin(Edges::horizontal(10.0))
        .with_border_radius(BorderRadius::all(999.0))
        .with_foreground_image("skip-forward.png")
        .with_animation(AnimationConfig {
            duration: Duration::from_millis(200),
            easing: EasingFunction::EaseOutExpo,
            direction: AnimationDirection::Alternate,
            animation_type: AnimationType::Color {
                from: Color::Transparent,
                to: Color::DarkGray,
            },
            when: AnimationWhen::Hover,
        })
        .build(
            &mut layout_context.world,
            wgpu_ctx,
            &mut layout_context.z_index_manager,
        );

    let repeat_button_id = ButtonBuilder::new()
        .with_debug_name("Repeat Button")
        .with_size(player_control_btns_size, player_control_btns_size)
        .with_click_event(AppEvent::Repeat)
        .with_background_color(BackgroundColorConfig {
            color: Color::Transparent,
        })
        .with_margin(Edges::left(10.0))
        .with_border_radius(BorderRadius::all(999.0))
        .with_foreground_image("repeat.png")
        .with_animation(AnimationConfig {
            duration: Duration::from_millis(200),
            easing: EasingFunction::EaseOutExpo,
            direction: AnimationDirection::Alternate,
            animation_type: AnimationType::Color {
                from: Color::Transparent,
                to: Color::DarkGray,
            },
            when: AnimationWhen::Hover,
        })
        .build(
            &mut layout_context.world,
            wgpu_ctx,
            &mut layout_context.z_index_manager,
        );

    layout_context
        .world
        .add_child_to_parent(player_controls_sub_container_id, shuffle_button_id);
    layout_context
        .world
        .add_child_to_parent(player_controls_sub_container_id, previous_button_id);
    layout_context
        .world
        .add_child_to_parent(player_controls_sub_container_id, play_button_id);
    layout_context
        .world
        .add_child_to_parent(player_controls_sub_container_id, next_button_id);
    layout_context
        .world
        .add_child_to_parent(player_controls_sub_container_id, repeat_button_id);

    layout_context.world.add_child_to_parent(
        player_controls_container_id,
        player_controls_sub_container_id,
    );

    let temp_song_progress_slider_id = BackgroundBuilder::with_color(BackgroundColorConfig {
        color: Color::DarkGray,
    })
    .with_debug_name("Song Progress Slider")
    .with_border_radius(BorderRadius::all(999.0))
    .with_height(4)
    .build(
        &mut layout_context.world,
        wgpu_ctx,
        &mut layout_context.z_index_manager,
    );

    layout_context
        .world
        .add_child_to_parent(player_controls_container_id, temp_song_progress_slider_id);

    let temp_volume_slider_id = BackgroundBuilder::with_color(BackgroundColorConfig {
        color: Color::DarkGray,
    })
    .with_debug_name("Volume Slider")
    .with_border_radius(BorderRadius::all(999.0))
    .with_height(4)
    .build(
        &mut layout_context.world,
        wgpu_ctx,
        &mut layout_context.z_index_manager,
    );

    layout_context
        .world
        .add_child_to_parent(player_container_id, player_container_background_id);

    layout_context
        .world
        .add_child_to_parent(player_container_id, player_container_background_id);
    layout_context
        .world
        .add_child_to_parent(player_container_id, song_info_container_id);
    layout_context
        .world
        .add_child_to_parent(player_container_id, player_controls_container_id);
    layout_context
        .world
        .add_child_to_parent(player_container_id, temp_volume_slider_id);

    player_container_id
}
