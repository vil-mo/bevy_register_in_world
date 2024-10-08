use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse_macro_input, parse_quote, DeriveInput, ExprPath, Ident, LitStr, Path, Result};

fn bevy_ecs_path() -> syn::Path {
    bevy_macro_utils::BevyManifest::default().get_path("bevy_ecs")
}

fn component_api_path() -> syn::Path {
    bevy_macro_utils::BevyManifest::parse_str("bevy_register_in_world::component")
}


#[proc_macro_derive(ComponentAutoRegister, attributes(component))]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);
    let bevy_ecs_path = bevy_ecs_path();
    let component_api_path = component_api_path(); 

    let attrs = match parse_component_attr(&ast) {
        Ok(attrs) => attrs,
        Err(e) => return e.into_compile_error().into(),
    };

    let storage = storage_path(&bevy_ecs_path, attrs.storage);

    let on_add = hook_register_on_add_call( attrs.on_add);
    let on_insert = hook_register_function_call(quote! {on_insert}, attrs.on_insert);
    let on_replace = hook_register_function_call(quote! {on_replace}, attrs.on_replace);
    let on_remove = hook_register_function_call(quote! {on_remove}, attrs.on_remove);

    ast.generics
        .make_where_clause()
        .predicates
        .push(parse_quote! { Self: Send + Sync + 'static });

    let struct_name = &ast.ident;
    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

    TokenStream::from(quote! {
        impl #impl_generics #bevy_ecs_path::component::Component for #struct_name #type_generics #where_clause {
            const STORAGE_TYPE: #bevy_ecs_path::component::StorageType = #storage;

            #[allow(unused_variables)]
            fn register_component_hooks(hooks: &mut #bevy_ecs_path::component::ComponentHooks) {
                #on_add
                #on_insert
                #on_replace
                #on_remove
            }
        }

        impl #impl_generics #component_api_path::ComponentAutoRegister for #struct_name #type_generics #where_clause {}
    })
}

const COMPONENT: &str = "component";
const STORAGE: &str = "storage";
const ON_ADD: &str = "on_add";
const ON_INSERT: &str = "on_insert";
const ON_REPLACE: &str = "on_replace";
const ON_REMOVE: &str = "on_remove";

struct Attrs {
    storage: StorageTy,
    on_add: Option<ExprPath>,
    on_insert: Option<ExprPath>,
    on_replace: Option<ExprPath>,
    on_remove: Option<ExprPath>,
}

#[derive(Clone, Copy)]
enum StorageTy {
    Table,
    SparseSet,
}

// values for `storage` attribute
const TABLE: &str = "Table";
const SPARSE_SET: &str = "SparseSet";

fn parse_component_attr(ast: &DeriveInput) -> Result<Attrs> {
    let mut attrs = Attrs {
        storage: StorageTy::Table,
        on_add: None,
        on_insert: None,
        on_replace: None,
        on_remove: None,
    };

    for meta in ast.attrs.iter().filter(|a| a.path().is_ident(COMPONENT)) {
        meta.parse_nested_meta(|nested| {
            if nested.path.is_ident(STORAGE) {
                attrs.storage = match nested.value()?.parse::<LitStr>()?.value() {
                    s if s == TABLE => StorageTy::Table,
                    s if s == SPARSE_SET => StorageTy::SparseSet,
                    s => {
                        return Err(nested.error(format!(
                            "Invalid storage type `{s}`, expected '{TABLE}' or '{SPARSE_SET}'.",
                        )));
                    }
                };
                Ok(())
            } else if nested.path.is_ident(ON_ADD) {
                attrs.on_add = Some(nested.value()?.parse::<ExprPath>()?);
                Ok(())
            } else if nested.path.is_ident(ON_INSERT) {
                attrs.on_insert = Some(nested.value()?.parse::<ExprPath>()?);
                Ok(())
            } else if nested.path.is_ident(ON_REPLACE) {
                attrs.on_replace = Some(nested.value()?.parse::<ExprPath>()?);
                Ok(())
            } else if nested.path.is_ident(ON_REMOVE) {
                attrs.on_remove = Some(nested.value()?.parse::<ExprPath>()?);
                Ok(())
            } else {
                Err(nested.error("Unsupported attribute"))
            }
        })?;
    }

    Ok(attrs)
}

fn storage_path(bevy_ecs_path: &Path, ty: StorageTy) -> TokenStream2 {
    let storage_type = match ty {
        StorageTy::Table => Ident::new("Table", Span::call_site()),
        StorageTy::SparseSet => Ident::new("SparseSet", Span::call_site()),
    };

    quote! { #bevy_ecs_path::component::StorageType::#storage_type }
}

fn hook_register_function_call(
    hook: TokenStream2,
    function: Option<ExprPath>,
) -> Option<TokenStream2> {
    function.map(|meta| quote! { hooks. #hook (#meta); })
}

fn hook_register_on_add_call(
    function: Option<ExprPath>,
) -> TokenStream2 {
    let component_api_path = component_api_path();
    let function = function.map(|meta| quote! { (#meta)(world, entity, id); });

    quote! {
        hooks.on_add(|mut world, entity, id| {
            #component_api_path::register_on_add::<Self>(world.reborrow());
            #function
        }); 
    }
}

