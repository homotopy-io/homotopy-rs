use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{FnArg, Ident, Item, Signature};

/// This macros autogenerates the functional component requirements for a yew function
/// This allows annotating individual functions as components
/// ```
/// #[derive(Default, Properties)]
/// pub struct CounterProps { count: u32 }
///
/// #[functional_component]
/// fn counter(props: &CounterProps) -> Html {
///     html!(<div>{props.count}</div>)
/// }
/// ```
#[proc_macro_attribute]
pub fn functional_component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    if let Item::Fn(ref f) = &syn::parse(item).unwrap() {
        let Signature { ident, inputs, .. } = &f.sig;

        // Create an uppercased version of the function
        // We follow Rust lint patterns where structs are CamelCase
        // and functions are snake_case
        let mut name = ident.to_string();
        name.get_mut(0..1).as_mut().unwrap().make_ascii_uppercase();
        let new_name = Ident::new(name.as_str(), Span::call_site().into());

        // We come up with a generated intermediate version to implement the function provider trait against
        // This will be in the form of `__genCounter`
        // The expanded body might look like:
        // ```
        //      struct __genCounter;
        //      impl FunctionProvider for __genCounter {...};
        //      export type Counter = FunctionComponent<__genCounter>
        // ```
        let gen_name = Ident::new(
            ["__gen", name.as_str()].concat().as_str(),
            Span::call_site().into(),
        );

        // Get the token of the input properties
        let prop_token = get_prop_name(inputs.first().unwrap());

        // Get the body of the functional component
        let function_body = &f.block;

        let gen = quote! {
            pub struct #gen_name;
            impl FunctionProvider for #gen_name {
                type TProps = #prop_token;

                fn run(props: &Self::TProps) -> Html {
                    #function_body
                }
            }

            pub type #new_name = FunctionComponent<#gen_name>;
        };
        gen.into()
    } else {
        unimplemented!("FunctionalComponent macro only works on functions!");
    }
}

fn get_prop_name(args: &FnArg) -> &Ident {
    // Ensure the props is an function arg
    // We've captured an explicitly typed function argument set
    if let FnArg::Typed(r) = args {
        // Ensure the prop is a reference type
        // This means the prop is being pased in by reference
        if let syn::Type::Reference(a) = &r.ty.as_ref() {
            // Now we grab out [&PathCounter] token and extract the ident
            if let syn::Type::Path(pathitem) = a.elem.as_ref() {
                return &pathitem.path.segments.first().unwrap().ident;
            }
        }
    }

    unimplemented!("Functional component declaration is malformed");
}
