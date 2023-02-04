use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[cfg(feature = "macros")]
use {
    crate::chunk::Chunk, proc_macro::TokenTree, proc_macro2::TokenStream as TokenStream2,
    proc_macro_error::proc_macro_error,
};

#[proc_macro_attribute]
pub fn lua_module(_: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let func_name = func.sig.ident.clone();

    let wrapped = quote! {
        ::mglua::require_module_feature!();

        #func

        #[no_mangle]
        unsafe extern "C" fn gmod13_open(state: *mut ::mglua::lua_State) -> ::std::os::raw::c_int {
            ::mglua::Lua::init_from_ptr(state)
                .entrypoint1(#func_name)
                .expect("cannot initialize module")
        }
    };

    wrapped.into()
}

#[cfg(feature = "macros")]
fn to_ident(tt: &TokenTree) -> TokenStream2 {
    let s: TokenStream = tt.clone().into();
    s.into()
}

#[cfg(feature = "macros")]
#[proc_macro]
#[proc_macro_error]
pub fn chunk(input: TokenStream) -> TokenStream {
    let chunk = Chunk::new(input);

    let source = chunk.source();

    let caps_len = chunk.captures().len();
    let caps = chunk.captures().iter().map(|cap| {
        let cap_name = cap.as_rust().to_string();
        let cap = to_ident(cap.as_rust());
        quote! { env.raw_set(#cap_name, #cap)?; }
    });

    let wrapped_code = quote! {{
        use ::mglua::{AsChunk, ChunkMode, Lua, Result, Value};
        use ::std::borrow::Cow;
        use ::std::io::Result as IoResult;
        use ::std::sync::Mutex;

        struct InnerChunk<F: for <'a> FnOnce(&'a Lua) -> Result<Value<'a>>>(Mutex<Option<F>>);

        impl<F> AsChunk<'static> for InnerChunk<F>
        where
            F: for <'a> FnOnce(&'a Lua) -> Result<Value<'a>>,
        {
            fn env<'lua>(&self, lua: &'lua Lua) -> Result<Value<'lua>> {
                if #caps_len > 0 {
                    if let Ok(mut make_env) = self.0.lock() {
                        if let Some(make_env) = make_env.take() {
                            return make_env(lua);
                        }
                    }
                }
                Ok(Value::Nil)
            }

            fn mode(&self) -> Option<ChunkMode> {
                Some(ChunkMode::Text)
            }

            fn source(self) -> IoResult<Cow<'static, [u8]>> {
                Ok(Cow::Borrowed((#source).as_bytes()))
            }
        }

        fn annotate<F: for<'a> FnOnce(&'a Lua) -> Result<Value<'a>>>(f: F) -> F { f }

        let make_env = annotate(move |lua: &Lua| -> Result<Value> {
            let globals = lua.globals();
            let env = lua.create_table()?;
            let meta = lua.create_table()?;
            meta.raw_set("__index", globals.clone())?;
            meta.raw_set("__newindex", globals)?;

            // Add captured variables
            #(#caps)*

            env.set_metatable(Some(meta));
            Ok(Value::Table(env))
        });

        InnerChunk(Mutex::new(Some(make_env)))
    }};

    wrapped_code.into()
}

#[cfg(feature = "macros")]
mod chunk;
#[cfg(feature = "macros")]
mod token;