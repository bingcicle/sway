use {
    crate::{
        SwayParseTree, ParseTree, TreeType, AstNode, AstNodeContent, Declaration,
        FunctionDeclaration, TraitDeclaration, StructDeclaration, EnumDeclaration, AbiDeclaration,
        ConstantDeclaration, StorageDeclaration,
        Visibility, StructField, TypeParameter, EnumVariant, FunctionParameter, CodeBlock, Purity,
        Supertrait, TraitFn, ImplTrait, ImplSelf,
        CallPath, StorageField,
        TypeInfo,
        Expression,
    },
    new_parser_again::{
        Program, ProgramKind,
        Item, ItemStruct, ItemEnum, ItemFn, ItemTrait, ItemImpl, ItemAbi, ItemConst, ItemStorage,
        TypeField, GenericParams, FnArgs, FnSignature, Traits, PathType,
        PubToken, ImpureToken,
        Ty,
        CodeBlockContents,
        Expr,
    },
};

pub fn convert(program: Program) -> SwayParseTree {
    let span = program.span();
    SwayParseTree {
        tree_type: match program.kind {
            ProgramKind::Script { .. } => TreeType::Script,
            ProgramKind::Contract { .. } => TreeType::Contract,
            ProgramKind::Predicate { .. } => TreeType::Predicate,
            ProgramKind::Library { name, .. } => TreeType::Library { name },
        },
        tree: ParseTree {
            span,
            root_nodes: program.items.into_iter().map(item_to_ast_node).collect(),
        },
    }
}

fn item_to_ast_node(item: Item) -> AstNode {
    let span = item.span();
    let content = match item {
        Item::Use(item_use) => AstNodeContent::UseStatement(item_use),
        Item::Struct(item_struct) => {
            let struct_declaration = item_struct_to_struct_declaration(item_struct);
            AstNodeContent::Declaration(Declaration::StructDeclaration(struct_declaration))
        },
        Item::Enum(item_enum) => {
            let enum_declaration = item_enum_to_enum_declaration(item_enum);
            AstNodeContent::Declaration(Declaration::EnumDeclaration(enum_declaration))
        },
        Item::Fn(item_fn) => {
            let function_declaration = item_fn_to_function_declaration(item_fn);
            AstNodeContent::Declaration(Declaration::FunctionDeclaration(function_declaration))
        },
        Item::Trait(item_trait) => {
            let trait_declaration = item_trait_to_trait_declaration(item_trait);
            AstNodeContent::Declaration(Declaration::TraitDeclaration(trait_declaration))
        },
        Item::Impl(item_impl) => {
            let declaration = item_impl_to_declaration(item_impl);
            AstNodeContent::Declaration(declaration)
        },
        Item::Abi(item_abi) => {
            let abi_declaration = item_abi_to_abi_declaration(item_abi);
            AstNodeContent::Declaration(Declaration::AbiDeclaration(abi_declaration))
        },
        Item::Const(item_const) => {
            let constant_declaration = item_const_to_constant_declaration(item_const);
            AstNodeContent::Declaration(Declaration::ConstantDeclaration(constant_declaration))
        },
        Item::Storage(item_storage) => {
            let storage_declaration = item_storage_to_storage_declaration(item_storage);
            AstNodeContent::Declaration(Declaration::StorageDeclaration(storage_declaration))
        },
    };
    AstNode { span, content }
}

fn item_struct_to_struct_declaration(item_struct: ItemStruct) -> StructDeclaration {
    StructDeclaration {
        name: item_struct.name,
        fields: item_struct.fields.into_inner().into_iter().map(type_field_to_struct_field).collect(),
        type_parameters: generic_params_opt_to_type_parameters(item_struct.generics),
        visibility: pub_token_opt_to_visibility(item_struct.visibility),
    }
}

fn item_enum_to_enum_declaration(item_enum: ItemEnum) -> EnumDeclaration {
    let span = item_enum.span();
    EnumDeclaration {
        name: item_enum.name,
        type_parameters: generic_params_opt_to_type_parameters(item_enum.generics),
        variants: {
            item_enum
            .fields
            .into_inner()
            .into_iter()
            .enumerate()
            .map(|(tag, type_field)| type_field_to_enum_variant(type_field, tag))
            .collect()
        },
        span,
        visibility: pub_token_opt_to_visibility(item_enum.visibility),
    }
}

fn item_fn_to_function_declaration(item_fn: ItemFn) -> FunctionDeclaration {
    let span = item_fn.span();
    let return_type_span = match &item_fn.fn_signature.return_type_opt {
        Some((_right_arrow_token, ty)) => ty.span(),
        None => item_fn.fn_signature.span(),
    };
    FunctionDeclaration {
        purity: impure_token_opt_to_purity(item_fn.fn_signature.impure),
        name: item_fn.fn_signature.name,
        visibility: pub_token_opt_to_visibility(item_fn.fn_signature.visibility),
        body: code_block_contents_to_code_block(item_fn.body.into_inner()),
        parameters: fn_args_to_function_parameters(item_fn.fn_signature.arguments.into_inner()),
        span,
        return_type: match item_fn.fn_signature.return_type_opt {
            Some((_right_arrow, ty)) => ty_to_type_info(ty),
            None => TypeInfo::Tuple(Vec::new()),
        },
        type_parameters: generic_params_opt_to_type_parameters(item_fn.fn_signature.generics),
        return_type_span,
    }
}

