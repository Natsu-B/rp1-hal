use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, ItemFn, ItemStatic, ext::IdentExt, parse_macro_input};

fn error(message: &str) -> TokenStream {
    syn::Error::new(proc_macro2::Span::call_site(), message)
        .to_compile_error()
        .into()
}

#[proc_macro_derive(Message)]
pub fn message(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    if !input.generics.params.is_empty() {
        return error("Message requires a concrete fixed-width struct");
    }
    let repr_c = input
        .attrs
        .iter()
        .any(|a| a.path().is_ident("repr") && a.parse_args::<syn::Ident>().is_ok_and(|r| r == "C"));
    if !repr_c {
        return error("Message requires #[repr(C)] (packed and native enums are unsupported)");
    }
    let Data::Struct(data) = input.data else {
        return error("use integer fields for wire enums and validate after reception");
    };
    let name = input.ident;
    let types: Vec<_> = data.fields.iter().map(|f| &f.ty).collect();
    let fields: Vec<syn::Member> = match &data.fields {
        Fields::Named(f) => f
            .named
            .iter()
            .map(|f| syn::Member::Named(f.ident.clone().unwrap()))
            .collect(),
        Fields::Unnamed(f) => (0..f.unnamed.len())
            .map(|i| syn::Member::Unnamed(syn::Index::from(i)))
            .collect(),
        Fields::Unit => return error("Message must not be empty"),
    };
    quote! {
        const _: () = {
            assert!(::core::mem::size_of::<#name>() > 0, "Message must not be empty");
            assert!(::core::mem::size_of::<#name>() == 0 #( + ::core::mem::size_of::<#types>() )*, "Message must have no padding");
        };
        unsafe impl ::rp1_proto::Message for #name {
            const TYPE_ID: u64 = {
                let mut h = ::rp1_proto::hash(concat!(module_path!(), "::", stringify!(#name)).as_bytes());
                h ^= ::core::mem::size_of::<Self>() as u64;
                #( h = h.wrapping_mul(0x100_0000_01b3) ^ <#types as ::rp1_proto::Message>::TYPE_ID;
                   h = h.wrapping_mul(0x100_0000_01b3) ^ ::core::mem::offset_of!(Self, #fields) as u64; )*
                h
            };
        }
    }.into()
}

#[proc_macro_attribute]
pub fn entry(args: TokenStream, input: TokenStream) -> TokenStream {
    let core = args.to_string();
    if core != "proc0" && core != "proc1" {
        return error("entry requires proc0 or proc1");
    }
    let f = parse_macro_input!(input as ItemFn);
    if f.sig.inputs.len() != 2
        || f.sig.asyncness.is_some()
        || f.sig.unsafety.is_some()
        || !f.sig.generics.params.is_empty()
        || !matches!(&f.sig.output, syn::ReturnType::Type(_, t) if matches!(t.as_ref(), syn::Type::Never(_)))
    {
        return error("entry must be fn(core: Proc0/Proc1, resources: rp1::Proc0/1Resources) -> !");
    }
    let name = &f.sig.ident;
    let wrapper = format_ident!("__rp1_{}_main", core);
    let token = format_ident!("{}", if core == "proc0" { "Proc0" } else { "Proc1" });
    let resources = format_ident!("{}Resources", token);
    quote! {
        #f
        #[unsafe(no_mangle)]
        unsafe extern "C" fn #wrapper() -> ! {
            #name(unsafe { ::rp1_rt::#token::new() }, unsafe { rp1::#resources::new() })
        }
    }
    .into()
}

/// Bind a named RP1 IRQ to one core. Use `proc0` or `proc1`, optionally followed by
/// `sram`; handlers otherwise start in ISRAM. System IRQ names are reserved.
#[proc_macro_attribute]
pub fn interrupt(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args with syn::punctuated::Punctuated::<syn::Ident, syn::Token![,]>::parse_terminated);
    let mut args = args.iter();
    let core = match args.next().map(ToString::to_string).as_deref() {
        Some("proc0") => 0,
        Some("proc1") => 1,
        _ => return error("interrupt requires proc0 or proc1, optionally followed by sram"),
    };
    let (mut sram, mut platform) = (false, false);
    for arg in args {
        match arg.to_string().as_str() {
            "sram" if !sram => sram = true,
            // Private opt-in for rp1-platform PCIE_OUT/PROC_MISC/CLOCKS_DEFAULT handlers.
            "__platform" if !platform && core == 0 => platform = true,
            _ => return error("invalid or repeated interrupt option"),
        }
    }
    let mut f = parse_macro_input!(input as ItemFn);
    let unit = match &f.sig.output {
        syn::ReturnType::Default => true,
        syn::ReturnType::Type(_, ty) => {
            matches!(ty.as_ref(), syn::Type::Tuple(t) if t.elems.is_empty())
        }
    };
    if !f.sig.inputs.is_empty()
        || !unit
        || f.sig.asyncness.is_some()
        || f.sig.unsafety.is_some()
        || f.sig.constness.is_some()
        || f.sig.abi.is_some()
        || !f.sig.generics.params.is_empty()
        || f.sig.generics.where_clause.is_some()
    {
        return error(
            "interrupt handler must be a safe, non-async fn NAME() with no generics or return value",
        );
    }
    let name = &f.sig.ident;
    let unraw = name.unraw();
    if matches!(
        unraw.to_string().as_str(),
        "PCIE_OUT" | "PROC_MISC" | "CLOCKS_DEFAULT"
    ) && !platform
    {
        return error("this interrupt is reserved for rp1-platform");
    }
    let symbol = format!("__rp1_p{core}_{unraw}");
    let wrapper = format_ident!("{symbol}_trampoline");
    // SRAM handlers must also be boundaries for the automatic ISRAM planner.
    let section = if sram {
        format!(".text.rp1_cold.{symbol}")
    } else {
        format!(".isram.text.{symbol}")
    };
    let attrs = core::mem::take(&mut f.attrs);
    let cfg: Vec<_> = attrs
        .iter()
        .filter(|a| a.path().is_ident("cfg") || a.path().is_ident("cfg_attr"))
        .collect();
    quote! {
        #(#cfg)*
        const _: ::rp1_rt::pac::Interrupt = ::rp1_rt::pac::Interrupt::#name;
        #(#attrs)*
        #[allow(non_snake_case)]
        #[unsafe(export_name = #symbol)]
        #[cfg_attr(all(target_arch = "arm", target_os = "none"), unsafe(link_section = #section))]
        unsafe extern "C" fn #wrapper() {
            #[allow(non_snake_case)]
            #[inline(always)]
            #f
            #name()
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn isram(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        return error("isram takes no arguments");
    }
    let f = parse_macro_input!(input as ItemFn);
    // Only the bare-metal RP1 target has ISRAM; host builds (tests, Linux users of rp1-proto)
    // keep ordinary placement and inlining.
    quote!(
        #[cfg_attr(
            all(target_arch = "arm", target_os = "none"),
            unsafe(link_section = ".isram.text"),
            inline(never)
        )]
        #f
    )
    .into()
}

/// Rarely executed code that must stay in shared SRAM: automatic ISRAM placement
/// (`cargo rp1 build`) stops at these functions even when an ISRAM function calls them.
#[proc_macro_attribute]
pub fn cold(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        return error("cold takes no arguments");
    }
    let f = parse_macro_input!(input as ItemFn);
    quote!(
        #[cold]
        #[inline(never)]
        #[cfg_attr(
            all(target_arch = "arm", target_os = "none"),
            unsafe(link_section = ".text.rp1_cold")
        )]
        #f
    )
    .into()
}

