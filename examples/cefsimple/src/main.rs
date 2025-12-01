use crate::sys::cef_log_items_t;
use cef::{args::Args, rc::*, *};
use std::sync::{Arc, Mutex};

use tracing::{error, info};

wrap_app! {
    struct MyApp {
        window: Arc<Mutex<Option<Window>>>,
    }

    impl App {
        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(MyBrowserProcessHandler::new(
                self.window.clone(),
            ))
        }

        // this is the place to control command-line parameters always, such as removing any that should not be allowed or
        // adding ones that should be passed.  TODO: need to check if the final params get shown in the process command line,
        // which is something that we'd like to avoid for security reasons
        fn on_before_command_line_processing(&self, _process_type: Option<&CefString>, command_line: Option<&mut CommandLine>) {
            if let Some(cmd_line) = command_line {
                // macos only at this point
                #[cfg(target_os = "macos")]
                cmd_line.append_switch(Some(&"use-mock-keychain".into()));   // avoid popup asking for keychain access on app launch

                // all platforms
                cmd_line.append_switch(Some(&"disable-component-update".into())); // attempt to not use google services, doesn't really seem to work
            }
        }

        // will be called to register any custom schemes; use the passed-in _registrar to add one or more custom schemes
        fn on_register_custom_schemes(&self, _registrar: Option<&mut SchemeRegistrar>) {
            info!("App::on_register_custom_schemes called");
        }
    }
}

wrap_browser_process_handler! {
    struct MyBrowserProcessHandler {
        window: Arc<Mutex<Option<Window>>>,
    }

    impl BrowserProcessHandler {
    //    fn on_register_custom_preferences(
    //         &self,
    //         _type_: PreferencesType,
    //         registrar: Option<&mut PreferenceRegistrar>,
    //     ) {
    //         info!("MyBrowserProcessHandler::on_register_custom_preferences");
    //         let mut off = value_create();
    //         if let Some(ref mut value) = off {
    //                 value.set_bool(0);
    //                 if let Some(registrar) = registrar {
    //                     let rv = registrar.add_preference(Some(&CefString::from("autofill.profile_enabled")), Some(value));
    //                     info!("    registrar.add_preference(autofill.profile_enabled) returned {}", rv);
    //                 }
    //         }
    //     }

        // The real lifespan of cef starts from `on_context_initialized`, so all the cef objects should be manipulated after that.
        fn on_context_initialized(&self) {
            info!("BrowserProcessHandler::on_context_intiialized");

            let pref_mgr = preference_manager_get_global();
            if let Some(ref mgr) = pref_mgr {
                let mut off = value_create();
                if let Some(ref mut value) = off {
                    value.set_bool(0);

                    let all = mgr.all_preferences(0).unwrap();
                    let mut list = CefStringList::new();
                    all.keys(Some(&mut list));
                    for x in list {
                        info!("{}", x);
                    }

                    let mut err = CefString::from("");
                    let result = mgr.set_preference(Some(&CefString::from("autofill.enabled")), Some(value), Some(&mut err));
                    if !err.to_string().is_empty() { error!("    {err}"); }
                    info!("    trying to set preference autofill.enabled returned {}", result);
                }
            }


            let mut client = MyClient::new();
            let url = CefString::from("https://www.google.com");

            let browser_view = browser_view_create(
                Some(&mut client),
                Some(&url),
                Some(&Default::default()),
                Option::<&mut DictionaryValue>::None,
                Option::<&mut RequestContext>::None,
                Option::<&mut BrowserViewDelegate>::None,
            )
            .expect("Failed to create browser view");

            let mut delegate = MyWindowDelegate::new(browser_view);
            if let Ok(mut window) = self.window.lock() {
                *window = Some(
                    window_create_top_level(Some(&mut delegate)).expect("Failed to create window"),
                );
            }
        }

        fn on_before_child_process_launch(&self, command_line: Option<&mut CommandLine>) {
            info!("MyBrowserProcessHandler::on_before_child_process_launch");

            if let Some(cmd_line) = command_line {
                // macos only at this point
                #[cfg(target_os = "macos")]
                cmd_line.append_switch(Some(&"use-mock-keychain".into()));   // avoid popup asking for keychain access on app launch

                // all platforms
                cmd_line.append_switch(Some(&"disable-component-update".into())); // attempt to not use google services, doesn't really seem to work
            }
        }
    }
}

