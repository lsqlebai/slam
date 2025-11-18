use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, parse2, ItemFn, ImplItemFn, Type};

/// #[inject_ctx]         -> ctx: &Context   (默认类型)
/// #[inject_ctx(MyCtx)]  -> ctx: &MyCtx    (自定义类型)
#[proc_macro_attribute]
pub fn inject_ctx(attr: TokenStream, item: TokenStream) -> TokenStream {
    // 1. 解析 attribute 里的类型
    let ctx_ty: Type = if attr.is_empty() {
        // 不写参数时默认 Context
        syn::parse_quote!(Context)
    } else {
        parse_macro_input!(attr as Type)
    };

    // 2. 把输入变成 TokenStream2，方便多次 parse
    let ts2: TokenStream2 = item.into();

    // 3. 先尝试当自由函数解析
    if let Ok(mut func) = parse2::<ItemFn>(ts2.clone()) {
        let ctx_arg: syn::FnArg = syn::parse_quote!(
            ctx: &#ctx_ty
        );
        // 放到最后
        func.sig.inputs.push(ctx_arg);

        return quote!(#func).into();
    }

    // 4. 再尝试当 impl 方法解析
    if let Ok(mut method) = parse2::<ImplItemFn>(ts2.clone()) {
        let ctx_arg: syn::FnArg = syn::parse_quote!(
            ctx: &#ctx_ty
        );
        // 也放到最后（self / &self / &mut self 会自然在最前面）
        method.sig.inputs.push(ctx_arg);

        return quote!(#method).into();
    }

    // 5. 都不是就报错
    syn::Error::new_spanned(
        ts2,
        "#[inject_ctx] 目前只支持自由函数和 impl 方法",
    )
    .to_compile_error()
    .into()
}