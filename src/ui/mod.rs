use crate::{
    app::AppEvent,
    ui::{
        animation::{
            AnimationConfig, AnimationDirection, AnimationType, AnimationWhen, EasingFunction,
        },
        color::Color,
        component::{
            Component, ComponentConfig, ComponentMetaData, GradientColorStop, ImageConfig,
            TextConfig,
        },
        components::{
            background::BackgroundBuilder,
            button::{ButtonBuilder, ButtonSubComponent},
            component_builder::ComponentBuilder,
            container::FlexContainerBuilder,
            image::{ImageBuilder, ScaleMode},
            label::LabelBuilder,
        },
        layout::{
            AlignItems, Anchor, BorderRadius, Bounds, Edges, FlexDirection, FlexValue,
            JustifyContent, Position,
        },
    },
    wgpu_ctx::{AppPipelines, WgpuCtx},
};
use components::slider::SliderBuilder;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;

pub mod animation;
pub mod asset;
pub mod color;
pub mod component;
pub mod component_update;
pub mod components;
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

pub trait Configurable {
    fn configure(
        component: &mut Component,
        config: ComponentConfig,
        wgpu_ctx: &mut WgpuCtx,
    ) -> Vec<ComponentMetaData>;
}

pub trait Renderable {
    fn draw(
        component: &mut Component,
        render_pass: &mut wgpu::RenderPass,
        app_pipelines: &mut AppPipelines,
    );
}

pub trait Positionable {
    fn set_position(component: &mut Component, wgpu_ctx: &mut WgpuCtx, bounds: Bounds);
}

pub fn create_login_ui(
    wgpu_ctx: &mut WgpuCtx,
    event_tx: UnboundedSender<AppEvent>,
    layout_context: &mut layout::LayoutContext,
) {
    let mut main_container = FlexContainerBuilder::new()
        .with_debug_name("Main Container")
        .with_direction(FlexDirection::Column)
        .build(wgpu_ctx);

    // Background
    let background = BackgroundBuilder::with_linear_gradient(
        vec![
            GradientColorStop {
                color: Color::Crimson,
                position: 0.0,
            },
            GradientColorStop {
                color: Color::MidnightBlue,
                position: 1.0,
            },
        ],
        90.0,
    )
    .with_debug_name("Background")
    .with_fixed_position(Anchor::Center)
    .build(wgpu_ctx);

    let nav_bar_container = create_nav_bar(wgpu_ctx, event_tx.clone());

    main_container.add_child(nav_bar_container);
    main_container.add_child(background);

    let mut sub_container = FlexContainerBuilder::new()
        .with_debug_name("Sub Container")
        .with_direction(FlexDirection::Column)
        .with_align_items(AlignItems::Center)
        .with_justify_content(JustifyContent::Center)
        .build(wgpu_ctx);

    // Welcome label
    let welcome_label = ButtonBuilder::new()
        .with_sub_component(ButtonSubComponent::Text(TextConfig {
            text: "Welcome to Frostify".to_string(),
            font_size: 24.0,
            color: Color::White,
            line_height: 1.0,
        }))
        .with_size(FlexValue::Fill, FlexValue::Fixed(30.0))
        .set_fit_to_size()
        .with_debug_name("Welcome Label")
        .build(wgpu_ctx);

    // Logo
    let logo = ImageBuilder::new("frostify_logo.png")
        .with_scale_mode(ScaleMode::Contain)
        .with_size(FlexValue::Fixed(100.0), FlexValue::Fixed(100.0))
        .with_margin(Edges::vertical(10.0))
        .with_debug_name("Logo")
        .build(wgpu_ctx);

    // Login button
    let login_button = ButtonBuilder::new()
        .with_sub_component(ButtonSubComponent::FrostedGlass {
            tint_color: Color::Black,
            blur_radius: 5.0,
            opacity: 1.0,
            tint_intensity: 0.5,
        })
        .with_sub_component(ButtonSubComponent::Text(TextConfig {
            text: "Login with Spotify".to_string(),
            font_size: 16.0,
            color: Color::White,
            line_height: 1.0,
        }))
        .with_border_radius(BorderRadius::all(5.0))
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
        // .with_animation(AnimationConfig {
        //     duration: Duration::from_millis(200),
        //     easing: EasingFunction::EaseOutExpo,
        //     direction: AnimationDirection::Alternate,
        //     animation_type: AnimationType::Scale {
        //         from: 1.0,
        //         to: 1.05,
        //     },
        //     when: AnimationWhen::Hover,
        // })
        .with_border(1.0, Color::White)
        .with_margin(Edges::all(10.0))
        .with_size(200.0, 50.0)
        .with_debug_name("Login Button")
        .with_click_event(AppEvent::Login)
        .with_event_sender(event_tx.clone())
        .build(wgpu_ctx);

    sub_container.add_child(welcome_label);
    sub_container.add_child(logo);
    sub_container.add_child(login_button);

    main_container.add_child(sub_container);

    layout_context.add_component(main_container);
}