#[proc_macro_attribute]
pub fn shared(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        return error("shared takes no arguments");
    }
    let s = parse_macro_input!(input as ItemStatic);
    if matches!(s.mutability, syn::StaticMutability::Mut(_)) {
        return error("shared static mut is forbidden; use a SharedSafe type");
    }
    let ty = &s.ty;
    quote! {
        const _: () = { fn check<T: ::rp1_proto::SharedSafe>() {} let _ = check::<#ty>; };
        #[unsafe(link_section = ".shared")]
        #s
    }
    .into()
}

fn local(input: TokenStream, core: &str) -> TokenStream {
    let s = parse_macro_input!(input as ItemStatic);
    if matches!(s.mutability, syn::StaticMutability::Mut(_)) {
        return error("core-local static mut is forbidden; use Cell or RefCell");
    }
    let ItemStatic {
        attrs,
        vis,
        ident,
        ty,
        expr,
        ..
    } = s;
    let core = format_ident!("{}", core);
    quote! {
        #(#attrs)* #vis static #ident: ::rp1_rt::Local<#ty, ::rp1_rt::#core> =
            ::rp1_rt::Local::new(#expr);
    }
    .into()
}
#[proc_macro_attribute]
pub fn proc0(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        return error("proc0 takes no arguments");
    }
    local(input, "Proc0")
}
#[proc_macro_attribute]
pub fn proc1(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        return error("proc1 takes no arguments");
    }
    local(input, "Proc1")
}