fn item_trait_to_trait_declaration(item_trait: ItemTrait) -> TraitDeclaration {
    TraitDeclaration {
        name: item_trait.name,
        interface_surface: {
            item_trait
            .trait_items
            .into_inner()
            .into_iter()
            .map(|(fn_signature, _semicolon_token)| fn_signature_to_trait_fn(fn_signature))
            .collect()
        },
        methods: {
            item_trait
            .trait_defs_opt
            .into_iter()
            .map(|trait_defs| trait_defs.into_inner().into_iter().map(item_fn_to_function_declaration))
            .flatten()
            .collect()
        },
        type_parameters: Vec::new(),
        supertraits: {
            item_trait
            .super_traits
            .map(|(_colon_token, traits)| traits_to_supertraits(traits))
            .unwrap_or_default()
        },
        visibility: pub_token_opt_to_visibility(item_trait.visibility),
    }
}

fn item_impl_to_declaration(item_impl: ItemImpl) -> Declaration {
    let block_span = item_impl.span();
    let type_arguments_span = item_impl.ty.span();
    let type_implementing_for_span = item_impl.ty.span();
    let type_implementing_for = ty_to_type_info(item_impl.ty);
    let functions = {
        item_impl
        .contents
        .into_inner()
        .into_iter()
        .map(item_fn_to_function_declaration)
        .collect()
    };
    match item_impl.trait_opt {
        Some((path_type, _for_token)) => {
            let impl_trait = ImplTrait {
                trait_name: path_type_to_call_path(path_type),
                type_implementing_for,
                type_implementing_for_span,
                type_arguments: Vec::new(),
                functions,
                block_span,
                type_arguments_span,
            };
            Declaration::ImplTrait(impl_trait)
        },
        None => {
            let impl_self = ImplSelf {
                type_implementing_for,
                type_arguments: Vec::new(),
                functions,
                block_span,
                type_arguments_span,
                type_name_span: type_implementing_for_span,
            };
            Declaration::ImplSelf(impl_self)
        },
    }
}

fn item_abi_to_abi_declaration(item_abi: ItemAbi) -> AbiDeclaration {
    let span = item_abi.span();
    AbiDeclaration {
        name: item_abi.name,
        interface_surface: {
            item_abi
            .abi_items
            .into_inner()
            .into_iter()
            .map(|(fn_signature, _semicolon_token)| fn_signature_to_trait_fn(fn_signature))
            .collect()
        },
        methods: {
            item_abi
            .abi_defs_opt
            .into_iter()
            .map(|abi_defs| abi_defs.into_inner().into_iter().map(item_fn_to_function_declaration))
            .flatten()
            .collect()
        },
        span,
    }
}

fn item_const_to_constant_declaration(item_const: ItemConst) -> ConstantDeclaration {
    ConstantDeclaration {
        name: item_const.name,
        type_ascription: match item_const.ty_opt {
            Some((_colon_token, ty)) => ty_to_type_info(ty),
            None => TypeInfo::Unknown,
        },
        value: expr_to_expression(item_const.expr),
        visibility: pub_token_opt_to_visibility(item_const.visibility),
    }
}

fn item_storage_to_storage_declaration(item_storage: ItemStorage) -> StorageDeclaration {
    let span = item_storage.span();
    StorageDeclaration {
        span,
        fields: item_storage.fields.into_inner().into_iter().map(storage_field_to_storage_field).collect(),
    }
}

fn type_field_to_struct_field(type_field: TypeField) -> StructField {
    let span = type_field.span();
    let type_span = type_field.ty.span();
    StructField {
        name: type_field.name,
        r#type: ty_to_type_info(type_field.ty),
        span,
        type_span,
    }
}

fn generic_params_opt_to_type_parameters(generic_params_opt: Option<GenericParams>) -> Vec<TypeParameter> {
    let generic_params = match generic_params_opt {
        Some(generic_params) => generic_params,
        None => return Vec::new(),
    };
    generic_params
    .parameters
    .into_inner()
    .into_iter()
    .map(|ident| TypeParameter {
        name: TypeInfo::Custom { name: ident.clone() },
        name_ident: ident.clone(),
        trait_constraints: Vec::new(),
    })
    .collect()
}

fn pub_token_opt_to_visibility(pub_token_opt: Option<PubToken>) -> Visibility {
    match pub_token_opt {
        Some(..) => Visibility::Public,
        None => Visibility::Private,
    }
}

fn type_field_to_enum_variant(type_field: TypeField, tag: usize) -> EnumVariant {
    let span = type_field.span();
    EnumVariant {
        name: type_field.name,
        r#type: ty_to_type_info(type_field.ty),
        tag,
        span,
    }
}

fn impure_token_opt_to_purity(impure_token_opt: Option<ImpureToken>) -> Purity {
    match impure_token_opt {
        Some(..) => Purity::Impure,
        None => Purity::Pure,
    }
}

fn code_block_contents_to_code_block(_code_block_contents: CodeBlockContents) -> CodeBlock {
    todo!()
}

fn fn_args_to_function_parameters(_fn_args: FnArgs) -> Vec<FunctionParameter> {
    todo!()
}

fn ty_to_type_info(_ty: Ty) -> TypeInfo {
    todo!()
}

fn fn_signature_to_trait_fn(_fn_signature: FnSignature) -> TraitFn {
    todo!()
}

fn traits_to_supertraits(_traits: Traits) -> Vec<Supertrait> {
    todo!()
}

fn path_type_to_call_path(_path_type: PathType) -> CallPath {
    todo!()
}

fn expr_to_expression(_expr: Expr) -> Expression {
    todo!()
}

fn storage_field_to_storage_field(_storage_field: new_parser_again::StorageField) -> StorageField {
    todo!()
}