pub fn create_app_ui(
    wgpu_ctx: &mut WgpuCtx,
    event_tx: UnboundedSender<AppEvent>,
    layout_context: &mut layout::LayoutContext,
) {
    // Main container
    let mut main_container = FlexContainerBuilder::new()
        .with_debug_name("Main Container")
        .with_direction(FlexDirection::Column)
        .with_size(FlexValue::Fill, FlexValue::Fill)
        .build(wgpu_ctx);

    // Background
    let background = ImageBuilder::new("test.png")
        .with_scale_mode(ScaleMode::Cover)
        .with_debug_name("Background")
        .with_fixed_position(Anchor::Center)
        .build(wgpu_ctx);

    let frosted_glass = BackgroundBuilder::with_frosted_glass(Color::Black, 20.0, 1.0, 0.5)
        .with_debug_name("Frosted Glass")
        .with_position(Position::Absolute(Anchor::Center))
        .with_z_index(1)
        .build(wgpu_ctx);

    // Create nav bar using the extracted function
    let nav_bar_container = create_nav_bar(wgpu_ctx, event_tx.clone());
    let player_container = create_player_bar(wgpu_ctx, event_tx.clone());
    let mut app_container = FlexContainerBuilder::new()
        .with_debug_name("App Container")
        .with_size(FlexValue::Fill, FlexValue::Fill)
        .with_direction(FlexDirection::Row)
        .build(wgpu_ctx);

    let mut library_container = FlexContainerBuilder::new()
        .with_debug_name("Library Container")
        .with_size(FlexValue::Fixed(80.0), FlexValue::Fill)
        .with_margin(Edges::left(10.0))
        .with_direction(FlexDirection::Column)
        .build(wgpu_ctx);

    let library_background = BackgroundBuilder::with_frosted_glass(Color::Black, 2000.0, 1.0, 0.5)
        .with_debug_name("Library Background")
        .with_border_radius(BorderRadius::all(5.0))
        .with_border(1.0, Color::DarkGray.darken(0.05))
        .with_z_index(-1)
        .with_fixed_position(Anchor::Center)
        .build(wgpu_ctx);

    library_container.add_child(library_background);

    let mut library_child_container = FlexContainerBuilder::new()
        .with_debug_name("Library Child Container")
        .with_direction(FlexDirection::Column)
        .with_margin(Edges::all(5.0))
        .with_vertical_scroll()
        .build(wgpu_ctx);

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
        let image = image_builder.build(wgpu_ctx);
        library_child_container.add_child(image);
    }

    library_container.add_child(library_child_container);

    let mut main_area_container = FlexContainerBuilder::new()
        .with_debug_name("Main Area Container")
        .with_margin(Edges::horizontal(10.0))
        .with_direction(FlexDirection::Column)
        .build(wgpu_ctx);

    let main_area_background = BackgroundBuilder::with_frosted_glass(Color::Black, 20.0, 1.0, 0.5)
        .with_debug_name("Main Area Background")
        .with_border_radius(BorderRadius::all(5.0))
        .with_border(1.0, Color::DarkGray.darken(0.05))
        .with_fixed_position(Anchor::Center)
        .build(wgpu_ctx);

    main_area_container.add_child(main_area_background);

    let mut now_playing_container = FlexContainerBuilder::new()
        .with_debug_name("Now Playing Container")
        .with_size(FlexValue::Fixed(350.0), FlexValue::Fill)
        .with_margin(Edges::right(10.0))
        .with_direction(FlexDirection::Column)
        .build(wgpu_ctx);

    let now_playing_background =
        BackgroundBuilder::with_frosted_glass(Color::Black, 20.0, 1.0, 0.5)
            .with_debug_name("Now Playing Background")
            .with_border_radius(BorderRadius::all(5.0))
            .with_border(1.0, Color::DarkGray.darken(0.05))
            .with_fixed_position(Anchor::Center)
            .build(wgpu_ctx);

    now_playing_container.add_child(now_playing_background);

    app_container.add_child(library_container);
    app_container.add_child(main_area_container);
    app_container.add_child(now_playing_container);

    // Add children to the main container
    main_container.add_child(background);
    main_container.add_child(frosted_glass);
    main_container.add_child(nav_bar_container);
    main_container.add_child(app_container);
    main_container.add_child(player_container);

    // main_container.add_child(border_demo_container);

    // Add components in the correct order
    layout_context.add_component(main_container);
}