wrap_client! {
    struct MyClient;
    impl Client {
        fn context_menu_handler(&self) -> Option<ContextMenuHandler> {
            Some(MyContextMenuHandler::new())
        }
    }
}

wrap_window_delegate! {
    struct MyWindowDelegate {
        browser_view: BrowserView,
    }

    impl ViewDelegate {
        fn on_child_view_changed(
            &self,
            _view: Option<&mut View>,
            _added: ::std::os::raw::c_int,
            _child: Option<&mut View>,
        ) {
            // view.as_panel().map(|x| x.as_window().map(|w| w.close()));
        }
    }

    impl PanelDelegate {}

    impl WindowDelegate {
        fn on_window_created(&self, window: Option<&mut Window>) {
            if let Some(window) = window {
                let view = self.browser_view.clone();
                window.add_child_view(Some(&mut (&view).into()));
                window.show();
            }
        }

        fn on_window_destroyed(&self, _window: Option<&mut Window>) {
            quit_message_loop();
        }

        fn with_standard_window_buttons(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
            1
        }

        fn can_resize(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
            1
        }

        fn can_maximize(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
            1
        }

        fn can_minimize(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
            1
        }

        fn can_close(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
            1
        }
    }
}

wrap_context_menu_handler! {
    struct MyContextMenuHandler;

    impl ContextMenuHandler {
        // by clearing the passed-in model param, we disable showing any context menu, because we've emptied out
        // any default context menu items
        fn on_before_context_menu(
            &self,
             _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            _params: Option<&mut ContextMenuParams>,
            model: Option<&mut MenuModel>,
        ) {

             info!("MyContextMenuHandler::on_before_context_menu");

            if let Some(model) = model {
                info!("   passed-in model has {} items", model.count());

                model.clear();
                info!("   after clearing the model, model has {} items", model.count());

            }
        }

        // this prevents context menu commands from running in case somehow a context menu is shown.
        // returning 1 tells CEF that the command was handled, whereas returning 0 would invoke a
        // default handler if one exists (I think)
        fn on_context_menu_command(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            _params: Option<&mut ContextMenuParams>,
            _command_id: ::std::os::raw::c_int,
            _event_flags: EventFlags,
        ) -> i32 {
            info!("MyContextMenuHandler::on_context_menu_command");
            return 1;
        }

        // on macos, this prevents OS default context menu from showing when right-clicking on a link,
        // preventing things such as various default services like speech, text editing, passing to other apps, etc.
        fn run_context_menu(&self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            _params: Option<&mut ContextMenuParams>,
            model: Option<&mut MenuModel>,
            _callback: Option<&mut RunContextMenuCallback>,) -> i32 {
                info!("MyContextMenuHandler::run_context_menu");
                if let Some(model) = model {
                    model.clear();
                }
                return 1;

        }

         fn on_quick_menu_command(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            _command_id: ::std::os::raw::c_int,
            _event_flags: EventFlags,
        ) -> ::std::os::raw::c_int {
            info!("MyContextMenu::on_quick_menu_command");
            Default::default()
        }
    }
}

