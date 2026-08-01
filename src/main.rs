use glfw::Context;

enum BackgroundEvent {
	Open,
}

fn main() {
	let mut glfw = glfw::init_no_callbacks().unwrap();

	unsafe {
		platform::init();
	}

	glfw.window_hint(glfw::WindowHint::Visible(false));
	glfw.window_hint(glfw::WindowHint::Floating(true));
	glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
	// glfw.window_hint(glfw::WindowHint::Decorated(false));

	let (mut window, events) = glfw.create_window(
		300,
		300,
		"pm",
		glfw::WindowMode::Windowed
	).unwrap();

	window.set_key_polling(true);
	window.set_char_polling(true);
	window.set_focus_polling(true);

	unsafe { platform::setup_window(&mut window) };

	let renderer = unsafe {
		femtovg::renderer::OpenGl::new_from_function(|proc| {
			match window.get_proc_address(proc) {
				Some(addr) => addr as _,
				None => std::ptr::null(),
			}
		})
	}.expect("femtovg renderer should be initiable");

	let mut canvas = femtovg::Canvas::new(renderer)
		.expect("femtovg canvas should be initiable");

	let (tx, rx) = std::sync::mpsc::channel();

	let kbhook = livesplit_hotkey::Hook::with_consume_preference(
		livesplit_hotkey::ConsumePreference::PreferConsume
	).expect("global hotkey hook should be registrable");

	let glfw_open = glfw::ThreadSafeGlfw::from(&mut glfw);

	kbhook.register(livesplit_hotkey::Hotkey {
		key_code: livesplit_hotkey::KeyCode::Space,
		modifiers: livesplit_hotkey::Modifiers::CONTROL,
	}, move || {
		let _ = tx.send(BackgroundEvent::Open);
		glfw_open.post_empty_event();
	}).expect("global hotkey should be registrable");

	let mut hidden = true;

	let mut buffer = String::new();

	while !window.should_close() {
		glfw.wait_events();

		match rx.try_recv() {
			Ok(BackgroundEvent::Open) => {
				if hidden {
					unsafe { platform::show_window(&mut window) };
					hidden = false;
				}
			}
			Err(std::sync::mpsc::TryRecvError::Empty) => {}
			Err(std::sync::mpsc::TryRecvError::Disconnected) => unreachable!("hotkey listener should not close by itself")
		}

		for (_, event) in glfw::flush_messages(&events) {
			println!("{:?}", event);
			match event {
				glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
					if !hidden {
						window.hide();
						hidden = true;
					}
				},
				glfw::WindowEvent::Char(c) => {
					buffer.push(c);
				},
				_ => {},
			}
		}

		let (width, height) = window.get_framebuffer_size();

		canvas.set_size(
			width.try_into().expect("framebuffer width should be positive"),
			height.try_into().expect("framebuffer height should be positive"),
			1.0, // Any other value for "DPI" (actually DPR) gives poor visual results.
		);

		canvas.clear_rect(0, 0, canvas.width(), canvas.height(), femtovg::Color::black());

		let mut line = femtovg::Path::new();

		line.move_to(0.0, 0.0);
		line.line_to(canvas.width() as f32, canvas.height() as f32);

		canvas.stroke_path(&line, &femtovg::Paint::color(femtovg::Color::white()).with_line_width(10.0));

		canvas.flush_to_output(());

		window.swap_buffers();
	}
}

#[cfg(target_os = "macos")]
mod platform {
	use objc2::ClassType;
	use raw_window_handle::HasWindowHandle;

	pub unsafe fn init() {
		let marker = unsafe { objc2_foundation::MainThreadMarker::new_unchecked() };
		let ns_app = objc2_app_kit::NSApplication::sharedApplication(marker);
		ns_app.setActivationPolicy(objc2_app_kit::NSApplicationActivationPolicy::Accessory);
	}

	unsafe fn window_ns_from_glfw(window: &mut glfw::Window) -> objc2::rc::Retained<objc2_app_kit::NSWindow> {
		let handle = window.window_handle().unwrap();

		match handle.as_raw() {
			raw_window_handle::RawWindowHandle::AppKit(handle) => {
				let ns_view = unsafe {
					objc2::rc::Retained::<objc2_app_kit::NSView>::retain(handle.ns_view.as_ptr().cast())
				}.expect("view should be non-null");

				ns_view.window().expect("view should be in a window")
			}
			handle => unreachable!("unknown handle {handle:?} for platform"),
		}
	}

	pub unsafe fn setup_window(window: &mut glfw::Window) {
		let ns_window = unsafe { window_ns_from_glfw(window) };

		unsafe {
			objc2::runtime::AnyObject::set_class(&ns_window, objc2_app_kit::NSPanel::class());
		}

		ns_window.setStyleMask(
			objc2_app_kit::NSWindowStyleMask::Titled
			| objc2_app_kit::NSWindowStyleMask::FullSizeContentView
			| objc2_app_kit::NSWindowStyleMask::NonactivatingPanel
		);

		unsafe {
			ns_window.setCollectionBehavior(
				objc2_app_kit::NSWindowCollectionBehavior::CanJoinAllSpaces
			);
		}

		ns_window.setTitleVisibility(objc2_app_kit::NSWindowTitleVisibility::NSWindowTitleHidden);
		ns_window.setTitlebarAppearsTransparent(true);
	}

	pub unsafe fn show_window(window: &mut glfw::Window) {
		unsafe { window_ns_from_glfw(window) }.makeKeyAndOrderFront(None);
	}
}