fn create_nav_bar(wgpu_ctx: &mut WgpuCtx, event_tx: UnboundedSender<AppEvent>) -> Component {
    // Nav bar container
    let mut nav_bar_container = FlexContainerBuilder::new()
        .with_debug_name("Nav Bar Container")
        .with_size(FlexValue::Fill, FlexValue::Fixed(64.0))
        .with_direction(FlexDirection::Row)
        .with_align_items(AlignItems::Center)
        .with_justify_content(JustifyContent::End)
        .with_padding(Edges::all(10.0))
        .build(wgpu_ctx);

    let nav_bar_background = BackgroundBuilder::with_frosted_glass(Color::Black, 20.0, 1.0, 0.5)
        .with_debug_name("Nav Bar Background")
        .with_border_radius(BorderRadius::all(5.0))
        .with_border(1.0, Color::DarkGray.darken(0.05))
        .with_fixed_position(Anchor::Center)
        .with_drag_event(AppEvent::DragWindow)
        .with_event_sender(event_tx.clone())
        .build(wgpu_ctx);

    // Nav buttons container
    let mut nav_buttons_container = FlexContainerBuilder::new()
        .with_debug_name("Nav Buttons Container")
        .with_direction(FlexDirection::Row)
        .with_align_items(AlignItems::Center)
        .with_justify_content(JustifyContent::SpaceBetween)
        .with_width(FlexValue::Fixed(128.0))
        .with_margin(Edges::all(5.0))
        .with_parent(nav_bar_container.id)
        .build(wgpu_ctx);

    // Minimize button
    let minimize_button = ButtonBuilder::new()
        .with_sub_component(ButtonSubComponent::Image(ImageConfig {
            file_name: "minimize.png".to_string(),
            scale_mode: ScaleMode::Contain,
        }))
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
        .with_border_radius(BorderRadius::all(4.0))
        .set_fit_to_size()
        .with_content_padding(Edges::all(2.0))
        .with_debug_name("Minimize Button")
        .with_click_event(AppEvent::Minimize)
        .with_event_sender(event_tx.clone())
        .build(wgpu_ctx);

    // Maximize button
    let maximize_button = ButtonBuilder::new()
        .with_sub_component(ButtonSubComponent::Image(ImageConfig {
            file_name: "maximize.png".to_string(),
            scale_mode: ScaleMode::Contain,
        }))
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
        .with_border_radius(BorderRadius::all(4.0))
        .set_fit_to_size()
        .with_content_padding(Edges::all(2.0))
        .with_debug_name("Maximize Button")
        .with_click_event(AppEvent::Maximize)
        .with_event_sender(event_tx.clone())
        .build(wgpu_ctx);

    // Close button
    let close_button = ButtonBuilder::new()
        .with_sub_component(ButtonSubComponent::Image(ImageConfig {
            file_name: "close.png".to_string(),
            scale_mode: ScaleMode::Contain,
        }))
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
        .with_border_radius(BorderRadius::all(4.0))
        .set_fit_to_size()
        .with_content_padding(Edges::all(2.0))
        .with_debug_name("Close Button")
        .with_click_event(AppEvent::Close)
        .with_event_sender(event_tx.clone())
        .build(wgpu_ctx);

    // Add children to the nav buttons container
    nav_buttons_container.add_child(minimize_button);
    nav_buttons_container.add_child(maximize_button);
    nav_buttons_container.add_child(close_button);

    // Add children to the nav bar container
    nav_bar_container.add_child(nav_buttons_container);
    nav_bar_container.add_child(nav_bar_background);

    nav_bar_container
}