// FIXME: Rewrite this example based on cef/tests/cefsimple
fn main() {
    let _stdout_subscriber = tracing_subscriber::fmt::init();

    #[cfg(target_os = "macos")]
    let _loader = {
        let loader = library_loader::LibraryLoader::new(&std::env::current_exe().unwrap(), false);
        assert!(loader.load());
        loader
    };

    #[cfg(target_os = "macos")]
    {
        use objc2::{
            ClassType, MainThreadMarker, msg_send,
            rc::Retained,
            runtime::{AnyObject, NSObjectProtocol},
        };
        use objc2_app_kit::NSApp;

        use application::SimpleApplication;

        let mtm = MainThreadMarker::new().unwrap();

        unsafe {
            // Initialize the SimpleApplication instance.
            // SAFETY: mtm ensures that here is the main thread.
            let _: Retained<AnyObject> = msg_send![SimpleApplication::class(), sharedApplication];
        }

        // If there was an invocation to NSApp prior to here,
        // then the NSApp will not be a SimpleApplication.
        // The following assertion ensures that this doesn't happen.
        assert!(NSApp(mtm).isKindOfClass(SimpleApplication::class()));
    }

    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);

    let args = Args::new();
    let cmd = args.as_cmd_line().unwrap();

    let switch = CefString::from("type");
    let is_browser_process = cmd.has_switch(Some(&switch)) != 1;

    let window = Arc::new(Mutex::new(None));
    let mut app = MyApp::new(window.clone());

    let ret = execute_process(
        Some(args.as_main_args()),
        Some(&mut app),
        std::ptr::null_mut(),
    );

    if is_browser_process {
        info!("launching browser process");
        assert!(ret == -1, "cannot execute browser process");
    } else {
        let process_type = CefString::from(&cmd.switch_value(Some(&switch)));
        info!("launch {process_type} process");
        assert!(ret >= 0, "cannot execute non-browser process");
        // non-browser process does not initialize cef
        return;
    }

    // TODO: need to set root_cache_path and cache_path to an application-specific name so as not to conflict with
    // other generic CEF instances. See:
    // root_cache_path:  https://cef-builds.spotifycdn.com/docs/122.1/structcef__settings__t.html#a2e2be03f34ddd93de90e1cf196757a19
    // cache_path:  https://cef-builds.spotifycdn.com/docs/122.1/structcef__settings__t.html#ad1644a7eb23cad969181db010f007710
    // OnAlreadyRunningAppRelaunche:  https://cef-builds.spotifycdn.com/docs/122.1/classCefBrowserProcessHandler.html#a052a91639483467c0b546d57a05c2f06
    let settings = Settings {
        no_sandbox: !cfg!(feature = "sandbox") as _,
        log_items: cef_log_items_t::LOG_ITEMS_NONE.into(), // don't log pid, tid, ticks in cef log messages
        ..Default::default()
    };
    assert_eq!(
        initialize(
            Some(args.as_main_args()),
            Some(&settings),
            Some(&mut app),
            std::ptr::null_mut(),
        ),
        1
    );

    run_message_loop();

    let window = window.lock().expect("Failed to lock window");
    let window = window.as_ref().expect("Window is None");
    assert!(window.has_one_ref());

    shutdown();
}

#[cfg(target_os = "macos")]
mod application {
    use std::cell::Cell;

    use cef::application_mac::{CefAppProtocol, CrAppControlProtocol, CrAppProtocol};
    use objc2::{DefinedClass, define_class, runtime::Bool};
    use objc2_app_kit::NSApplication;

    /// Instance variables of `SimpleApplication`.
    pub struct SimpleApplicationIvars {
        handling_send_event: Cell<Bool>,
    }

    define_class!(
        /// A `NSApplication` subclass that implements the required CEF protocols.
        ///
        /// This class provides the necessary `CefAppProtocol` conformance to
        /// ensure that events are handled correctly by the Chromium framework on macOS.
        #[unsafe(super(NSApplication))]
        #[ivars = SimpleApplicationIvars]
        pub struct SimpleApplication;

        unsafe impl CrAppControlProtocol for SimpleApplication {
            #[unsafe(method(setHandlingSendEvent:))]
            unsafe fn set_handling_send_event(&self, handling_send_event: Bool) {
                self.ivars().handling_send_event.set(handling_send_event);
            }
        }

        unsafe impl CrAppProtocol for SimpleApplication {
            #[unsafe(method(isHandlingSendEvent))]
            unsafe fn is_handling_send_event(&self) -> Bool {
                self.ivars().handling_send_event.get()
            }
        }

        unsafe impl CefAppProtocol for SimpleApplication {}
    );
}
