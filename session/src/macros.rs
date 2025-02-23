macro_rules! reply {
    ($payload:expr) => {{
        use gstd::msg;
        msg::reply($payload, 0).expect("Error in sending reply");
    }};
}

macro_rules! create_inner_state {
    ($ident:ident, $type:ty) => {
        static mut $ident: Option<$type> = None;

        unsafe fn init_inner_state(val: $type) {
            $ident = Some(val);
        }

        fn get_inner_state_mut() -> &'static mut $type {
            unsafe { $ident.as_mut().expect("State is not initialized") }
        }

        fn get_inner_state() -> $type {
            unsafe { $ident.take().expect("State is not initialized") }
        }
    };
}
