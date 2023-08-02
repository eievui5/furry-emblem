use proc_macro2::TokenStream;
use quote::quote;
use std::num::*;

pub trait ToStatic {
    fn static_type() -> TokenStream;
    fn static_value(&self) -> TokenStream;
}

macro_rules! plain_static {
    ($type:ident, $($remaining:ident),+ $(,)?) => {
        plain_static!($type);
        plain_static!($($remaining),+);
    };
    ($type:ident $(,)?) => {
        impl ToStatic for $type {
            fn static_type() -> TokenStream {
                quote!($type)
            }
            fn static_value(&self) -> TokenStream {
                quote!(#self)
            }
        }
    };
}

macro_rules! nonzero_static {
    ($type:ident, $($remaining:ident),+ $(,)?) => {
        nonzero_static!($type);
        nonzero_static!($($remaining),+);
    };
    ($type:ident $(,)?) => {
        impl ToStatic for $type {
            fn static_type() -> TokenStream {
                quote!($type)
            }
            fn static_value(&self) -> TokenStream {
                let value = self.get();
                // This value is generated using a NonZero type.
                // Under no circumstance will it be 0.
                // using the unsafe method is necessary because Option::unwrap is not const
                quote!(unsafe { $type::new_unchecked(#value) })
            }
        }
    };
}

plain_static!(u8, u16, u32, u64);
plain_static!(i8, i16, i32, i64);
plain_static!(bool);

nonzero_static!(NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64);
nonzero_static!(NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64);

impl ToStatic for String {
    fn static_type() -> TokenStream {
        quote!(&'static str)
    }
    fn static_value(&self) -> TokenStream {
        quote!(#self)
    }
}

impl<T: ToStatic> ToStatic for Vec<T> {
    fn static_type() -> TokenStream {
        let ty = T::static_type();
        quote!(&'static [#ty])
    }
    fn static_value(&self) -> TokenStream {
        let iter = self.iter().map(|t| t.static_value());
        quote! {
            &[#(#iter),*]
        }
    }
}

impl<T: ToStatic> ToStatic for Option<T> {
    fn static_type() -> TokenStream {
        let ty = T::static_type();
        quote!(Option<#ty>)
    }
    fn static_value(&self) -> TokenStream {
        match self {
            Some(thing) => {
                let thing = thing.static_value();
                quote!(Some(#thing))
            }
            None => quote!(None),
        }
    }
}
