use std::collections::{HashMap, HashSet};

include!("data/mw_anim_names.rs");
include!("data/pm_anim_names.rs");
include!("data/rc_anim_names.rs");

include!("data/mw_anim_root_names.rs");
include!("data/pm_anim_root_names.rs");
include!("data/rc_anim_root_names.rs");

include!("data/mw_anim_list.rs");
include!("data/pm_anim_list.rs");
include!("data/rc_anim_list.rs");

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    map(&out_dir, "mw_anim_names", MW_ANIM_NAMES);
    map(&out_dir, "pm_anim_names", PM_ANIM_NAMES);
    map(&out_dir, "rc_anim_names", RC_ANIM_NAMES);

    map(&out_dir, "mw_anim_root_names", MW_ANIM_ROOT_NAMES);
    map(&out_dir, "pm_anim_root_names", PM_ANIM_ROOT_NAMES);
    map(&out_dir, "rc_anim_root_names", RC_ANIM_ROOT_NAMES);

    map(&out_dir, "mw_anim_list", MW_ANIM_LIST);
    map(&out_dir, "pm_anim_list", PM_ANIM_LIST);
    map(&out_dir, "rc_anim_list", RC_ANIM_LIST);
}

fn map<const N: usize>(out_dir: &str, name: &str, names: &[(&[u8; N], &str)]) {
    let mut seen = HashMap::new();
    let mut hashes = HashSet::new();

    let data: Vec<_> = names
        .iter()
        .copied()
        .map(|(bytes, string)| {
            if seen.insert(bytes, string).is_some() {
                panic!(
                    "duplicate key `{}`, value `{}`",
                    bytes.escape_ascii(),
                    string
                );
            }
            let hash = fxhash::hash32(bytes);
            if !hashes.insert(hash) {
                panic!("duplicate hash for `{}`", bytes.escape_ascii());
            }
            (hash, bytes, string)
        })
        .collect();

    let span = proc_macro2::Span::call_site();

    let mut sorted = data.clone();
    // sort by hash
    sorted.sort_by_key(|(h, _b, _s)| *h);

    let (index, table): (Vec<_>, Vec<_>) = sorted
        .into_iter()
        .map(|(hash, bytes, string)| {
            let hash = syn::LitInt::new(&format!("0x{:08X}", hash), span);
            let bytes = syn::LitByteStr::new(bytes, span);
            let string = syn::LitStr::new(string, span);
            let tuple: syn::ExprTuple = syn::parse_quote! {
                (#bytes, #string)
            };
            (hash, tuple)
        })
        .unzip();

    let index: syn::ItemConst = syn::parse_quote! {
        pub(crate) const INDEX: &[u32] = &[
            #(#index,)*
        ];
    };

    let size = syn::LitInt::new(&format!("{}", N), span);

    let table: syn::ItemConst = syn::parse_quote! {
        #[allow(clippy::octal_escapes)]
        pub(crate) const TABLE: &[(&[u8; #size], &str)] = &[
            #(#table,)*
        ];
    };

    let file: syn::File = syn::parse_quote! {
        #index
        #table
    };
    let contents = prettyplease::unparse(&file);

    let path = format!("{}/{}.rs", out_dir, name);
    std::fs::write(path, contents).unwrap();

    let all: Vec<syn::ExprTuple> = data
        .into_iter()
        .map(|(hash, bytes, string)| {
            let hash = syn::LitInt::new(&format!("0x{:08X}", hash), span);
            let bytes = syn::LitByteStr::new(bytes, span);
            let string = syn::LitStr::new(string, span);
            syn::parse_quote! {
                (#hash, #bytes, #string)
            }
        })
        .collect();

    let all: syn::ItemConst = syn::parse_quote! {
        pub(super) const ALL: &[(u32, &[u8; #size], &str)] = &[
            #(#all,)*
        ];
    };
    let file: syn::File = syn::parse_quote! {
        #all
    };
    let contents = prettyplease::unparse(&file);

    let path = format!("{}/{}_test.rs", out_dir, name);
    std::fs::write(path, contents).unwrap();
}
