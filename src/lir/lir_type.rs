// SPDX-License-Identifier: Apache-2.0

use std::fmt;

use crate::lir::expressions::Operand;
use crate::sema::ast;
use crate::sema::ast::ArrayLength;

/// A struct type definition that is similar to the one in ast.rs,
/// extended with a Vector type, as we need a lower level representation of
/// String and DynamicBytes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructType {
    UserDefined(usize),
    SolAccountInfo,
    SolAccountMeta,
    SolParameters,
    ExternalFunction,
    /// Vector is used here to represent String and DynamicBytes
    Vector(Box<Type>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    // a UserType will be lower into a primitive type it is representing
    Bool,
    // Value is a integer, but width is platform dependent.
    // FunctionSelector is an integer, 4bytes on Polkadot and 8bytes on Solana
    Int(u16),
    /// Enum can be represented by Uint here.
    Uint(u16),
    /// bytes and string are encoded identically. In general, the encoding is similar to bytes1[].
    Bytes(u8),

    // Array can be represented as Ptr(Box<Array>)
    // Struct can be represented as Ptr(Box<Struct>)
    // Slice can be represented as Ptr(Box<Slice(Box<Type>)>)
    // BufferPointer is a Ptr to u8 (a byte)
    // DynamicBytes is a Ptr of Bytes
    // address is a ptr to bytes1[], representing the location of another contract. The length is platform dependent.
    // string is a ptr to bytes1[]
    /// pointer to another address space
    Ptr(Box<Type>),
    /// pointer to another storage address space, first bool is true for immutables
    StoragePtr(bool, Box<Type>),

    Function {
        params: Vec<Type>,
        returns: Vec<Type>,
    },

    Mapping {
        key_ty: Box<Type>,
        value_ty: Box<Type>,
    },

    Array(Box<Type>, Vec<ArrayLength>),
    Struct(StructType),
    // a slice is a ptr to struct that contains the ptr to data and the length
    Slice(Box<Type>),
}

#[derive(Clone, Debug)]
pub enum InternalCallTy {
    Static { cfg_no: usize },
    Dynamic(Operand),
    Builtin { ast_func_no: usize },
}

#[derive(Clone, Debug)]
pub struct PhiInput {
    pub operand: Operand,
    pub block_no: usize,
}

impl From<&ast::StructType> for StructType {
    fn from(ty: &ast::StructType) -> Self {
        match ty {
            ast::StructType::AccountInfo => StructType::SolAccountInfo,
            ast::StructType::AccountMeta => StructType::SolAccountMeta,
            ast::StructType::ExternalFunction => StructType::ExternalFunction,
            ast::StructType::SolParameters => StructType::SolParameters,
            ast::StructType::UserDefined(i) => StructType::UserDefined(*i),
        }
    }
}

impl fmt::Display for StructType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StructType::UserDefined(i) => write!(f, "{}", i),
            StructType::SolAccountInfo => write!(f, "SolAccountInfo"),
            StructType::SolAccountMeta => write!(f, "SolAccountMeta"),
            StructType::ExternalFunction => write!(f, "ExternalFunction"),
            StructType::SolParameters => write!(f, "SolParameters"),
            StructType::Vector(elem_ty) => write!(f, "vector<{}>", elem_ty),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Bool => write!(f, "bool"),
            Type::Int(width) => write!(f, "int{}", width),
            Type::Uint(width) => write!(f, "uint{}", width),
            Type::Bytes(width) => write!(f, "bytes{}", width),
            Type::Ptr(ty) => write!(f, "ptr<{}>", ty),
            Type::StoragePtr(immutable, ty) => {
                if *immutable {
                    write!(f, "const_storage_ptr<{}>", ty)
                } else {
                    write!(f, "storage_ptr<{}>", ty)
                }
            }
            Type::Array(ty, len) => {
                write!(f, "{}", ty)?;
                len.iter().for_each(|len| match len {
                    ArrayLength::Fixed(len) => write!(f, "[{}]", len).unwrap(),
                    ArrayLength::Dynamic => write!(f, "[]").unwrap(),
                    ArrayLength::AnyFixed => write!(f, "[?]").unwrap(),
                });
                Ok(())
            }
            Type::Slice(ty) => write!(f, "slice<{}>", ty),
            Type::Struct(ty) => write!(f, "struct.{}", ty),
            Type::Function { params, returns } => {
                write!(f, "fn(")?;
                for (i, param) in params.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ")")?;
                if !returns.is_empty() {
                    write!(f, " -> (")?;
                    for (i, ret) in returns.iter().enumerate() {
                        if i != 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", ret)?;
                    }
                    write!(f, ")")?;
                } else {
                    write!(f, " -> ()")?;
                }
                Ok(())
            }
            Type::Mapping { key_ty, value_ty } => {
                write!(f, "mapping<{} -> {}>", key_ty, value_ty)
            }
        }
    }
}

impl PhiInput {
    pub fn new(operand: Operand, block_no: usize) -> Self {
        Self { operand, block_no }
    }
}