use std::borrow::BorrowMut;

use emacs::{raw::emacs_value, Env, IntoLisp, Result, UnibyteString, Value};
use emacs_module::emacs_env_31;

#[derive(Clone, Copy, Debug)]
pub struct RawValue {
    pub raw: emacs_value,
    pub raw_env: *mut emacs_env_31,
}

impl RawValue {
    pub fn make_global(&mut self) {
        let global_callback = unsafe {
            let make_global_ref = (*self.raw_env).make_global_ref.unwrap();
            make_global_ref(self.raw_env, self.raw)
        };
        self.raw = global_callback;
    }

    pub fn free_global(self) {
        unsafe {
            let free_global_ref = (*self.raw_env).free_global_ref.unwrap();
            free_global_ref(self.raw_env, self.raw);
        };
    }

    pub fn replace_env(&mut self, env: &Env) {
        self.raw_env = env.raw;
    }

    pub fn call(self, arg: UnibyteString) {
        unsafe {
            let funcall = (*self.raw_env).funcall.unwrap();
            let arg = make_unibyte_string(arg, self.raw_env);
            let mut args = vec![arg];
            let lisp_args: &mut [emacs_value] = args.borrow_mut();
            let ptr = lisp_args.as_mut_ptr();
            let length = lisp_args.len() as isize;
            println!("funcall: {self:?}, {ptr:?}, {length:?}");
            funcall(self.raw_env, self.raw, length, ptr);
        }
    }
}

fn make_unibyte_string(str: UnibyteString, env: *mut emacs_env_31) -> emacs_value {
    let bytes = str.content;
    let len = bytes.len();
    let ptr = bytes.as_ptr();
    let res = unsafe {
        let fun = (*env).make_unibyte_string.unwrap();
        fun(env, ptr as *const std::os::raw::c_char, len as isize)
    };
    res
}

impl IntoLisp<'_> for RawValue {
    fn into_lisp(self, env: &'_ Env) -> Result<Value<'_>> {
        Ok(Value { raw: self.raw, env })
    }
}

impl From<Value<'_>> for RawValue {
    fn from(value: Value) -> Self {
        Self {
            raw: value.raw,
            raw_env: value.env.raw,
        }
    }
}

unsafe impl Send for RawValue {}
unsafe impl Sync for RawValue {}