fn create_player_bar(wgpu_ctx: &mut WgpuCtx, event_tx: UnboundedSender<AppEvent>) -> Component {
    // Player container
    let mut player_container = FlexContainerBuilder::new()
        .with_debug_name("Player Container")
        .with_size(FlexValue::Fill, FlexValue::Fixed(100.0))
        .with_direction(FlexDirection::Row)
        .with_align_items(AlignItems::Center)
        .with_justify_content(JustifyContent::SpaceBetween)
        .with_padding(Edges::all(10.0))
        .build(wgpu_ctx);

    // Create frosted glass background with an outside border
    let player_container_background =
        BackgroundBuilder::with_frosted_glass(Color::Black, 20.0, 1.0, 0.5)
            .with_debug_name("Player Container Background")
            .with_border_radius(BorderRadius::all(5.0))
            .with_border(1.0, Color::DarkGray.darken(0.05))
            .with_shadow(Color::Black, (0.0, 0.0), 10.0, 0.5)
            .with_fixed_position(Anchor::Center)
            .build(wgpu_ctx);

    let current_song_album_art = ImageBuilder::new("album_art.png")
        .with_scale_mode(ScaleMode::Contain)
        .with_debug_name("Current Song Album Art")
        .with_z_index(1)
        .with_margin(Edges::all(10.0))
        .with_uniform_border_radius(5.0)
        .set_fit_to_size()
        .build(wgpu_ctx);

    let current_song_info = LabelBuilder::new("Song Name\n\nArtist Name")
        .with_color(Color::White)
        .with_font_size(16.0)
        .with_debug_name("Current Song Info")
        .set_fit_to_size()
        .with_z_index(1)
        .build(wgpu_ctx);

    let mut song_info_container = FlexContainerBuilder::new()
        .with_debug_name("Song Info Container")
        .with_direction(FlexDirection::Row)
        .with_align_items(AlignItems::Center)
        .with_justify_content(JustifyContent::Start)
        .build(wgpu_ctx);

    song_info_container.add_child(current_song_album_art);
    song_info_container.add_child(current_song_info);

    let mut player_controls_container = FlexContainerBuilder::new()
        .with_fixed_position(Anchor::Center)
        .with_size(FlexValue::Fraction(0.6), FlexValue::Fill)
        .with_debug_name("Player Controls Container")
        .with_direction(FlexDirection::Column)
        .with_align_items(AlignItems::Center)
        .with_justify_content(JustifyContent::Center)
        .build(wgpu_ctx);

    let mut player_controls_sub_container = FlexContainerBuilder::new()
        .with_debug_name("Player Controls Sub Container")
        .with_direction(FlexDirection::Row)
        .with_align_items(AlignItems::Center)
        .with_justify_content(JustifyContent::Center)
        .with_padding(Edges::horizontal(20.0))
        .with_margin(Edges::top(10.0))
        .build(wgpu_ctx);

    let player_control_btns_size = 20.0;

    let shuffle_button = ButtonBuilder::new()
        .with_sub_component(ButtonSubComponent::Image(ImageConfig {
            file_name: "shuffle.png".to_string(),
            scale_mode: ScaleMode::default(),
        }))
        .with_size(player_control_btns_size, player_control_btns_size)
        .with_debug_name("Shuffle Button")
        .with_click_event(AppEvent::Shuffle)
        .with_event_sender(event_tx.clone())
        .with_margin(Edges::right(10.0))
        .build(wgpu_ctx);

    let previous_button = ButtonBuilder::new()
        .with_sub_component(ButtonSubComponent::Image(ImageConfig {
            file_name: "skip-back.png".to_string(),
            scale_mode: ScaleMode::default(),
        }))
        .with_size(player_control_btns_size, player_control_btns_size)
        .with_debug_name("Previous Button")
        .with_click_event(AppEvent::PreviousTrack)
        .with_event_sender(event_tx.clone())
        .with_margin(Edges::horizontal(10.0))
        .build(wgpu_ctx);

    let play_button = ButtonBuilder::new()
        .with_sub_component(ButtonSubComponent::Image(ImageConfig {
            file_name: "play.png".to_string(),
            scale_mode: ScaleMode::default(),
        }))
        .with_size(player_control_btns_size, player_control_btns_size)
        .with_debug_name("Play Button")
        .with_click_event(AppEvent::PlayPause)
        .with_event_sender(event_tx.clone())
        .with_margin(Edges::horizontal(10.0))
        .build(wgpu_ctx);

    let next_button = ButtonBuilder::new()
        .with_sub_component(ButtonSubComponent::Image(ImageConfig {
            file_name: "skip-forward.png".to_string(),
            scale_mode: ScaleMode::default(),
        }))
        .with_size(player_control_btns_size, player_control_btns_size)
        .with_debug_name("Next Button")
        .with_click_event(AppEvent::NextTrack)
        .with_event_sender(event_tx.clone())
        .with_margin(Edges::horizontal(10.0))
        .build(wgpu_ctx);

    let repeat_button = ButtonBuilder::new()
        .with_sub_component(ButtonSubComponent::Image(ImageConfig {
            file_name: "repeat.png".to_string(),
            scale_mode: ScaleMode::default(),
        }))
        .with_size(player_control_btns_size, player_control_btns_size)
        .with_debug_name("Repeat Button")
        .with_click_event(AppEvent::Repeat)
        .with_event_sender(event_tx.clone())
        .with_margin(Edges::left(10.0))
        .build(wgpu_ctx);

    player_controls_sub_container.add_child(shuffle_button);
    player_controls_sub_container.add_child(previous_button);
    player_controls_sub_container.add_child(play_button);
    player_controls_sub_container.add_child(next_button);
    player_controls_sub_container.add_child(repeat_button);

    player_controls_container.add_child(player_controls_sub_container);

    // Replace the placeholder song progress slider with a fully customized slider
    let song_progress_slider = SliderBuilder::new()
        .with_value(25.0) // Start at 25% as an example
        .with_range(0.0, 100.0)
        .with_track_color(Color::DarkGray.lighten(0.1))
        .with_track_fill_color(Color::Blue.lighten(0.1))
        .with_track_height(4.0)
        .with_track_border_radius(BorderRadius::all(2.0))
        .with_thumb_color(Color::White)
        .with_thumb_size(8.0)
        .with_debug_name("Song Progress Slider")
        .build(wgpu_ctx);

    player_controls_container.add_child(song_progress_slider);

    // Replace the placeholder volume slider with a customized slider
    let volume_slider = SliderBuilder::new()
        .with_value(70.0) // Set volume at 70% as an example
        .with_range(0.0, 100.0)
        .with_track_color(Color::Gray.lighten(0.2))
        .with_track_fill_color(Color::Green.lighten(0.1))
        .with_track_height(4.0)
        .with_track_border_radius(BorderRadius::all(2.0))
        .with_thumb_color(Color::White)
        .with_thumb_size(8.0)
        .with_margin(Edges::all(10.0))
        .with_size(FlexValue::Fixed(100.0), FlexValue::Fixed(20.0))
        .with_debug_name("Volume Slider")
        .build(wgpu_ctx);

    player_container.add_child(player_container_background);
    player_container.add_child(song_info_container);
    player_container.add_child(player_controls_container);
    player_container.add_child(volume_slider);

    player_container
}
