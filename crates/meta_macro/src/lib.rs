use quote::quote;
use syn::ItemStruct;

#[proc_macro_attribute]
pub fn impl_contract_code(
    _: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item_struct = syn::parse_macro_input!(item as ItemStruct);
    let struct_ident = &item_struct.ident;
    quote!(
        #item_struct
        impl ContractCode for #struct_ident {
            fn get_byte_code(&self) -> Option<&String> {
                self.byte_code.as_ref()
            }
            fn get_code_hash(&self) -> Option<&String> {
                self.code_hash.as_ref()
            }
        }

    )
    .into()
}
